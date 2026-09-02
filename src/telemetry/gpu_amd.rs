use std::fs;
use std::path::PathBuf;
use crate::telemetry::gpu_nvml::GpuStats;

pub struct AmdGpuCollector {
    card_path: Option<PathBuf>,
    hwmon_path: Option<PathBuf>,
}

impl AmdGpuCollector {
    pub fn new(gpu_index: u32) -> Self {
        let card_path = PathBuf::from(format!("/sys/class/drm/card{}", gpu_index));
        let hwmon_path = card_path.join("device/hwmon/hwmon0");
        
        let valid = card_path.exists();
        Self {
            card_path: if valid { Some(card_path) } else { None },
            hwmon_path: if hwmon_path.exists() { Some(hwmon_path) } else { None },
        }
    }

    pub fn is_available(&self) -> bool {
        self.card_path.is_some()
    }

    fn read_sysfs_u32(&self, base: &Option<PathBuf>, filename: &str) -> Option<u32> {
        let p = base.as_ref()?.join(filename);
        let s = fs::read_to_string(p).ok()?;
        s.trim().parse::<u32>().ok()
    }

    fn read_sysfs_f32(&self, base: &Option<PathBuf>, filename: &str) -> Option<f32> {
        let p = base.as_ref()?.join(filename);
        let s = fs::read_to_string(p).ok()?;
        s.trim().parse::<f32>().ok()
    }

    pub fn poll(&self) -> Option<GpuStats> {
        let card = self.card_path.as_ref()?;
        let dev = card.join("device");

        let util = self.read_sysfs_f32(&Some(dev.clone()), "gpu_busy_percent").unwrap_or(0.0);
        let clock_hz = self.read_sysfs_u32(&Some(dev.clone()), "current_gfxclk").unwrap_or(0);
        let clock_mhz = clock_hz / 1_000_000;

        let temp_raw = self.read_sysfs_f32(&self.hwmon_path, "temp1_input").unwrap_or(0.0);
        let temp = temp_raw / 1000.0;

        let power_raw = self.read_sysfs_f32(&self.hwmon_path, "power1_average").unwrap_or(0.0);
        let power_w = power_raw / 1_000_000.0;

        let vram_used_bytes = self.read_sysfs_f32(&Some(dev.clone()), "mem_info_vram_used").unwrap_or(0.0);
        let vram_total_bytes = self.read_sysfs_f32(&Some(dev.clone()), "mem_info_vram_total").unwrap_or(0.0);
        let vram_used_gb = vram_used_bytes / (1024.0 * 1024.0 * 1024.0);
        let vram_total_gb = vram_total_bytes / (1024.0 * 1024.0 * 1024.0);
        let vram_percent = if vram_total_gb > 0.0 { (vram_used_gb / vram_total_gb) * 100.0 } else { 0.0 };

        let fan_speed = self.read_sysfs_u32(&self.hwmon_path, "pwm1").map(|p| ((p as f32 / 255.0) * 100.0) as u32).unwrap_or(0);

        Some(GpuStats {
            name: "AMD Radeon GPU".to_string(),
            util,
            mem_util: 0.0,
            encoder_util: 0.0,
            clock_mhz,
            mem_clock_mhz: 0,
            temp,
            power_w,
            fan_speed,
            vram_used_gb,
            vram_total_gb,
            vram_percent,
        })
    }
}
