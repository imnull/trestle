//! 路由规则页面

use eframe::egui::{self, Color32, RichText};

#[derive(Debug, Clone, Default)]
pub struct RoutesPage;

impl RoutesPage {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("路由规则").size(24.0).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("➕ 添加规则").clicked() {}
                });
            });
            ui.add_space(20.0);

            // 表头
            egui::Grid::new("routes_header")
                .num_columns(5)
                .spacing([20.0, 10.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("优先级").strong());
                    ui.label(RichText::new("匹配规则").strong());
                    ui.label(RichText::new("服务商").strong());
                    ui.label(RichText::new("目标模型").strong());
                    ui.label(RichText::new("操作").strong());
                    ui.end_row();
                });

            ui.add_space(10.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                // 示例数据
                let sample_routes = [
                    (1, "gpt-4*", "OpenAI", "gpt-4-turbo"),
                    (1, "claude-*", "Anthropic", "claude-3-5-sonnet"),
                    (2, "local-*", "Ollama", "(原样)"),
                    (99, "*", "OpenAI", "gpt-4o (默认)"),
                ];

                for (priority, pattern, provider, model) in sample_routes {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(40, 40, 40))
                        .rounding(3.0)
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            egui::Grid::new(format!("route_{}", priority))
                                .num_columns(5)
                                .spacing([20.0, 5.0])
                                .show(ui, |ui| {
                                    ui.label(priority.to_string());
                                    ui.label(RichText::new(pattern).code());
                                    ui.label(provider);
                                    ui.label(model);
                                    ui.horizontal(|ui| {
                                        if ui.small_button("编辑").clicked() {}
                                        if ui.small_button("删除").clicked() {}
                                    });
                                    ui.end_row();
                                });
                        });
                    ui.add_space(5.0);
                }
            });
        });
    }
}
