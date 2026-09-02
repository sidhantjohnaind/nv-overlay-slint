#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub enabled: bool,
    pub preset: i32, // 0: Basic, 1: Advanced, 2: DLSS, 3: FPS Only
    pub refresh_interval_ms: u64,
    pub gpu_backend: String,
    pub gpu_index: u32,
    pub enable_mangohud_pipe: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            preset: 0,
            refresh_interval_ms: 1000,
            gpu_backend: "auto".to_string(),
            gpu_index: 0,
            enable_mangohud_pipe: true,
        }
    }
}
