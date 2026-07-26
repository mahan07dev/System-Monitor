<p align="center">
  <img src="assets/banner.png" alt="System Monitor Logo" width="300">
</p>

<p align="center">
  <strong>A modern, lightweight and blazing-fast cross-platform system monitor built with Rust and Tauri.</strong>
</p>

<p align="center">
  Monitor your computer in real time with a clean native interface, low resource usage, and detailed hardware statistics.
</p>

<p align="center">

![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20macOS-blue?style=for-the-badge)
![Rust](https://img.shields.io/badge/Rust-Backend-orange?style=for-the-badge&logo=rust)
![Tauri](https://img.shields.io/badge/Tauri-v2-24C8DB?style=for-the-badge&logo=tauri)
![License](https://img.shields.io/github/license/mahan07dev/System-Monitor?style=for-the-badge)
![Release](https://img.shields.io/github/v/release/mahan07dev/System-Monitor?style=for-the-badge)
![Downloads](https://img.shields.io/github/downloads/mahan07dev/System-Monitor/total?style=for-the-badge)
![Stars](https://img.shields.io/github/stars/mahan07dev/System-Monitor?style=for-the-badge)
![Issues](https://img.shields.io/github/issues/mahan07dev/System-Monitor?style=for-the-badge)

</p>

---

# ⚡ Quick Install

Install System Monitor instantly with a single terminal command.

### 🐧 Linux & 🍎 macOS (Terminal)
```bash
curl -fsSL https://raw.githubusercontent.com/mahan07dev/System-Monitor/main/install.sh | bash
```

### 🪟 Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/mahan07dev/System-Monitor/main/install.ps1 | iex
```

---

# ✨ Overview

**System Monitor** is a modern desktop application built using **Rust** and **Tauri v2** that provides fast, accurate, and real-time system monitoring while consuming minimal system resources.

Unlike Electron-based alternatives, System Monitor focuses on **native performance**, **small footprint**, and **instant startup responsiveness** without sacrificing a polished user experience.

Whether you are checking CPU temperatures, tracking RAM usage, measuring network bandwidth, or inspecting running processes, System Monitor provides all essential telemetry in a clean, intuitive dashboard.

---

# 📸 Screenshot

<p align="center">
  <img src="assets/screenshot.png" alt="Application Screenshot">
</p>

---

# 🚀 Features

### 🖥️ CPU Monitoring
- Real-time CPU usage percentage
- CPU temperature monitoring
- CPU clock speed & frequency
- Model, vendor, and architecture info
- Individual core usage breakdowns
- Thread count display

### 💾 Memory & Swap
- Total, used, and available RAM
- Memory usage percentages
- Swap space utilization

### 💽 Storage Telemetry
- Mounted storage volumes & drives
- Real-time disk capacity & free space
- Filesystem type identification
- SSD / HDD drive type detection

### 🌐 Network Activity
- Live upload & download speeds
- Total bytes transmitted and received
- Per-interface network statistics

### ⚙️ Process Viewer
- Top running processes
- Real-time CPU and Memory allocation
- Auto-sorted by highest CPU consumption

### 📋 System Info
- Hostname, OS name, and kernel version
- System uptime & boot time
- Load averages (1m, 5m, 15m)

### 🎨 Clean UI & UX
- Native rendering with seamless performance
- Non-flicker launch optimization
- Dark mode interface with dynamic dashboard elements

---

# ⚡ Why Tauri & Rust?

System Monitor takes advantage of **Rust's concurrency and speed** combined with **Tauri's low overhead**:

- **Low RAM Usage:** Uses system native WebViews instead of bundling heavy browser engines.
- **Micro Binaries:** Small executable footprint.
- **High Security & Safety:** Safe memory management powered by Rust.
- **Cross-Platform:** Native builds tailored for Linux, macOS, and Windows.

---

# 📦 Supported Platforms

| Platform | Status | Packages Available |
| :--- | :---: | :--- |
| **Windows x64** | ✅ | `.exe`, `.msi` |
| **Linux x64** | ✅ | `.AppImage`, `.deb`, `.rpm` |
| **macOS Apple Silicon** | ✅ | `.dmg`, `.app.tar.gz` |

---

# 📥 Manual Installation

Prefer manual downloads? You can grab the latest installers directly from the [Releases Page](https://github.com/mahan07dev/System-Monitor/releases).

* **Windows:** `.exe` setup or `.msi` package
* **Linux:** `.deb` (Debian/Ubuntu), `.rpm` (Fedora/RHEL), or portable `.AppImage`
* **macOS:** `.dmg` disk image or `.app.tar.gz` archive

---

# 🛠️ Building From Source

### Prerequisites
- [Rust & Cargo](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) (v18+)
- npm

### Instructions

1. **Clone the repository:**
   ```bash
   git clone [https://github.com/mahan07dev/System-Monitor.git](https://github.com/mahan07dev/System-Monitor.git)
   cd System-Monitor
   ```

2. **Install frontend dependencies:**
   ```bash
   npm install
   ```

3. **Run in development mode:**
   ```bash
   npm run tauri dev
   ```

4. **Build release executable:**
   ```bash
   npm run tauri build
   ```

---

# 🏗️ Tech Stack

- **Backend:** Rust, `sysinfo`, `tauri` (v2)
- **Frontend:** HTML5, CSS3, JavaScript
- **Framework:** Tauri v2 Native WebView

---

# 📈 Project Roadmap

### ✅ Completed
- [x] Real-time CPU, RAM, & Swap monitoring
- [x] CPU Temperature readings
- [x] Disk storage detection & speed parsing
- [x] Network speed tracking
- [x] Top process inspector
- [x] Cross-platform build pipelines
- [x] One-line terminal installer scripts

### 🔮 Planned
- [ ] GPU Usage & Temperature tracking
- [ ] Battery telemetry for laptops
- [ ] Historical telemetry graphing
- [ ] Customizable updates & polling interval settings
- [ ] Desktop tray integration

---

# 🤝 Website

The official project landing page is maintained separately.

🌐 **Website:** [Kasrarasa-coder System Monitor Page](https://kasrarasa-coder.github.io/System-Monitor)

*Developed and maintained by **[@kasrarasa-coder](https://github.com/kasrarasa-coder)**.*

---

# 👨‍💻 Author

Developed with ❤️ by **[Mahan07dev](https://github.com/mahan07dev)**.

---

# ❤️ Acknowledgements

Special thanks to the **Rust**, **Tauri**, and open-source communities for building and supporting the tools that make this project possible.

---

# 📄 License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for details.