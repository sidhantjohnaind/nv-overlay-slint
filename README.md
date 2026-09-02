# NV-Overlay (Slint Edition)

A high-performance, lightweight NVIDIA GeForce-style performance overlay written in **Rust** and powered by **[Slint UI](https://slint.dev/)**. Designed with a **Zero-GPU footprint** by utilizing pure CPU software rendering.

---

## ⚡ Highlights

- **Zero-GPU Rendering**: Uses Slint's `winit-software` backend to avoid taking rendering resources or GPU cycles away from your games.
- **100% Real Hardware Metrics**:
  - **GPU**: Real-time stats via NVIDIA Management Library (NVML) — utilization, clock speeds (Core/Memory), temperatures, power draw (W), fan speeds, NVENC encoder usage, bus memory controller util, and VRAM breakdown.
  - **CPU & RAM**: Precise per-core/package metrics from kernel and sysinfo.
  - **In-Game FPS & Latency**: Seamless integration with MangoHud pipeline for real-time framerate and render latency tracking.
- **4 Customizable Display Presets**:
  1. **Basic**: FPS | GPU% | CPU% | Render Latency
  2. **Advanced**: Complete telemetry dashboard (Clocks, Temps, Power, Fans, VRAM)
  3. **Bandwidth & Streaming**: GPU% | VRAM | Bus Controller% | NVENC% | CPU% | RAM
  4. **Minimal FPS**: Compact single-stat framerate counter
- **X11 / Wayland Support**: Borderless, transparent, click-through, and docked always-on-top.
- **Single-Instance IPC Daemon**: Toggle overlay visibility or cycle presets in real-time via IPC commands or global keybindings.

---

## 🚀 Getting Started

### Prerequisites

- **Rust & Cargo** (1.75+ recommended): [rustup.rs](https://rustup.rs/)
- **NVIDIA GPU & Drivers** (NVML support)
- *Optional (for in-game FPS)*: [MangoHud](https://github.com/flightlessmango/MangoHud)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/sidhantjohnaind/nv-overlay-slint.git
cd nv-overlay-slint

# Build optimized release binary
cargo build --release
```

The compiled binary will be located at `target/release/nv-overlay-slint`.

---

## 🎮 Usage

### Launch Daemon
Run the overlay in the background:
```bash
./target/release/nv-overlay-slint
```

### IPC Controls & Shortcuts
You can control the running overlay using CLI flags (ideal for binding to window manager or system shortcuts):

- **Toggle Visibility** (e.g. `Alt + R`):
  ```bash
  nv-overlay-slint --toggle
  ```
- **Cycle Layout Presets** (e.g. `Alt + Shift + R`):
  ```bash
  nv-overlay-slint --cycle
  ```
- **Quit Background Daemon**:
  ```bash
  nv-overlay-slint --quit
  ```

---

## 📐 Architecture & Technology Stack

- **GUI / Layout Engine**: [Slint 1.9](https://slint.dev/) (Software Renderer)
- **Telemetry Collection**:
  - `nvml-wrapper` (Dynamic NVML binding)
  - `sysinfo` (CPU, memory, system resources)
- **IPC**: Unix domain sockets / lightweight inter-process messaging
- **CLI**: `clap` 4.5

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
