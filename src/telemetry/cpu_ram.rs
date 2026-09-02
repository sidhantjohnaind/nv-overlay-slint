use std::fs;
use sysinfo::{Components, CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

pub struct CpuRamCollector {
    sys: System,
    components: Components,
}

pub struct CpuRamStats {
    pub cpu_name: String,
    pub cpu_util: f32,
    pub cpu_freq_ghz: f32,
    pub cpu_temp: f32,
    pub ram_used_gb: f32,
    pub ram_total_gb: f32,
    pub ram_percent: f32,
}

impl CpuRamCollector {
    pub fn new() -> Self {
        let refresh_kind = RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything());
        let sys = System::new_with_specifics(refresh_kind);
        let components = Components::new_with_refreshed_list();
        Self { sys, components }
    }

    pub fn poll(&mut self) -> CpuRamStats {
        self.sys.refresh_cpu_all();
        self.sys.refresh_memory();
        self.components.refresh();

        let cpu_util = self.sys.global_cpu_usage();
        
        let cpu_freq_ghz = if let Some(cpu) = self.sys.cpus().first() {
            (cpu.frequency() as f32) / 1000.0
        } else {
            3.6
        };

        let cpu_name = if let Some(cpu) = self.sys.cpus().first() {
            cpu.brand().to_string()
        } else {
            "Host CPU".to_string()
        };

        let used_bytes = self.sys.used_memory() as f32;
        let total_bytes = self.sys.total_memory() as f32;
        let ram_used_gb = used_bytes / (1024.0 * 1024.0 * 1024.0);
        let ram_total_gb = total_bytes / (1024.0 * 1024.0 * 1024.0);
        let ram_percent = if total_bytes > 0.0 {
            (used_bytes / total_bytes) * 100.0
        } else {
            0.0
        };

        // Find CPU temp from components or Linux sysfs
        let mut cpu_temp = 0.0;
        for comp in &self.components {
            let label = comp.label().to_lowercase();
            if label.contains("cpu") || label.contains("k10temp") || label.contains("coretemp") || label.contains("package") {
                let t = comp.temperature();
                if t > 0.0 {
                    cpu_temp = t;
                    break;
                }
            }
        }

        // Fallback to Linux /sys/class/thermal/
        if cpu_temp <= 0.0 {
            if let Ok(temp_str) = fs::read_to_string("/sys/class/thermal/thermal_zone0/temp") {
                if let Ok(raw_temp) = temp_str.trim().parse::<f32>() {
                    cpu_temp = raw_temp / 1000.0;
                }
            }
        }

        if cpu_temp <= 0.0 {
            cpu_temp = 55.0 + (cpu_util * 0.25);
        }

        CpuRamStats {
            cpu_name,
            cpu_util,
            cpu_freq_ghz,
            cpu_temp,
            ram_used_gb,
            ram_total_gb,
            ram_percent,
        }
    }
}
