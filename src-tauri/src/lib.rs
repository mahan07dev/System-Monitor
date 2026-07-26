// ============================================================
//  lib.rs – Tauri v2 backend using sysinfo
//  Fully optimized, cross-platform system monitor backend
// ============================================================

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::sync::Mutex;
use std::time::Instant;
use sysinfo::{Components, Disks, Networks, ProcessesToUpdate, System};
use tauri::State;

// -----------------------------------------------------------------------------
//  Data Structures
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
    pub frequency: u64,
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
    pub download: u64, // bytes/sec
    pub upload: u64,   // bytes/sec
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
//  GPU Provider Abstraction
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
//  Application State
// -----------------------------------------------------------------------------

struct InnerState {
    sys: System,
    disks: Disks,
    networks: Networks,
    components: Components,
    net_cache: Option<(u64, u64, Instant)>,
    initialized: bool,
}

struct AppState {
    inner: Mutex<InnerState>,
}

// -----------------------------------------------------------------------------
//  Helper Functions
// -----------------------------------------------------------------------------

fn get_cpu_stats(sys: &System) -> CpuStats {
    let cpus = sys.cpus();
    let first = cpus.first();

    let usage = sys.global_cpu_usage();
    let model = first
        .map(|c| c.brand().trim().to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let vendor = first
        .map(|c| c.vendor_id().trim().to_string())
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
            let used = total.saturating_sub(free);
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

fn compute_network_speeds(
    current_rx: u64,
    current_tx: u64,
    cache: &mut Option<(u64, u64, Instant)>,
) -> (u64, u64) {
    let now = Instant::now();
    let (download, upload) = if let Some((prev_rx, prev_tx, prev_time)) = cache.as_ref() {
        let elapsed = now.duration_since(*prev_time).as_secs_f64();
        if elapsed > 0.0 {
            let down_delta = current_rx.saturating_sub(*prev_rx);
            let up_delta = current_tx.saturating_sub(*prev_tx);
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
        if label.contains("cpu") || label.contains("core") || label.contains("package") {
            if cpu_temp.is_none() {
                cpu_temp = comp.temperature();
            }
        } else if label.contains("gpu") {
            if gpu_temp.is_none() {
                gpu_temp = comp.temperature();
            }
        }
    }

    TemperatureStats {
        cpu: cpu_temp,
        gpu: gpu_temp,
    }
}

fn get_top_processes(sys: &System, limit: usize) -> Vec<ProcessInfo> {
    let total_mem = sys.total_memory();

    let mut processes: Vec<ProcessInfo> = sys
        .processes()
        .values()
        .map(|proc| {
            let cpu_usage = proc.cpu_usage();
            let memory_usage = proc.memory();
            let memory_percent = if total_mem > 0 {
                (memory_usage as f32 / total_mem as f32) * 100.0
            } else {
                0.0
            };

            ProcessInfo {
                pid: proc.pid().as_u32(),
                name: proc.name().to_string_lossy().to_string(),
                cpu_usage,
                memory_usage,
                memory_percent, // Fixed: using correctly matched variable
                status: format!("{:?}", proc.status()),
                run_time: proc.run_time(),
            }
        })
        .collect();

    processes.sort_unstable_by(|a, b| {
        b.cpu_usage
            .partial_cmp(&a.cpu_usage)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    processes.truncate(limit);
    processes
}

fn get_system_info() -> SystemInfo {
    let load_avg = System::load_average();

    SystemInfo {
        hostname: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
        os_name: System::name().unwrap_or_else(|| "Unknown".to_string()),
        os_version: System::os_version().unwrap_or_else(|| "Unknown".to_string()),
        kernel_version: System::kernel_version().unwrap_or_else(|| "Unknown".to_string()),
        uptime: System::uptime(),
        boot_time: System::boot_time(),
        load_avg_1: load_avg.one,
        load_avg_5: load_avg.five,
        load_avg_15: load_avg.fifteen,
    }
}

// -----------------------------------------------------------------------------
//  Tauri Command
// -----------------------------------------------------------------------------

#[tauri::command]
fn get_stats(state: State<AppState>) -> AllStats {
    let mut inner = state
        .inner
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    // Lazy full device list population on first request (avoids Windows launch freeze)
    if !inner.initialized {
        inner.disks.refresh(true);
        inner.networks.refresh(true);
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            inner.components.refresh(true);
        }));
        inner.initialized = true;
    }

    // Refresh telemetry
    inner.sys.refresh_cpu_all();
    inner.sys.refresh_memory();
    inner.sys.refresh_processes(ProcessesToUpdate::All, true);
    inner.disks.refresh(true);
    inner.networks.refresh(true);

    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        inner.components.refresh(true);
    }));

    let cpu = get_cpu_stats(&inner.sys);
    let ram = get_ram_stats(&inner.sys);
    let swap = get_swap_stats(&inner.sys);
    let storage = get_disk_stats(&inner.disks);
    let temperature = get_temperature_stats(&inner.components);
    let system = get_system_info();

    let (interfaces, total_rx, total_tx) = collect_network_data(&inner.networks);
    let (download, upload) = compute_network_speeds(total_rx, total_tx, &mut inner.net_cache);
    let network = NetworkStats {
        download,
        upload,
        interfaces,
    };

    let processes = get_top_processes(&inner.sys, 10);
    let gpu = NoGpuProvider.get_gpu_stats();

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
//  Tauri Entry Point
// -----------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Set panic hook so any crash logs to stdout/stderr in debug mode
    std::panic::set_hook(Box::new(|info| {
        eprintln!("Application Panic: {:?}", info);
    }));

    let initial_state = AppState {
        inner: Mutex::new(InnerState {
            sys: System::new_all(),
            disks: Disks::new(),
            networks: Networks::new(),
            components: Components::new(),
            net_cache: None,
            initialized: false,
        }),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(initial_state)
        .invoke_handler(tauri::generate_handler![get_stats])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}