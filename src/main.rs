mod config;
mod ipc;
mod platform;
mod telemetry;

use clap::Parser;
use config::AppConfig;
use slint::ComponentHandle;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Arc;
use telemetry::TelemetryEngine;

slint::include_modules!();

#[derive(Parser, Debug)]
#[command(name = "nv-overlay-slint")]
#[command(author = "Antigravity Team")]
#[command(version = "1.0.0")]
#[command(about = "NVIDIA Performance Overlay (Slint / Zero-GPU Edition)")]
struct CliArgs {
    /// Toggle overlay visibility (used by global hotkey Alt+R)
    #[arg(short, long)]
    toggle: bool,

    /// Cycle overlay layout presets (Basic -> Advanced -> Bandwidth/Stream -> FPS Only)
    #[arg(short, long)]
    cycle: bool,

    /// Terminate the running background daemon
    #[arg(short, long)]
    quit: bool,
}

fn get_preset_width(preset: i32) -> f32 {
    match preset {
        0 => 430.0, // Basic
        1 => 820.0, // Advanced
        2 => 660.0, // Bandwidth & Streaming
        3 => 160.0, // FPS Only
        _ => 430.0,
    }
}

fn main() -> Result<(), slint::PlatformError> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = CliArgs::parse();

    // 1. Handle CLI commands sent from global hotkeys
    if args.toggle {
        match ipc::send_ipc_command(ipc::IpcCommand::Toggle) {
            Ok(res) => println!("{}", res),
            Err(e) => eprintln!("Error sending toggle command to overlay: {}", e),
        }
        return Ok(());
    }

    if args.cycle {
        match ipc::send_ipc_command(ipc::IpcCommand::Cycle) {
            Ok(res) => println!("{}", res),
            Err(e) => eprintln!("Error sending cycle command to overlay: {}", e),
        }
        return Ok(());
    }

    if args.quit {
        match ipc::send_ipc_command(ipc::IpcCommand::Quit) {
            Ok(res) => println!("{}", res),
            Err(e) => eprintln!("Error sending quit command to overlay: {}", e),
        }
        return Ok(());
    }

    // 2. Single-Instance Check: If daemon is already running, do NOT launch a second one
    if let Ok(res) = ipc::send_ipc_command(ipc::IpcCommand::Ping) {
        if res.contains("PONG") {
            println!("NV-Overlay is already running in background (press Alt+R to toggle).");
            return Ok(());
        }
    }

    // Default to pure CPU software rasterizer (0.00% GPU) if not explicitly overridden
    if std::env::var_os("SLINT_BACKEND").is_none() {
        std::env::set_var("SLINT_BACKEND", "winit-software");
    }

    log::info!("Starting NV-Overlay (Slint / Zero-GPU Edition - 100% Real Hardware Metrics)...");

    platform::init_linux_platform();

    let config = AppConfig::default();
    let (telemetry_engine, _rx) = TelemetryEngine::new(&config);
    let engine_rc = Rc::new(telemetry_engine);

    // Create the Slint overlay window
    let window = OverlayWindow::new()?;
    let window_handle = window.as_weak();

    // Calculate initial position at top-right
    let (screen_w, _screen_h) = platform::x11::get_screen_resolution();
    let initial_width = get_preset_width(0);
    let pos_x = (screen_w - initial_width - 16.0).max(0.0) as i32;
    let pos_y = 48;
    window.window().set_position(slint::PhysicalPosition::new(pos_x, pos_y));

    // Shared state for IPC callbacks
    let visible_state = Arc::new(AtomicBool::new(true));
    let preset_state = Arc::new(AtomicI32::new(0));

    // 3. Start IPC Server to handle Alt+R toggle and Alt+Shift+R mode cycle
    {
        let win_weak = window.as_weak();
        let vis = Arc::clone(&visible_state);
        let pre = Arc::clone(&preset_state);

        ipc::start_ipc_server(move |cmd| match cmd {
            ipc::IpcCommand::Toggle => {
                let current = vis.load(Ordering::SeqCst);
                let new_val = !current;
                vis.store(new_val, Ordering::SeqCst);

                let win_clone = win_weak.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(w) = win_clone.upgrade() {
                        w.set_overlay_visible(new_val);
                    }
                });

                log::info!("IPC Toggle: Overlay visibility is now {}", new_val);
                format!("Toggled overlay to {}", new_val)
            }
            ipc::IpcCommand::Cycle => {
                let current = pre.load(Ordering::SeqCst);
                let next_preset = (current + 1) % 4; // Cycle 0 -> 1 -> 2 -> 3 -> 0
                pre.store(next_preset, Ordering::SeqCst);

                let win_clone = win_weak.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(w) = win_clone.upgrade() {
                        w.set_preset_mode(next_preset);
                        let width = get_preset_width(next_preset);
                        let (s_w, _) = platform::x11::get_screen_resolution();
                        let new_x = (s_w - width - 16.0).max(0.0) as i32;
                        w.window().set_position(slint::PhysicalPosition::new(new_x, 48));
                        #[cfg(target_family = "unix")]
                        platform::x11::enforce_always_on_top("NV-Overlay HUD", true, Some((new_x, 48)));
                    }
                });

                log::info!("IPC Cycle: Cycled preset to {}", next_preset);
                format!("Cycled preset to {}", next_preset)
            }
            ipc::IpcCommand::Quit => {
                log::info!("IPC Quit command received. Exiting.");
                std::process::exit(0);
            }
            ipc::IpcCommand::Settings => "Settings not implemented in Slint edition".to_string(),
            ipc::IpcCommand::Ping => "PONG".to_string(),
        });
    }

    // Periodic 1 FPS (1000ms) timer for zero GPU footprint
    let timer = slint::Timer::default();
    let engine_for_timer = Rc::clone(&engine_rc);

    timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(1000),
        move || {
            if let Some(w) = window_handle.upgrade() {
                let m = engine_for_timer.get_latest();

                let fps_str = if m.fps > 0.0 {
                    format!("{:.0}", m.fps)
                } else {
                    "N/A".to_string()
                };
                let lat_str = if m.render_latency > 0.0 {
                    format!("{:.1}", m.render_latency)
                } else {
                    "N/A".to_string()
                };

                let data = TelemetryData {
                    fps: fps_str.into(),
                    latency: lat_str.into(),
                    gpu_util: format!("{:.0}", m.gpu_util).into(),
                    gpu_mem_util: format!("{:.0}", m.gpu_mem_util).into(),
                    gpu_encoder_util: format!("{:.0}", m.gpu_encoder_util).into(),
                    gpu_temp: format!("{:.0}", m.gpu_temp).into(),
                    gpu_clock: format!("{}", m.gpu_clock).into(),
                    gpu_mem_clock: format!("{}", m.gpu_mem_clock).into(),
                    gpu_power: format!("{:.0}", m.gpu_power).into(),
                    gpu_fan: format!("{}", m.gpu_fan_speed).into(),
                    vram_used: format!("{:.1}", m.vram_used).into(),
                    cpu_util: format!("{:.0}", m.cpu_util).into(),
                    cpu_temp: format!("{:.0}", m.cpu_temp).into(),
                    ram_used: format!("{:.1}", m.ram_used).into(),
                };

                w.set_data(data);
            }
        },
    );

    // Apply X11 click-through and always-on-top docking after window is shown
    let win_handle_clone = window.as_weak();
    slint::Timer::single_shot(std::time::Duration::from_millis(250), move || {
        if let Some(_w) = win_handle_clone.upgrade() {
            #[cfg(target_family = "unix")]
            platform::x11::enforce_always_on_top("NV-Overlay HUD", true, Some((pos_x, pos_y)));
        }
    });

    window.run()
}
