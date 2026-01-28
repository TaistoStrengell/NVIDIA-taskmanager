mod pci;
mod nvml_ops;
mod process_ops;
mod gui;

use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use pci::PciDevice;
use nvml_ops::NvmlHandler;

pub enum AppCommand {
    SetPowerMode(String),
    KillProcess(u32),
}
// Entry point for the NVIDIA Task Manager.
/// 
/// Spawns a background worker to poll GPU metrics every 800ms. To minimize power impact 
/// on hybrid systems, NVML is only initialized and queried when the dGPU's PCI 
/// runtime status is "active".
fn main() -> eframe::Result<()> {
    let (tx_update, rx_update) = mpsc::channel();
    let (tx_cmd, rx_cmd) = mpsc::channel();

    thread::spawn(move || {
        let mut nvml_handler: Option<NvmlHandler> = None;
        let mut process_cache: HashMap<u32, (String, String)> = HashMap::new();
        
        let pci_dev = match PciDevice::find_nvidia() {
            Ok(dev) => dev,
            Err(_) => return,
        };

        loop {
            while let Ok(cmd) = rx_cmd.try_recv() {
                match cmd {
                    AppCommand::SetPowerMode(mode) => {
                        if let Err(e) = pci_dev.set_runtime_control(&mode) {
                            eprintln!("Failed to set power mode: {}", e);
                        }
                    }
                    AppCommand::KillProcess(pid) => { let _ = process_ops::kill_process(pid, true); }
                }
            }

            let status = pci_dev.get_runtime_status();
            let control = pci_dev.get_runtime_control();
            let mut stats = None;
            let mut full_processes = Vec::new();

            if status == "active" {
                if nvml_handler.is_none() {
                    nvml_handler = NvmlHandler::new().ok();
                }

                if let Some(ref handler) = nvml_handler {
                    if let Ok(current_stats) = handler.get_stats() {
                        let current_pids: HashSet<u32> = current_stats.active_processes.iter().map(|p| p.pid).collect();
                        process_cache.retain(|pid, _| current_pids.contains(pid));

                        for gpu_proc in &current_stats.active_processes {
                            let (name, cmd) = process_cache
                                .entry(gpu_proc.pid)
                                .or_insert_with(|| (process_ops::get_process_name(gpu_proc.pid), process_ops::get_process_cmdline(gpu_proc.pid)))
                                .clone();

                            full_processes.push(gui::FullProcessInfo {
                                pid: gpu_proc.pid,
                                name,
                                cmdline: cmd,
                                used_mem_mb: gpu_proc.used_mem_bytes / 1024 / 1024,
                                is_ghost: gpu_proc.is_ghost,
                            });
                        }
                        stats = Some(current_stats);
                    }
                }
            } else {
                nvml_handler = None;
                process_cache.clear();
            }

            let _ = tx_update.send(gui::GpuUpdate {
                pci_status: status,
                pci_control: control,
                gpu_stats: stats,
                processes: full_processes,
            });

            thread::sleep(Duration::from_millis(800));
        }
    });

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "NVIDIA Task Manager",
        native_options,
        Box::new(|cc| Box::new(gui::TaskManagerApp::new(cc, rx_update, tx_cmd))),
    )
}