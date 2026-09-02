use std::time::Instant;
use rand::Rng;
use crate::telemetry::gpu_nvml::GpuStats;

pub struct SimulatedCollector {
    start_time: Instant,
}

impl SimulatedCollector {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }

    pub fn poll(&self) -> (GpuStats, f32, f32) {
        let t = self.start_time.elapsed().as_secs_f32();
        let mut rng = rand::thread_rng();

        let hz = crate::platform::x11::get_monitor_refresh_rate();
        let base_fps = hz + rng.gen_range(-0.4..0.4);
        let latency = (1000.0 / base_fps.max(1.0)) * 1.05;

        let gpu_util = (97.0 + 3.0 * (t * 0.5).sin() + rng.gen_range(-1.0..1.0)).clamp(0.0, 100.0);
        let gpu_temp = 66.0 + 3.5 * (t * 0.3).sin();
        let power_w = 275.0 + 25.0 * (t * 0.4).sin();
        let clock_mhz = (2565.0 + 35.0 * (t * 0.6).sin()) as u32;

        let stats = GpuStats {
            name: "NVIDIA GeForce RTX 4080 SUPER".to_string(),
            util: gpu_util,
            mem_util: 72.0,
            encoder_util: 0.0,
            clock_mhz,
            mem_clock_mhz: 11200,
            temp: gpu_temp,
            power_w,
            fan_speed: 56,
            vram_used_gb: 9.8,
            vram_total_gb: 16.0,
            vram_percent: 61.25,
        };

        (stats, base_fps, latency)
    }
}
