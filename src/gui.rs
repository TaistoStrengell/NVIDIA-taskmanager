use eframe::egui;
use std::sync::mpsc::{Receiver, Sender};
use crate::nvml_ops::GpuStats;
use crate::AppCommand;

/// Represents a process using the GPU, including metadata and state.
pub struct FullProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cmdline: String,
    pub used_mem_mb: u64,
    /// True if the process is reported by NVML but not found in `/proc`.
    pub is_ghost: bool,
}

/// Payload sent from the background worker to the UI thread.
pub struct GpuUpdate {
    pub pci_status: String,
    pub pci_control: String,
    pub gpu_stats: Option<GpuStats>,
    pub processes: Vec<FullProcessInfo>,
}

/// Main application state for the egui interface.
pub struct TaskManagerApp {
    receiver: Receiver<GpuUpdate>,
    cmd_sender: Sender<AppCommand>,
    current_data: GpuUpdate,
}

impl TaskManagerApp {
    /// Initializes the app with communication channels to the background worker.
    pub fn new(_cc: &eframe::CreationContext<'_>, rx: Receiver<GpuUpdate>, tx: Sender<AppCommand>) -> Self {
        Self {
            receiver: rx,
            cmd_sender: tx,
            current_data: GpuUpdate {
                pci_status: "Init...".into(),
                pci_control: "Init...".into(),
                gpu_stats: None,
                processes: Vec::new(),
            },
        }
    }
}

impl eframe::App for TaskManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Drain all pending updates to get the latest state.
        while let Ok(new_data) = self.receiver.try_recv() {
            self.current_data = new_data;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("PCI: {}", self.current_data.pci_status)).strong());
                ui.separator();
                ui.label(format!("Control: {}", self.current_data.pci_control));
                if ui.small_button("Auto").clicked() {
                    let _ = self.cmd_sender.send(AppCommand::SetPowerMode("auto".into()));
                }
                if ui.small_button("On").clicked() {
                    let _ = self.cmd_sender.send(AppCommand::SetPowerMode("on".into()));
                }
            });

            ui.add_space(4.0);

            if let Some(stats) = &self.current_data.gpu_stats {
                ui.group(|ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(format!("{}°C", stats.temp_celsius));
                        ui.separator();
                        ui.label(format!("{}W", stats.power_usage_watts));
                        ui.separator();
                        ui.label(format!("GPU: {}% | Mem: {}%", stats.utilization_gpu, stats.utilization_mem));
                        ui.separator();
                        ui.label(format!("P-State: {}", stats.performance_state));
                    });
                    ui.label(format!("VRAM: {} / {} MB", stats.vram_used_mb, stats.vram_total_mb));
                });
            } else {
                ui.colored_label(egui::Color32::GRAY, "GPU is sleeping...");
            }
            
            ui.add_space(10.0);

            
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("proc_grid")
                    .num_columns(5)
                    .spacing([20.0, 8.0])
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("NAME").strong());
                        ui.label(egui::RichText::new("PID").strong());
                        ui.label(egui::RichText::new("VRAM").strong());
                        ui.label(egui::RichText::new("COMMAND").strong());
                        ui.label(egui::RichText::new("ACTION").strong());
                        ui.end_row();

                        for proc in &self.current_data.processes {
                            let row_color = if proc.is_ghost {
                                egui::Color32::from_rgb(255, 100, 100)
                            } else {
                                ui.visuals().text_color()
                            };

                            ui.colored_label(row_color, &proc.name);
                            ui.colored_label(row_color, proc.pid.to_string());
                            ui.colored_label(row_color, format!("{} MB", proc.used_mem_mb));
                            
                            
                            let cmd_text = egui::RichText::new(&proc.cmdline).size(11.0).color(row_color);
                            let cmd_ui = ui.add(egui::Label::new(cmd_text).truncate(true));
                            cmd_ui.on_hover_text(&proc.cmdline);

                            if ui.button("Kill").clicked() {
                                let _ = self.cmd_sender.send(AppCommand::KillProcess(proc.pid));
                            }
                            ui.end_row();
                        }
                    });
            });
        });

        // Request a repaint to ensure the UI updates even if no events occur.
        ctx.request_repaint_after(std::time::Duration::from_millis(500));
    }
}