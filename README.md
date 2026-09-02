# NV-Overlay (Slint Edition)

A high-performance, lightweight NVIDIA GeForce-style performance overlay written in **Rust** and powered by **[Slint UI](https://slint.dev/)**. Designed with a **Zero-GPU footprint** by utilizing pure CPU software rendering.

---

## ⚡ Highlights

- **Zero-GPU Rendering**: Uses Slint's `winit-software` backend to avoid consuming GPU rendering resources or frame cycles from your games.
- **100% Real Hardware Metrics**:
  - **GPU**: Real-time stats via NVIDIA Management Library (NVML) — utilization, core & memory clock speeds, temperatures, board power draw (W), fan speeds, NVENC encoder load, bus controller utilization, and VRAM breakdown.
  - **CPU & RAM**: Precise per-core/package metrics from kernel sysfs and `sysinfo`.
  - **In-Game FPS & Latency**: Seamless integration with MangoHud pipeline for real-time framerate and frame time tracking.
- **4 Customizable Display Presets**:
  1. **Basic**: `FPS | GPU% | CPU% | Render Latency`
  2. **Advanced**: Full hardware dashboard (`Clocks, Temps, Power, Fans, VRAM, RAM`)
  3. **Bandwidth & Streaming**: `GPU% | VRAM | Bus Controller% | NVENC% | CPU% | RAM`
  4. **Minimal FPS**: Compact single-stat framerate counter
- **Multi-Platform & Multi-Architecture**: Supports **AMD64 / x86_64**, **ARM64 / AArch64**, **ARMv7**, and **RISC-V 64-bit** across Linux and Windows.
- **Background Daemon & System Services**: Easily run as a **Systemd User Service**, XDG autostart, or Windows startup task.
- **Single-Instance IPC Daemon**: Toggle visibility or cycle presets instantaneously via CLI commands or global keybindings.

---

## 🖥️ Supported Architectures & Platforms

| Architecture | Platform | Target Triple | Release Package |
| :--- | :--- | :--- | :--- |
| **AMD / Intel (x86_64)** | Linux (GNU) | `x86_64-unknown-linux-gnu` | `*.tar.gz` |
| **AMD / Intel (x86_64)** | Linux (Musl) | `x86_64-unknown-linux-musl` | `*.tar.gz` |
| **AMD / Intel (x86_64)** | Windows | `x86_64-pc-windows-msvc` | `*.zip` |
| **ARM (ARM64 / AArch64)** | Linux (GNU) | `aarch64-unknown-linux-gnu` | `*.tar.gz` |
| **ARM (ARM64 / AArch64)** | Linux (Musl) | `aarch64-unknown-linux-musl` | `*.tar.gz` |
| **ARM (ARMv7 32-bit)** | Linux (GNU) | `armv7-unknown-linux-gnueabihf` | `*.tar.gz` |
| **ARM (ARM64)** | Windows | `aarch64-pc-windows-msvc` | `*.zip` |
| **RISC-V (64-bit)** | Linux (GNU) | `riscv64gc-unknown-linux-gnu` | `*.tar.gz` |

---

## 📦 Installation & Services

### Method 1: Automatic Installer (Linux: AMD64 / ARM / RISC-V)

Clone or download the release archive, then run the installer:

```bash
chmod +x packaging/scripts/install.sh
./packaging/scripts/install.sh
```

This installs:
- Binary to `~/.local/bin/nv-overlay-slint`
- Systemd user service to `~/.config/systemd/user/nv-overlay.service`
- Desktop entry & autostart to `~/.config/autostart/` and `~/.local/share/applications/`

#### Enable Systemd User Service:
```bash
systemctl --user enable --now nv-overlay.service
```

#### Check Service Status / Logs:
```bash
systemctl --user status nv-overlay.service
journalctl --user -u nv-overlay.service -f
```

---

### Method 2: Windows Startup Service

Run PowerShell as user:
```powershell
powershell -ExecutionPolicy Bypass -File packaging/windows/install-startup.ps1
```

To remove from startup:
```powershell
powershell -ExecutionPolicy Bypass -File packaging/windows/uninstall-startup.ps1
```

---

## 🛠️ Build from Source

### Prerequisites
- **Rust & Cargo** (1.75+): [rustup.rs](https://rustup.rs/)
- **NVIDIA Drivers / NVML**

```bash
# Clone the repository
git clone https://github.com/sidhantjohnaind/nv-overlay-slint.git
cd nv-overlay-slint

# Build optimized release binary
cargo build --release
```

---

## 🎮 IPC Controls & Shortcuts

You can bind these commands to global keyboard shortcuts in your Desktop Environment (GNOME, KDE Plasma, Sway, Hyprland, Windows Hotkeys):

- **Toggle Overlay Visibility** (e.g. `Alt + R`):
  ```bash
  nv-overlay-slint --toggle
  ```
- **Cycle HUD Presets** (e.g. `Alt + Shift + R`):
  ```bash
  nv-overlay-slint --cycle
  ```
- **Quit Running Daemon**:
  ```bash
  nv-overlay-slint --quit
  ```

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
