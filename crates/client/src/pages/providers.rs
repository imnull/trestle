//! 服务商管理页面

use eframe::egui::{self, Color32, RichText};
use crate::api::ApiClient;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Default)]
pub struct ProvidersPage {
    providers: Arc<Mutex<Vec<crate::api::Provider>>>,
    loaded: bool,
    add_dialog_open: bool,
    new_provider: NewProvider,
}

#[derive(Debug, Clone, Default)]
struct NewProvider {
    name: String,
    provider_type: String,
    base_url: String,
    api_key: String,
}

impl ProvidersPage {
    pub fn show(&mut self, ui: &mut egui::Ui, api: &ApiClient) {
        // 首次加载
        if !self.loaded {
            self.load_providers(api);
            self.loaded = true;
        }

        ui.vertical(|ui| {
            // 标题栏
            ui.horizontal(|ui| {
                ui.label(RichText::new("服务商").size(24.0).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("➕ 添加服务商").clicked() {
                        self.add_dialog_open = true;
                    }
                    if ui.button("⟳ 刷新").clicked() {
                        self.loaded = false;
                    }
                });
            });
            ui.add_space(20.0);

            // 服务商列表
            let providers = self.providers.lock().unwrap().clone();
            egui::ScrollArea::vertical().show(ui, |ui| {
                for provider in &providers {
                    provider_card(ui, provider);
                    ui.add_space(10.0);
                }

                if providers.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new("暂无服务商").color(Color32::GRAY));
                            ui.add_space(10.0);
                            ui.label(RichText::new("点击上方按钮添加").color(Color32::GRAY));
                        });
                    });
                }
            });
        });

        // 添加对话框
        if self.add_dialog_open {
            self.show_add_dialog(ui);
        }
    }

    fn load_providers(&mut self, api: &ApiClient) {
        let providers = self.providers.clone();
        let api = api.clone();
        
        std::thread::spawn(move || {
            if let Ok(rt) = tokio::runtime::Runtime::new() {
                if let Ok(data) = rt.block_on(api.get::<Vec<crate::api::Provider>>("/api/providers")) {
                    *providers.lock().unwrap() = data;
                }
            }
        });
    }

    fn show_add_dialog(&mut self, ui: &mut egui::Ui) {
        egui::Window::new("添加服务商")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.set_min_width(400.0);

                egui::Grid::new("add_provider_form")
                    .num_columns(2)
                    .spacing([10.0, 10.0])
                    .show(ui, |ui| {
                        ui.label("名称:");
                        ui.text_edit_singleline(&mut self.new_provider.name);
                        ui.end_row();

                        ui.label("类型:");
                        egui::ComboBox::from_id_salt("provider_type")
                            .selected_text(&self.new_provider.provider_type)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.new_provider.provider_type, "openai".to_string(), "OpenAI");
                                ui.selectable_value(&mut self.new_provider.provider_type, "anthropic".to_string(), "Anthropic");
                                ui.selectable_value(&mut self.new_provider.provider_type, "openai-compatible".to_string(), "OpenAI 兼容");
                            });
                        ui.end_row();

                        ui.label("Base URL:");
                        ui.text_edit_singleline(&mut self.new_provider.base_url);
                        ui.end_row();

                        ui.label("API Key:");
                        ui.add(egui::TextEdit::singleline(&mut self.new_provider.api_key).password(true));
                        ui.end_row();
                    });

                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    if ui.button("取消").clicked() {
                        self.add_dialog_open = false;
                        self.new_provider = NewProvider::default();
                    }
                    if ui.button("保存").clicked() {
                        // TODO: 调用 API 保存
                        self.add_dialog_open = false;
                        self.new_provider = NewProvider::default();
                    }
                });
            });
    }
}

fn provider_card(ui: &mut egui::Ui, provider: &crate::api::Provider) {
    egui::Frame::none()
        .fill(Color32::from_rgb(40, 40, 40))
        .rounding(5.0)
        .inner_margin(15.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&provider.name).size(16.0).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("编辑").clicked() {}
                });
            });

            ui.add_space(5.0);
            ui.label(RichText::new(&provider.base_url).color(Color32::GRAY).size(12.0));

            ui.add_space(5.0);
            ui.horizontal(|ui| {
                if provider.enabled {
                    ui.label(RichText::new("● 启用").color(Color32::from_rgb(0, 200, 100)).size(12.0));
                } else {
                    ui.label(RichText::new("○ 已禁用").color(Color32::GRAY).size(12.0));
                }
                ui.label(RichText::new("|").color(Color32::GRAY));
                ui.label(RichText::new(&provider.provider_type).color(Color32::GRAY).size(12.0));
            });
        });
}
