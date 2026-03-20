//! 日志页面

use eframe::egui::{self, Color32, RichText};
use crate::api::ApiClient;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Default)]
pub struct LogsPage {
    logs: Arc<Mutex<Vec<crate::api::RequestLog>>>,
    search: String,
    loaded: bool,
}

impl LogsPage {
    pub fn show(&mut self, ui: &mut egui::Ui, api: &ApiClient) {
        if !self.loaded {
            self.load_logs(api);
            self.loaded = true;
        }

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("请求日志").size(24.0).strong());
            });
            ui.add_space(20.0);

            // 搜索栏
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.search).hint_text("搜索..."));
                let _ = ui.button("导出 CSV");
                if ui.button("⟳ 刷新").clicked() {
                    self.loaded = false;
                }
            });

            ui.add_space(15.0);

            // 日志表格
            let logs = self.logs.lock().unwrap().clone();
            egui::ScrollArea::vertical().show(ui, |ui| {
                // 表头
                egui::Grid::new("logs_header")
                    .num_columns(6)
                    .spacing([15.0, 5.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("时间").strong().size(11.0));
                        ui.label(RichText::new("方法").strong().size(11.0));
                        ui.label(RichText::new("端点").strong().size(11.0));
                        ui.label(RichText::new("模型").strong().size(11.0));
                        ui.label(RichText::new("延迟").strong().size(11.0));
                        ui.label(RichText::new("状态").strong().size(11.0));
                        ui.end_row();
                    });

                ui.add_space(5.0);

                if logs.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new("暂无请求日志").color(Color32::GRAY));
                            ui.add_space(10.0);
                            ui.label(RichText::new("发送请求后将显示在这里").color(Color32::GRAY));
                        });
                    });
                } else {
                    for log in &logs {
                        log_row(ui, log);
                        ui.add_space(3.0);
                    }
                }
            });
        });
    }

    fn load_logs(&mut self, api: &ApiClient) {
        match api.get::<Vec<crate::api::RequestLog>>("/api/logs") {
            Ok(data) => {
                *self.logs.lock().unwrap() = data;
            }
            Err(e) => {
                eprintln!("Failed to load logs: {}", e);
            }
        }
    }
}

fn log_row(ui: &mut egui::Ui, log: &crate::api::RequestLog) {
    egui::Frame::none()
        .fill(Color32::from_rgb(35, 35, 35))
        .rounding(2.0)
        .inner_margin(5.0)
        .show(ui, |ui| {
            egui::Grid::new(&log.id)
                .num_columns(6)
                .spacing([15.0, 3.0])
                .show(ui, |ui| {
                    ui.label(RichText::new(&log.timestamp).size(11.0).color(Color32::GRAY));
                    ui.label(RichText::new(&log.method).size(11.0));
                    ui.label(RichText::new(&log.path).size(11.0).color(Color32::GRAY));
                    ui.label(RichText::new(&log.model).size(11.0).code());
                    ui.label(RichText::new(format!("{}ms", log.latency_ms)).size(11.0));
                    ui.label(
                        RichText::new(log.status.to_string())
                            .size(11.0)
                            .color(if log.status == 200 { 
                                Color32::from_rgb(0, 200, 100) 
                            } else { 
                                Color32::from_rgb(255, 100, 100) 
                            })
                    );
                    ui.end_row();
                });
        });
}
