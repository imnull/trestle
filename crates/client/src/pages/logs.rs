//! 日志页面

use eframe::egui::{self, Color32, RichText};

#[derive(Debug, Clone, Default)]
pub struct LogsPage {
    search: String,
}

impl LogsPage {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("请求日志").size(24.0).strong());
            });
            ui.add_space(20.0);

            // 搜索栏
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.search).hint_text("搜索..."));
                ui.button("导出 CSV");
            });

            ui.add_space(15.0);

            // 日志表格
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

                // 示例日志
                let sample_logs = [
                    ("14:32:01", "POST", "/v1/chat/completions", "gpt-4", "234ms", 200),
                    ("14:31:58", "POST", "/v1/chat/completions", "claude-3", "189ms", 200),
                    ("14:31:55", "POST", "/v1/messages", "claude-3", "312ms", 200),
                    ("14:31:52", "POST", "/v1/chat/completions", "gpt-4", "0ms", 429),
                ];

                for (time, method, path, model, latency, status) in sample_logs {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(35, 35, 35))
                        .rounding(2.0)
                        .inner_margin(5.0)
                        .show(ui, |ui| {
                            egui::Grid::new(format!("log_{}", time))
                                .num_columns(6)
                                .spacing([15.0, 3.0])
                                .show(ui, |ui| {
                                    ui.label(RichText::new(time).size(11.0).color(Color32::GRAY));
                                    ui.label(RichText::new(method).size(11.0));
                                    ui.label(RichText::new(path).size(11.0).color(Color32::GRAY));
                                    ui.label(RichText::new(model).size(11.0).code());
                                    ui.label(RichText::new(latency).size(11.0));
                                    ui.label(
                                        RichText::new(status.to_string())
                                            .size(11.0)
                                            .color(if status == 200 { Color32::from_rgb(0, 200, 100) } else { Color32::from_rgb(255, 100, 100) })
                                    );
                                    ui.end_row();
                                });
                        });
                    ui.add_space(3.0);
                }
            });
        });
    }
}
