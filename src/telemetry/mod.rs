#![allow(dead_code)]

pub mod cpu_ram;
pub mod fps_mangohud;
pub mod gpu_amd;
pub mod gpu_nvml;
pub mod gpu_smi;
pub mod metrics;
pub mod simulated;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender};
use parking_lot::RwLock;

use crate::config::AppConfig;
use crate::telemetry::cpu_ram::CpuRamCollector;
use crate::telemetry::fps_mangohud::MangoHudFpsReader;
use crate::telemetry::gpu_nvml::NvmlCollector;
use crate::telemetry::metrics::HardwareMetrics;

pub struct TelemetryEngine {
    latest_metrics: Arc<RwLock<HardwareMetrics>>,
    running: Arc<AtomicBool>,
}

impl TelemetryEngine {
    pub fn new(config: &AppConfig) -> (Self, Receiver<HardwareMetrics>) {
        let latest_metrics = Arc::new(RwLock::new(HardwareMetrics::default()));
        let running = Arc::new(AtomicBool::new(true));
        let (tx, rx) = crossbeam_channel::bounded::<HardwareMetrics>(4);

        let latest_clone = Arc::clone(&latest_metrics);
        let running_clone = Arc::clone(&running);
        let config_clone = config.clone();

        thread::spawn(move || {
            Self::worker_loop(config_clone, latest_clone, running_clone, tx);
        });

        (
            Self {
                latest_metrics,
                running,
            },
            rx,
        )
    }

    pub fn get_latest(&self) -> HardwareMetrics {
        self.latest_metrics.read().clone()
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    fn worker_loop(
        config: AppConfig,
        latest_lock: Arc<RwLock<HardwareMetrics>>,
        running: Arc<AtomicBool>,
        tx: Sender<HardwareMetrics>,
    ) {
        let mut cpu_ram = CpuRamCollector::new();
        let nvml = NvmlCollector::new(config.gpu_index);
        let mangohud = MangoHudFpsReader::new(config.enable_mangohud_pipe);

        let start_time = Instant::now();

        while running.load(Ordering::Relaxed) {
            let loop_start = Instant::now();

            let mut m = latest_lock.read().clone();
            m.timestamp = start_time.elapsed().as_secs_f64();

            // 1. GPU Stats (100% Real from NVML)
            if let Some(gpu) = nvml.poll() {
                m.gpu_name = gpu.name;
                m.gpu_util = gpu.util;
                m.gpu_mem_util = gpu.mem_util;
                m.gpu_encoder_util = gpu.encoder_util;
                m.gpu_clock = gpu.clock_mhz;
                m.gpu_mem_clock = gpu.mem_clock_mhz;
                m.gpu_temp = gpu.temp;
                m.gpu_power = gpu.power_w;
                m.gpu_fan_speed = gpu.fan_speed;
                m.vram_used = gpu.vram_used_gb;
                m.vram_total = gpu.vram_total_gb;
                m.vram_percent = gpu.vram_percent;
            }

            // 2. CPU & RAM Stats (100% Real from Kernel / Sysfs)
            let cpu = cpu_ram.poll();
            m.cpu_name = cpu.cpu_name;
            m.cpu_util = cpu.cpu_util;
            m.cpu_freq = cpu.cpu_freq_ghz;
            m.cpu_temp = cpu.cpu_temp;
            m.ram_used = cpu.ram_used_gb;
            m.ram_total = cpu.ram_total_gb;
            m.ram_percent = cpu.ram_percent;

            // 3. FPS / MangoHud frame rate (Real in games)
            if let Some((fps, lat)) = mangohud.poll() {
                m.fps = fps;
                m.render_latency = lat;
            } else {
                m.fps = 0.0;
                m.render_latency = 0.0;
            }

            // Update shared lock
            *latest_lock.write() = m.clone();
            let _ = tx.try_send(m);

            // Sleep until next 1000ms cycle
            let elapsed = loop_start.elapsed();
            let target_interval = Duration::from_millis(config.refresh_interval_ms.max(500));
            if elapsed < target_interval {
                thread::sleep(target_interval - elapsed);
            }
        }
    }
}
