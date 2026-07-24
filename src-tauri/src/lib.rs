// ============================================================
//  lib.rs – Tauri v2 backend using sysinfo 0.39.6
//  Full-featured system monitor – borrow‑checker fixed.
// ============================================================

use serde::Serialize;
use std::sync::Mutex;
use std::time::Instant;
use sysinfo::{Components, Disks, Networks, System, ProcessesToUpdate};
use tauri::State;

// -----------------------------------------------------------------------------
//  Data structures – expanded frontend contract
// -----------------------------------------------------------------------------

#[derive(Serialize, Clone)]
pub struct CpuCoreStats {
    pub usage: f32,
    pub frequency: u64, // MHz
}

#[derive(Serialize, Clone)]
pub struct CpuStats {
    pub usage: f32,
    pub model: String,
    pub vendor: String,
    pub architecture: String,
    pub cores: usize,
    pub threads: usize,
    pub frequency: u64, // average or first core
    pub per_core: Vec<CpuCoreStats>,
}

#[derive(Serialize, Clone)]
pub struct GpuStats {
    pub available: bool,
    pub usage: f32,
    pub name: String,
    pub vram_total: u64,
    pub vram_used: u64,
}

#[derive(Serialize, Clone)]
pub struct RamStats {
    pub used: u64,
    pub total: u64,
    pub available: u64,
    pub usage: f32,
}

#[derive(Serialize, Clone)]
pub struct SwapStats {
    pub used: u64,
    pub total: u64,
    pub free: u64,
    pub usage: f32,
}

#[derive(Serialize, Clone)]
pub struct DiskStats {
    pub name: String,
    pub mount_point: String,
    pub file_system: String,
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub usage: f32,
    pub is_removable: bool,
    pub kind: String, // "SSD", "HDD", "Unknown"
}

#[derive(Serialize, Clone)]
pub struct NetworkInterfaceStats {
    pub name: String,
    pub received: u64,
    pub transmitted: u64,
    pub packets_received: u64,
    pub packets_transmitted: u64,
    pub errors_received: u64,
    pub errors_transmitted: u64,
}

#[derive(Serialize, Clone)]
pub struct NetworkStats {
    pub download: u64, // bytes per second (total)
    pub upload: u64,   // bytes per second (total)
    pub interfaces: Vec<NetworkInterfaceStats>,
}

#[derive(Serialize, Clone)]
pub struct TemperatureStats {
    pub cpu: Option<f32>,
    pub gpu: Option<f32>,
}

#[derive(Serialize, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub memory_percent: f32,
    pub status: String,
    pub run_time: u64,
}

#[derive(Serialize, Clone)]
pub struct SystemInfo {
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub uptime: u64,
    pub boot_time: u64,
    pub load_avg_1: f64,
    pub load_avg_5: f64,
    pub load_avg_15: f64,
}

#[derive(Serialize, Clone)]
pub struct AllStats {
    pub cpu: CpuStats,
    pub gpu: GpuStats,
    pub ram: RamStats,
    pub swap: SwapStats,
    pub storage: Vec<DiskStats>,
    pub network: NetworkStats,
    pub temperature: TemperatureStats,
    pub processes: Vec<ProcessInfo>,
    pub system: SystemInfo,
}

// -----------------------------------------------------------------------------
//  GPU provider abstraction (no‑op for now)
// -----------------------------------------------------------------------------

pub trait GpuProvider: Send + Sync {
    fn get_gpu_stats(&self) -> GpuStats;
}

pub struct NoGpuProvider;

impl GpuProvider for NoGpuProvider {
    fn get_gpu_stats(&self) -> GpuStats {
        GpuStats {
            available: false,
            usage: 0.0,
            name: "Unavailable".to_string(),
            vram_total: 0,
            vram_used: 0,
        }
    }
}

// -----------------------------------------------------------------------------
//  Application state – holds reusable resources
// -----------------------------------------------------------------------------

struct InnerState {
    sys: System,
    disks: Disks,
    networks: Networks,
    components: Components,
    net_cache: Option<(u64, u64, Instant)>, // (prev_rx, prev_tx, timestamp)
}

struct AppState {
    inner: Mutex<InnerState>,
}

// -----------------------------------------------------------------------------
//  Helper functions
// -----------------------------------------------------------------------------

fn get_cpu_stats(sys: &System) -> CpuStats {
    let cpus = sys.cpus();
    let first = cpus.first();

    let usage = sys.global_cpu_usage();
    let model = first
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let vendor = first
        .map(|c| c.vendor_id().to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let architecture = std::env::consts::ARCH.to_string();
    let cores = System::physical_core_count().unwrap_or(0);
    let threads = cpus.len();
    let frequency = first.map(|c| c.frequency()).unwrap_or(0);

    let per_core: Vec<CpuCoreStats> = cpus
        .iter()
        .map(|c| CpuCoreStats {
            usage: c.cpu_usage(),
            frequency: c.frequency(),
        })
        .collect();

    CpuStats {
        usage,
        model,
        vendor,
        architecture,
        cores,
        threads,
        frequency,
        per_core,
    }
}

fn get_ram_stats(sys: &System) -> RamStats {
    let total = sys.total_memory();
    let used = sys.used_memory();
    let available = sys.available_memory();
    let usage = if total > 0 {
        (used as f32 / total as f32) * 100.0
    } else {
        0.0
    };

    RamStats {
        used,
        total,
        available,
        usage,
    }
}

fn get_swap_stats(sys: &System) -> SwapStats {
    let total = sys.total_swap();
    let used = sys.used_swap();
    let free = total.saturating_sub(used);
    let usage = if total > 0 {
        (used as f32 / total as f32) * 100.0
    } else {
        0.0
    };

    SwapStats { used, total, free, usage }
}

fn get_disk_stats(disks: &Disks) -> Vec<DiskStats> {
    disks
        .iter()
        .map(|disk| {
            let total = disk.total_space();
            let free = disk.available_space();
            let used = total - free;
            let usage = if total > 0 {
                (used as f32 / total as f32) * 100.0
            } else {
                0.0
            };

            let mount_point = disk
                .mount_point()
                .to_str()
                .unwrap_or("Unknown")
                .to_string();
            let name = disk.name().to_string_lossy().to_string();
            let file_system = disk.file_system().to_string_lossy().to_string();
            let is_removable = disk.is_removable();
            let kind = match disk.kind() {
                sysinfo::DiskKind::SSD => "SSD",
                sysinfo::DiskKind::HDD => "HDD",
                _ => "Unknown",
            }
            .to_string();

            DiskStats {
                name,
                mount_point,
                file_system,
                total,
                used,
                free,
                usage,
                is_removable,
                kind,
            }
        })
        .collect()
}

/// Collects network interface stats and total RX/TX (immutable).
fn collect_network_data(networks: &Networks) -> (Vec<NetworkInterfaceStats>, u64, u64) {
    let mut interfaces = Vec::with_capacity(networks.len());
    let mut total_rx = 0u64;
    let mut total_tx = 0u64;

    for (name, data) in networks.iter() {
        let rx = data.received();
        let tx = data.transmitted();
        total_rx += rx;
        total_tx += tx;

        interfaces.push(NetworkInterfaceStats {
            name: name.clone(),
            received: rx,
            transmitted: tx,
            packets_received: data.packets_received(),
            packets_transmitted: data.packets_transmitted(),
            errors_received: data.errors_on_received(),
            errors_transmitted: data.errors_on_transmitted(),
        });
    }

    (interfaces, total_rx, total_tx)
}

/// Computes download/upload speeds from current totals and cache (mutable).
fn compute_network_speeds(
    current_rx: u64,
    current_tx: u64,
    cache: &mut Option<(u64, u64, Instant)>,
) -> (u64, u64) {
    let now = Instant::now();
    let (download, upload) = if let Some((prev_rx, prev_tx, prev_time)) = cache.as_ref() {
        let elapsed = now.duration_since(*prev_time).as_secs_f64();
        if elapsed > 0.0 {
            let down_delta = if current_rx >= *prev_rx {
                current_rx - *prev_rx
            } else {
                0
            };
            let up_delta = if current_tx >= *prev_tx {
                current_tx - *prev_tx
            } else {
                0
            };
            (
                (down_delta as f64 / elapsed) as u64,
                (up_delta as f64 / elapsed) as u64,
            )
        } else {
            (0, 0)
        }
    } else {
        (0, 0)
    };

    *cache = Some((current_rx, current_tx, now));
    (download, upload)
}

fn get_temperature_stats(components: &Components) -> TemperatureStats {
    let mut cpu_temp = None;
    let mut gpu_temp = None;

    for comp in components.iter() {
        let label = comp.label().to_lowercase();
        if label.contains("cpu") || label.contains("core") {
            cpu_temp = comp.temperature();
        } else if label.contains("gpu") {
            gpu_temp = comp.temperature();
        }
        if cpu_temp.is_some() && gpu_temp.is_some() {
            break;
        }
    }

    TemperatureStats {
        cpu: cpu_temp,
        gpu: gpu_temp,
    }
}

/// Returns top N processes by CPU usage.
fn get_top_processes(sys: &System, limit: usize) -> Vec<ProcessInfo> {
    let mut processes: Vec<ProcessInfo> = sys
        .processes()
        .iter()
        .filter_map(|(_pid, proc)| {
            let cpu = proc.cpu_usage();
            let mem = proc.memory();
            let total_mem = sys.total_memory();
            let mem_percent = if total_mem > 0 {
                (mem as f32 / total_mem as f32) * 100.0
            } else {
                0.0
            };

            Some(ProcessInfo {
                pid: proc.pid().as_u32(),
                name: proc.name().to_string_lossy().to_string(),
                cpu_usage: cpu,
                memory_usage: mem,
                memory_percent: mem_percent,
                status: format!("{:?}", proc.status()),
                run_time: proc.run_time(),
            })
        })
        .collect();

    processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));
    processes.truncate(limit);
    processes
}

/// System information – uses static methods.
fn get_system_info() -> SystemInfo {
    let hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());
    let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
    let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
    let kernel_version = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
    let uptime = System::uptime();
    let boot_time = System::boot_time();
    let load_avg = System::load_average();

    SystemInfo {
        hostname,
        os_name,
        os_version,
        kernel_version,
        uptime,
        boot_time,
        load_avg_1: load_avg.one,
        load_avg_5: load_avg.five,
        load_avg_15: load_avg.fifteen,
    }
}

// -----------------------------------------------------------------------------
//  Tauri command
// -----------------------------------------------------------------------------

#[tauri::command]
fn get_stats(state: State<AppState>) -> AllStats {
    let mut inner = state
        .inner
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    // Refresh all needed subsystems.
    inner.sys.refresh_cpu_all();
    inner.sys.refresh_memory();
    inner.sys.refresh_processes(ProcessesToUpdate::All, true);
    inner.disks.refresh(true);
    inner.networks.refresh(true);
    inner.components.refresh(true);

    // CPU
    let cpu = get_cpu_stats(&inner.sys);

    // RAM
    let ram = get_ram_stats(&inner.sys);

    // Swap
    let swap = get_swap_stats(&inner.sys);

    // Storage
    let storage = get_disk_stats(&inner.disks);

    // Temperature
    let temperature = get_temperature_stats(&inner.components);

    // System info (static, no borrow)
    let system = get_system_info();

    // Network – split collection and speed computation to avoid borrow conflicts.
    let (interfaces, total_rx, total_tx) = collect_network_data(&inner.networks);
    let (download, upload) = compute_network_speeds(total_rx, total_tx, &mut inner.net_cache);
    let network = NetworkStats {
        download,
        upload,
        interfaces,
    };

    // Processes
    let processes = get_top_processes(&inner.sys, 10);

    // GPU (no‑op)
    let gpu_provider = NoGpuProvider;
    let gpu = gpu_provider.get_gpu_stats();

    AllStats {
        cpu,
        gpu,
        ram,
        swap,
        storage,
        network,
        temperature,
        processes,
        system,
    }
}

// -----------------------------------------------------------------------------
//  Tauri entry point
// -----------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            inner: Mutex::new(InnerState {
                sys: System::new(),
                disks: Disks::new_with_refreshed_list(),
                networks: Networks::new_with_refreshed_list(),
                components: Components::new_with_refreshed_list(),
                net_cache: None,
            }),
        })
        .invoke_handler(tauri::generate_handler![get_stats])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}