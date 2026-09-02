#[cfg(feature = "nvml")]
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
#[cfg(feature = "nvml")]
use nvml_wrapper::Nvml;

pub struct NvmlCollector {
    #[cfg(feature = "nvml")]
    nvml: Option<Nvml>,
    gpu_index: u32,
}

pub struct GpuStats {
    pub name: String,
    pub util: f32,
    pub mem_util: f32,      // Real Memory Controller Bus Load %
    pub encoder_util: f32,  // Real NVENC Video Encoder Load %
    pub clock_mhz: u32,
    pub mem_clock_mhz: u32,
    pub temp: f32,
    pub power_w: f32,
    pub fan_speed: u32,     // Real Fan Duty Cycle %
    pub vram_used_gb: f32,
    pub vram_total_gb: f32,
    pub vram_percent: f32,
}

impl NvmlCollector {
    pub fn new(gpu_index: u32) -> Self {
        #[cfg(feature = "nvml")]
        {
            let nvml = match Nvml::init() {
                Ok(n) => {
                    log::info!("NVML initialized successfully");
                    Some(n)
                }
                Err(e) => {
                    log::debug!("NVML initialization failed: {:?}", e);
                    None
                }
            };
            Self { nvml, gpu_index }
        }
        #[cfg(not(feature = "nvml"))]
        {
            Self { gpu_index }
        }
    }

    pub fn is_available(&self) -> bool {
        #[cfg(feature = "nvml")]
        {
            self.nvml.is_some()
        }
        #[cfg(not(feature = "nvml"))]
        {
            false
        }
    }

    pub fn poll(&self) -> Option<GpuStats> {
        #[cfg(feature = "nvml")]
        {
            let nvml = self.nvml.as_ref()?;
            let device = nvml.device_by_index(self.gpu_index).ok()?;

            let name = device.name().unwrap_or_else(|_| "NVIDIA GPU".to_string());
            
            // Real Core & Memory Bus Utilization
            let (util, mem_util) = if let Ok(rates) = device.utilization_rates() {
                (rates.gpu as f32, rates.memory as f32)
            } else {
                (0.0, 0.0)
            };

            // Real Hardware Video Encoder (NVENC) Load
            let encoder_util = device.encoder_utilization().map(|u| u.utilization as f32).unwrap_or(0.0);

            // Real Frequencies & Thermals
            let clock_mhz = device.clock_info(nvml_wrapper::enum_wrappers::device::Clock::Graphics).unwrap_or(0);
            let mem_clock_mhz = device.clock_info(nvml_wrapper::enum_wrappers::device::Clock::Memory).unwrap_or(0);
            let temp = device.temperature(TemperatureSensor::Gpu).map(|t| t as f32).unwrap_or(0.0);
            
            // Real Board Power
            let power_w = device.power_usage().map(|p| (p as f32) / 1000.0).unwrap_or(0.0);
            
            // Real Fan Controller Duty %
            let fan_speed = device.fan_speed(0).unwrap_or(0);

            // Real VRAM Allocation
            let (vram_used_gb, vram_total_gb, vram_percent) = if let Ok(mem) = device.memory_info() {
                let used = (mem.used as f32) / (1024.0 * 1024.0 * 1024.0);
                let total = (mem.total as f32) / (1024.0 * 1024.0 * 1024.0);
                let pct = if total > 0.0 { (used / total) * 100.0 } else { 0.0 };
                (used, total, pct)
            } else {
                (0.0, 0.0, 0.0)
            };

            Some(GpuStats {
                name,
                util,
                mem_util,
                encoder_util,
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
        #[cfg(not(feature = "nvml"))]
        {
            None
        }
    }
}
