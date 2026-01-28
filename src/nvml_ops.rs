use nvml_wrapper::Nvml;
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use nvml_wrapper::enums::device::UsedGpuMemory;
use std::path::Path;

pub struct GpuProcess {
    pub pid: u32,
    pub used_mem_bytes: u64,
    pub is_ghost: bool,
}

pub struct GpuStats {
    pub temp_celsius: u32,
    pub vram_used_mb: u64,
    pub vram_total_mb: u64,
    pub utilization_gpu: u32,
    pub utilization_mem: u32,
    pub power_usage_watts: f32,
    pub performance_state: String,
    pub active_processes: Vec<GpuProcess>,
}

pub struct NvmlHandler {
    nvml: Nvml,
}

impl NvmlHandler {
    /// Initializes a new NVML handler.
    pub fn new() -> Result<Self, String> {
        Nvml::init()
            .map(|nvml| Self { nvml })
            .map_err(|e| e.to_string())
    }

    /// Retrieves all relevant GPU statistics.
    pub fn get_stats(&self) -> Result<GpuStats, String> {
        let device = self.nvml.device_by_index(0).map_err(|e| e.to_string())?;

        let mem_info = device.memory_info().map_err(|e| e.to_string())?;
        let util = device.utilization_rates().map_err(|e| e.to_string())?;
        let temp = device.temperature(TemperatureSensor::Gpu).unwrap_or(0);
        let power_mw = device.power_usage().unwrap_or(0);
        let p_state = device.performance_state()
            .map(|s| format!("{:?}", s))
            .unwrap_or_else(|_| "Unknown".to_string());

        let mut processes = Vec::new();

        let mut scan_procs = |raw_procs: Vec<nvml_wrapper::struct_wrappers::device::ProcessInfo>| {
            for proc in raw_procs {
                let is_alive = Path::new(&format!("/proc/{}", proc.pid)).exists();
                let used_mem_bytes = match proc.used_gpu_memory {
                    UsedGpuMemory::Used(bytes) => bytes,
                    UsedGpuMemory::Unavailable => 0,
                };

                processes.push(GpuProcess {
                    pid: proc.pid,
                    used_mem_bytes,
                    is_ghost: !is_alive,
                });
            }
        };

        if let Ok(cp) = device.running_compute_processes() { scan_procs(cp); }
        if let Ok(gp) = device.running_graphics_processes() { scan_procs(gp); }

        Ok(GpuStats {
            temp_celsius: temp,
            vram_used_mb: mem_info.used / 1024 / 1024,
            vram_total_mb: mem_info.total / 1024 / 1024,
            utilization_gpu: util.gpu,
            utilization_mem: util.memory,
            power_usage_watts: power_mw as f32 / 1000.0,
            performance_state: p_state,
            active_processes: processes,
        })
    }
}