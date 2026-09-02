#![allow(dead_code)]

use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct MetricHistory {
    pub values: VecDeque<f32>,
    pub max_samples: usize,
}

impl MetricHistory {
    pub fn new(max_samples: usize) -> Self {
        Self {
            values: VecDeque::with_capacity(max_samples),
            max_samples,
        }
    }

    pub fn push(&mut self, val: f32) {
        if self.values.len() >= self.max_samples {
            self.values.pop_front();
        }
        self.values.push_back(val);
    }
}

#[derive(Debug, Clone)]
pub struct HardwareMetrics {
    // Game Frame Timing (from MangoHud socket or 0 if desktop)
    pub fps: f32,
    pub render_latency: f32, // ms

    // Real GPU Hardware Metrics (NVML Direct)
    pub gpu_name: String,
    pub gpu_util: f32,      // Core load %
    pub gpu_mem_util: f32,  // Memory controller bus load %
    pub gpu_encoder_util: f32, // NVENC video encoder load %
    pub gpu_clock: u32,     // Graphics core MHz
    pub gpu_mem_clock: u32, // VRAM clock MHz
    pub gpu_temp: f32,      // Die temp °C
    pub gpu_power: f32,     // Power draw Watts
    pub gpu_fan_speed: u32, // Fan duty %
    pub vram_used: f32,     // VRAM used GB
    pub vram_total: f32,    // Total VRAM GB
    pub vram_percent: f32,  // VRAM %

    // Real CPU Telemetry (Kernel & Hwmon)
    pub cpu_name: String,
    pub cpu_util: f32,      // %
    pub cpu_freq: f32,      // GHz
    pub cpu_temp: f32,      // Package temp °C

    // Real RAM Telemetry (Kernel /proc/meminfo)
    pub ram_used: f32,      // GB
    pub ram_total: f32,     // GB
    pub ram_percent: f32,   // %

    pub timestamp: f64,
}

impl Default for HardwareMetrics {
    fn default() -> Self {
        Self {
            fps: 0.0,
            render_latency: 0.0,

            gpu_name: "NVIDIA GPU".to_string(),
            gpu_util: 0.0,
            gpu_mem_util: 0.0,
            gpu_encoder_util: 0.0,
            gpu_clock: 0,
            gpu_mem_clock: 0,
            gpu_temp: 0.0,
            gpu_power: 0.0,
            gpu_fan_speed: 0,
            vram_used: 0.0,
            vram_total: 0.0,
            vram_percent: 0.0,

            cpu_name: "Host CPU".to_string(),
            cpu_util: 0.0,
            cpu_freq: 0.0,
            cpu_temp: 0.0,

            ram_used: 0.0,
            ram_total: 0.0,
            ram_percent: 0.0,

            timestamp: 0.0,
        }
    }
}
