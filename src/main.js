// ============================================================
//  main.js – frontend logic (vanilla JS, Tauri v2)
// ============================================================

const { invoke } = window.__TAURI__.core;

const { openUrl } = window.__TAURI__.opener;

document.getElementById('githubLink').addEventListener('click', async (e) => {
    e.preventDefault();
    const url = e.currentTarget.getAttribute('href');
    await openUrl(url);
});

// ----- Modal logic -----
const helpBtn = document.getElementById('helpBtn');
const modalOverlay = document.getElementById('modalOverlay');
const modalClose = document.getElementById('modalClose');

helpBtn.addEventListener('click', () => {
    modalOverlay.classList.add('active');
});
modalClose.addEventListener('click', () => {
    modalOverlay.classList.remove('active');
});
modalOverlay.addEventListener('click', (e) => {
    if (e.target === modalOverlay) modalOverlay.classList.remove('active');
});

// ----- DOM refs -----
const $ = (id) => document.getElementById(id);

// CPU
const cpuRing = $('cpu-ring');
const cpuTempRing = $('cpu-temp-ring'); // CPU Temp SVG ring
const cpuPercent = $('cpu-percent');
const cpuModel = $('cpu-model');
const cpuCores = $('cpu-cores');
const cpuFreq = $('cpu-freq');
const cpuBadge = $('cpu-usage-badge');
const perCoreContainer = $('per-core-container');

// RAM
const ramBar = $('ram-bar');
const ramCacheBar = $('ram-cache-bar'); // Segmented Cached Memory Bar
const ramPercent = $('ram-percent');
const ramUsed = $('ram-used');
const ramTotal = $('ram-total');
const ramAvailable = $('ram-available');
const ramBadge = $('ram-usage-badge');
const swapBar = $('swap-bar');
const swapPercent = $('swap-percent');
const swapUsed = $('swap-used');
const swapTotal = $('swap-total');

// Storage
const storageList = $('storage-list');
const storageCount = $('storage-count');

// Network
const downloadSpeed = $('download-speed');
const uploadSpeed = $('upload-speed');
const netGraph = $('network-graph');
const interfaceList = $('interface-list');

// Temperature
const cpuTemp = $('cpu-temp');
const gpuTemp = $('gpu-temp');
const tempStatus = $('temp-status');

// Processes
const processBody = $('process-body');
const processCount = $('process-count');

// System
const sysHostname = $('sys-hostname');
const sysOs = $('sys-os');
const sysKernel = $('sys-kernel');
const sysLoad = $('sys-load');
const sysBoot = $('sys-boot');
const uptimeEl = $('uptime');

// ----- Graph state -----
const MAX_POINTS = 30;
let downloadHistory = new Array(MAX_POINTS).fill(0);
let uploadHistory = new Array(MAX_POINTS).fill(0);

// ----- Helpers -----
function formatSpeed(bytesPerSec) {
    if (bytesPerSec >= 1_000_000) return (bytesPerSec / 1_000_000).toFixed(1) + ' MB/s';
    if (bytesPerSec >= 1_000) return (bytesPerSec / 1_000).toFixed(1) + ' KB/s';
    return bytesPerSec.toFixed(0) + ' B/s';
}

function formatBytesShort(bytes) {
    if (bytes >= 1_000_000_000) return (bytes / 1_000_000_000).toFixed(1) + ' GB';
    if (bytes >= 1_000_000) return (bytes / 1_000_000).toFixed(1) + ' MB';
    return (bytes / 1_000).toFixed(1) + ' KB';
}

function formatUptime(seconds) {
    const d = Math.floor(seconds / 86400);
    const h = Math.floor((seconds % 86400) / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const parts = [];
    if (d) parts.push(d + 'd');
    if (h) parts.push(h + 'h');
    if (m) parts.push(m + 'm');
    return parts.join(' ') || '0m';
}

// ----- Draw graph -----
function drawGraph() {
    const ctx = netGraph.getContext('2d');
    const w = netGraph.width;
    const h = netGraph.height;
    ctx.clearRect(0, 0, w, h);

    ctx.strokeStyle = 'rgba(255,255,255,0.03)';
    ctx.lineWidth = 0.5;
    for (let y = 0; y <= h; y += h / 4) {
        ctx.beginPath();
        ctx.moveTo(0, y);
        ctx.lineTo(w, y);
        ctx.stroke();
    }

    const allData = [...downloadHistory, ...uploadHistory];
    const maxVal = Math.max(1, ...allData);

    const drawCurve = (data, color) => {
        ctx.beginPath();
        ctx.strokeStyle = color;
        ctx.lineWidth = 2;
        ctx.shadowBlur = 8;
        ctx.shadowColor = '#3B82F6';
        ctx.lineCap = 'round';
        ctx.lineJoin = 'round';
        for (let i = 0; i < data.length; i++) {
            const x = (i / (data.length - 1)) * w;
            const y = h - (data[i] / maxVal) * h * 0.85 - 4;
            if (i === 0) ctx.moveTo(x, y);
            else ctx.lineTo(x, y);
        }
        ctx.stroke();
    };

    drawCurve(downloadHistory, '#4a9eff');
    drawCurve(uploadHistory, '#f7b731');
}

// ----- Update UI -----
function updateUI(data) {
    // ---- CPU Usage Ring ----
    const cpu = data.cpu;
    cpuRing.style.strokeDashoffset = (314.159 * (1 - cpu.usage / 100));
    cpuPercent.textContent = cpu.usage.toFixed(1) + '%';
    cpuBadge.textContent = cpu.usage.toFixed(1) + '%';
    cpuModel.textContent = cpu.model || 'Unknown';
    cpuCores.textContent = `${cpu.cores} cores / ${cpu.threads} threads`;
    cpuFreq.textContent = cpu.frequency ? `${cpu.frequency} MHz` : '—';

    perCoreContainer.innerHTML = '';
    if (cpu.per_core && cpu.per_core.length) {
        cpu.per_core.forEach((core, idx) => {
            const div = document.createElement('div');
            div.className = 'per-core-item';
            div.innerHTML = `
                <span class="core-idx">${idx}</span>
                <div class="core-bar-track"><div class="core-bar-fill" style="width:${core.usage}%"></div></div>
                <span class="core-pct">${core.usage.toFixed(0)}%</span>
            `;
            perCoreContainer.appendChild(div);
        });
    }

    // ---- CPU Temperature Ring ----
    const MAX_TEMP = 100;       // 100°C = 100% full ring
    const DANGEROUS_TEMP = 90; // 90°C+ threshold triggers critical warning

    const temp = data.temperature;
    if (temp && temp.cpu !== null && temp.cpu !== undefined) {
        const cTemp = temp.cpu;
        cpuTemp.textContent = cTemp.toFixed(1) + '°C';
        
        // Compute SVG stroke offset (0°C = 0%, 100°C = 100%)
        const tempFillRatio = Math.min(Math.max(cTemp / MAX_TEMP, 0), 1);
        if (cpuTempRing) {
            cpuTempRing.style.strokeDashoffset = (314.159 * (1 - tempFillRatio));
        }

        // Dynamic temp status badge in card header
        if (cTemp >= DANGEROUS_TEMP) {
            tempStatus.textContent = '🔥 DANGER';
            tempStatus.style.color = '#ef4444';
        } else if (cTemp >= 75) {
            tempStatus.textContent = '⚠️ WARM';
            tempStatus.style.color = 'var(--accent-warning)';
        } else {
            tempStatus.textContent = '● LIVE';
            tempStatus.style.color = 'var(--accent-cyan)';
        }
    } else {
        cpuTemp.textContent = '—';
        if (cpuTempRing) cpuTempRing.style.strokeDashoffset = 314.159;
        tempStatus.textContent = 'Unavailable';
    }
    gpuTemp.textContent = (temp && temp.gpu !== null) ? temp.gpu.toFixed(1) + '°C' : '—';

    // ---- RAM (Active + Dynamic Cached) ----
    const ram = data.ram;
    const activePct = ram.usage; // Active Process Memory %

    // Calculate dynamic cached RAM bytes & percentage
    const cachedBytes = Math.max(0, ram.total - ram.available - ram.used);
    const cachedPct = ram.total > 0 ? (cachedBytes / ram.total) * 100 : 0;

    ramBar.style.width = activePct.toFixed(1) + '%';
    if (ramCacheBar) {
        ramCacheBar.style.width = cachedPct.toFixed(1) + '%';
    }

    ramPercent.textContent = Math.min(100, activePct + cachedPct).toFixed(1) + '%';
    ramBadge.textContent = activePct.toFixed(1) + '%';
    ramUsed.textContent = formatBytesShort(ram.used);
    ramTotal.textContent = formatBytesShort(ram.total);
    ramAvailable.textContent = formatBytesShort(ram.available);

    // ---- Swap ----
    const swap = data.swap;
    const swapPct = swap.usage;
    swapBar.style.width = swapPct + '%';
    swapPercent.textContent = swapPct.toFixed(1) + '%';
    swapUsed.textContent = formatBytesShort(swap.used);
    swapTotal.textContent = formatBytesShort(swap.total);

    // ---- Storage ----
    storageList.innerHTML = '';
    if (data.storage && data.storage.length) {
        data.storage.forEach(disk => {
            const div = document.createElement('div');
            div.className = 'storage-item';
            const pct = disk.usage;
            div.innerHTML = `
                <div class="storage-meta">
                    <span>${disk.name} (${disk.mount_point})</span>
                    <span>${disk.file_system} · ${disk.kind} ${disk.is_removable ? '💾' : ''}</span>
                </div>
                <div class="storage-bar-container">
                    <div class="storage-bar-track"><div class="storage-bar-fill" style="width:${pct}%; background: ${pct > 80 ? '#ff6b6b' : pct > 60 ? '#f7b731' : '#4a9eff'};"></div></div>
                    <span class="storage-bar-label">${pct.toFixed(0)}%</span>
                </div>
                <div class="storage-meta">
                    <span>Used ${formatBytesShort(disk.used)}</span>
                    <span>Free ${formatBytesShort(disk.free)}</span>
                    <span>Total ${formatBytesShort(disk.total)}</span>
                </div>
            `;
            storageList.appendChild(div);
        });
        storageCount.textContent = data.storage.length + ' drives';
    } else {
        storageCount.textContent = '0 drives';
        storageList.innerHTML = '<div style="opacity:0.4; text-align:center; padding:0.5rem 0;">No disks found</div>';
    }

    // ---- Network ----
    const net = data.network;
    downloadSpeed.textContent = '↓ ' + formatSpeed(net.download);
    uploadSpeed.textContent = '↑ ' + formatSpeed(net.upload);
    downloadHistory.push(net.download);
    downloadHistory.shift();
    uploadHistory.push(net.upload);
    uploadHistory.shift();
    drawGraph();

    interfaceList.innerHTML = '';
    if (net.interfaces && net.interfaces.length) {
        net.interfaces.forEach(iface => {
            const span = document.createElement('span');
            span.textContent = `${iface.name}: ↓${formatBytesShort(iface.received)} ↑${formatBytesShort(iface.transmitted)}`;
            interfaceList.appendChild(span);
        });
    }

    // ---- Processes ----
    processBody.innerHTML = '';
    if (data.processes && data.processes.length) {
        data.processes.forEach(proc => {
            const tr = document.createElement('tr');
            tr.innerHTML = `
                <td class="process-name" title="${proc.name}">${proc.name}</td>
                <td>${proc.cpu_usage.toFixed(1)}%</td>
                <td>${proc.memory_percent.toFixed(1)}%</td>
            `;
            processBody.appendChild(tr);
        });
        processCount.textContent = data.processes.length;
    } else {
        processCount.textContent = '0';
        processBody.innerHTML = '<tr><td colspan="3" style="opacity:0.4; text-align:center;">No processes</td></tr>';
    }

    // ---- System ----
    const sys = data.system;
    sysHostname.textContent = sys.hostname || '—';
    sysOs.textContent = sys.os_name || '—';
    sysKernel.textContent = sys.kernel_version || '—';
    sysLoad.textContent = `${sys.load_avg_1.toFixed(2)}, ${sys.load_avg_5.toFixed(2)}, ${sys.load_avg_15.toFixed(2)}`;
    if (sys.boot_time) {
        const boot = new Date(sys.boot_time * 1000);
        sysBoot.textContent = boot.toLocaleString();
    } else {
        sysBoot.textContent = '—';
    }
    uptimeEl.textContent = `Uptime ${formatUptime(sys.uptime)}`;

    // Mark cards as loaded
    document.querySelectorAll('.card').forEach(card => {
        card.dataset.loaded = 'true';
    });
}

// ----- Poll -----
async function pollData() {
    try {
        const result = await invoke('get_stats');
        if (result && typeof result === 'object' && 'Err' in result) {
            console.error('Backend error:', result.Err);
            return;
        }
        updateUI(result);
    } catch (err) {
        console.error('Poll error:', err);
    }
}

// ----- Start -----
pollData();
setInterval(pollData, 1000);