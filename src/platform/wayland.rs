use std::env;

/// Checks if running under a Linux Wayland compositor (KDE Plasma Wayland, GNOME Wayland, Hyprland, Sway, Gamescope).
pub fn is_wayland_session() -> bool {
    let session_type = env::var("XDG_SESSION_TYPE").unwrap_or_default().to_lowercase();
    session_type == "wayland" || env::var("WAYLAND_DISPLAY").is_ok()
}

/// Optimizes environment variables for Linux Wayland and Gamescope compatibility.
pub fn setup_wayland_env() {
    if is_wayland_session() {
        log::info!("Linux Wayland session detected. Enabling translucent overlay compatibility.");
        if env::var("WINIT_UNIX_BACKEND").is_err() {
            // Prefer wayland with x11 fallback
            env::set_var("WINIT_UNIX_BACKEND", "wayland,x11");
        }
    }
}
