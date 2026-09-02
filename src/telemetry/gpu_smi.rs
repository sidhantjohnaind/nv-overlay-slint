use std::process::Command;
use crate::telemetry::gpu_nvml::GpuStats;

pub struct NvidiaSmiCollector {
    gpu_index: u32,
}

impl NvidiaSmiCollector {
    pub fn new(gpu_index: u32) -> Self {
        Self { gpu_index }
    }

    pub fn poll(&self) -> Option<GpuStats> {
        let output = Command::new("nvidia-smi")
            .arg(format!("--id={}", self.gpu_index))
            .arg("--query-gpu=name,utilization.gpu,utilization.memory,clocks.current.graphics,clocks.current.memory,temperature.gpu,power.draw,fan.speed,memory.used,memory.total")
            .arg("--format=csv,noheader,nounits")
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let text = String::from_utf8_lossy(&output.stdout);
        let line = text.lines().next()?.trim();
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

        if parts.len() < 10 {
            return None;
        }

        let name = parts[0].to_string();
        let util: f32 = parts[1].parse().unwrap_or(0.0);
        let mem_util: f32 = parts[2].parse().unwrap_or(0.0);
        let clock_mhz: u32 = parts[3].parse().unwrap_or(0);
        let mem_clock_mhz: u32 = parts[4].parse().unwrap_or(0);
        let temp: f32 = parts[5].parse().unwrap_or(0.0);
        let power_w: f32 = parts[6].parse().unwrap_or(0.0);
        let fan_speed: u32 = parts[7].parse().unwrap_or(0);
        let mem_used_mb: f32 = parts[8].parse().unwrap_or(0.0);
        let mem_total_mb: f32 = parts[9].parse().unwrap_or(0.0);

        let vram_used_gb = mem_used_mb / 1024.0;
        let vram_total_gb = mem_total_mb / 1024.0;
        let vram_percent = if vram_total_gb > 0.0 {
            (vram_used_gb / vram_total_gb) * 100.0
        } else {
            0.0
        };

        Some(GpuStats {
            name,
            util,
            mem_util,
            encoder_util: 0.0,
            clock_mhz,
            mem_clock_mhz,
            temp,
            power_w,
            fan_speed,
            vram_used_gb,
            vram_total_gb,
            vram_percent,
        })
    }
}
