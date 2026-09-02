#![allow(dead_code)]

pub mod wayland;
pub mod x11;

use std::env;

pub fn is_x11() -> bool {
    let session = env::var("XDG_SESSION_TYPE").unwrap_or_default().to_lowercase();
    if session == "x11" {
        return true;
    }
    let display = env::var("DISPLAY").unwrap_or_default();
    let wayland = env::var("WAYLAND_DISPLAY").unwrap_or_default();
    !display.is_empty() && wayland.is_empty()
}

pub fn is_wayland() -> bool {
    wayland::is_wayland_session()
}

pub fn init_linux_platform() {
    log::info!("Initializing Linux Gaming Display platform subsystems...");
    if is_wayland() {
        wayland::setup_wayland_env();
    }
}
