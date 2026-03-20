//! 日志页面

use eframe::egui::{self, Color32, RichText};
use crate::api::ApiClient;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Default)]
pub struct LogsPage {
    logs: Arc<Mutex<Vec<crate::api::RequestLog>>>,
    search: String,
    loaded: bool,
    export_status: Option<String>,
}

impl LogsPage {
    pub fn show(&mut self, ui: &mut egui::Ui, api: &ApiClient) {
        if !self.loaded {
            self.load_logs(api);
            self.loaded = true;
        }

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("[#] 请求日志").size(24.0).strong());
            });
            ui.add_space(20.0);

            // 搜索栏和操作按钮
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.search).hint_text("搜索模型/路径..."));
                
                if ui.button("刷新").clicked() {
                    self.loaded = false;
                    self.export_status = None;
                }
                
                if ui.button("导出 CSV").clicked() {
                    self.export_csv();
                }
            });

            if let Some(ref status) = self.export_status {
                ui.add_space(5.0);
                ui.label(RichText::new(status).color(Color32::from_rgb(0, 200, 100)));
            }

            ui.add_space(15.0);

            // 日志表格
            let logs = self.logs.lock().unwrap().clone();
            let filtered_logs: Vec<_> = if self.search.is_empty() {
                logs.into_iter().collect()
            } else {
                logs.into_iter()
                    .filter(|log| {
                        log.model.to_lowercase().contains(&self.search.to_lowercase()) ||
                        log.path.to_lowercase().contains(&self.search.to_lowercase())
                    })
                    .collect()
            };

            // 统计
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("共 {} 条记录", filtered_logs.len())).color(Color32::GRAY));
            });
            ui.add_space(10.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                // 表头
                egui::Frame::none()
                    .fill(Color32::from_rgb(35, 35, 35))
                    .rounding(2.0)
                    .inner_margin(5.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.set_min_width(ui.available_width());
                            ui.label(RichText::new("时间").strong().size(11.0));
                            ui.label(RichText::new("方法").strong().size(11.0));
                            ui.label(RichText::new("端点").strong().size(11.0));
                            ui.label(RichText::new("模型").strong().size(11.0));
                            ui.label(RichText::new("延迟").strong().size(11.0));
                            ui.label(RichText::new("状态").strong().size(11.0));
                        });
                    });

                ui.add_space(5.0);

                if filtered_logs.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new("暂无请求日志").color(Color32::GRAY));
                            ui.add_space(10.0);
                            ui.label(RichText::new("发送请求后将显示在这里").color(Color32::GRAY));
                        });
                    });
                } else {
                    for log in &filtered_logs {
                        log_row(ui, log);
                        ui.add_space(3.0);
                    }
                }
            });
        });
    }

    fn load_logs(&mut self, api: &ApiClient) {
        match api.get_logs() {
            Ok(data) => {
                *self.logs.lock().unwrap() = data;
            }
            Err(e) => {
                eprintln!("Failed to load logs: {}", e);
            }
        }
    }

    fn export_csv(&mut self) {
        let logs = self.logs.lock().unwrap().clone();
        
        if logs.is_empty() {
            self.export_status = Some("没有日志可导出".to_string());
            return;
        }

        // 生成 CSV 内容
        let mut csv = String::from("时间,方法,端点,模型,延迟(ms),状态码\n");
        for log in &logs {
            csv.push_str(&format!(
                "{},{},{},{},{},{}\n",
                log.timestamp,
                log.method,
                log.path,
                log.model,
                log.latency_ms,
                log.status
            ));
        }

        // 保存到文件
        let export_path = std::env::temp_dir().join("trestle_logs_export.csv");
        match std::fs::write(&export_path, csv) {
            Ok(_) => {
                self.export_status = Some(format!(
                    "已导出到: {}",
                    export_path.to_string_lossy()
                ));
            }
            Err(e) => {
                self.export_status = Some(format!("导出失败: {}", e));
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
            ui.horizontal(|ui| {
                ui.set_min_width(ui.available_width());
                
                ui.label(RichText::new(&log.timestamp).size(11.0).color(Color32::GRAY));
                ui.label(RichText::new(&log.method).size(11.0));
                ui.label(RichText::new(&log.path).size(11.0).color(Color32::GRAY));
                ui.label(RichText::new(&log.model).size(11.0).code());
                ui.label(RichText::new(format!("{}ms", log.latency_ms)).size(11.0));
                
                let status_color = if log.status == 200 {
                    Color32::from_rgb(0, 200, 100)
                } else if log.status >= 400 {
                    Color32::from_rgb(255, 100, 100)
                } else {
                    Color32::from_rgb(255, 200, 0)
                };
                ui.label(RichText::new(log.status.to_string()).size(11.0).color(status_color));
            });
        });
}
