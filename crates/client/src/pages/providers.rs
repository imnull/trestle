//! 服务商管理页面

use eframe::egui::{self, Color32, RichText};
use crate::api::ApiClient;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Default)]
pub struct ProvidersPage {
    providers: Arc<Mutex<Vec<crate::api::Provider>>>,
    loaded: bool,
    dialog_open: bool,
    editing_provider: Option<String>, // None = 添加, Some(name) = 编辑
    form: ProviderForm,
    error_message: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct ProviderForm {
    name: String,
    provider_type: String,
    base_url: String,
    api_key: String,
    enabled: bool,
}

impl ProvidersPage {
    pub fn show(&mut self, ui: &mut egui::Ui, api: &ApiClient) {
        if !self.loaded {
            self.load_providers(api);
            self.loaded = true;
        }

        ui.vertical(|ui| {
            // 标题栏
            ui.horizontal(|ui| {
                ui.label(RichText::new("[*] 服务商").size(24.0).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("添加").clicked() {
                        self.open_add_dialog();
                    }
                    if ui.button("刷新").clicked() {
                        self.loaded = false;
                    }
                });
            });
            ui.add_space(20.0);

            // 错误提示
            if let Some(ref err) = self.error_message {
                ui.label(RichText::new(format!("错误: {}", err)).color(Color32::RED));
                ui.add_space(10.0);
            }

            // 服务商列表
            let providers = self.providers.lock().unwrap().clone();
            egui::ScrollArea::vertical().show(ui, |ui| {
                for provider in &providers {
                    self.provider_card(ui, api, provider);
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

        // 对话框
        if self.dialog_open {
            self.show_dialog(ui, api);
        }
    }

    fn open_add_dialog(&mut self) {
        self.editing_provider = None;
        self.form = ProviderForm::default();
        self.form.provider_type = "openai".to_string();
        self.form.enabled = true;
        self.dialog_open = true;
        self.error_message = None;
    }

    fn open_edit_dialog(&mut self, provider: &crate::api::Provider) {
        self.editing_provider = Some(provider.name.clone());
        self.form = ProviderForm {
            name: provider.name.clone(),
            provider_type: provider.provider_type.clone(),
            base_url: provider.base_url.clone(),
            api_key: String::new(), // 不显示已保存的 API key
            enabled: provider.enabled,
        };
        self.dialog_open = true;
        self.error_message = None;
    }

    fn load_providers(&mut self, api: &ApiClient) {
        match api.get_providers() {
            Ok(data) => {
                *self.providers.lock().unwrap() = data;
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!("加载失败: {}", e));
            }
        }
    }

    fn save_provider(&mut self, api: &ApiClient) {
        let provider = crate::api::Provider {
            name: self.form.name.clone(),
            provider_type: self.form.provider_type.clone(),
            base_url: self.form.base_url.clone(),
            enabled: self.form.enabled,
        };

        let result: Result<(), anyhow::Error> = (|| {
            if let Some(ref original_name) = self.editing_provider {
                // 更新
                if original_name != &provider.name {
                    // 名称变更：先创建新，再删除旧
                    api.create_provider(&provider)?;
                    api.delete_provider(original_name)?;
                } else {
                    api.update_provider(original_name, &provider)?;
                }
            } else {
                // 新建
                api.create_provider(&provider)?;
            }
            Ok(())
        })();

        match result {
            Ok(_) => {
                self.dialog_open = false;
                self.loaded = false; // 刷新列表
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!("保存失败: {}", e));
            }
        }
    }

    fn delete_provider(&mut self, api: &ApiClient, name: &str) {
        match api.delete_provider(name) {
            Ok(_) => {
                self.loaded = false;
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!("删除失败: {}", e));
            }
        }
    }

    fn provider_card(&mut self,
        ui: &mut egui::Ui,
        api: &ApiClient,
        provider: &crate::api::Provider,
    ) {
        egui::Frame::none()
            .fill(Color32::from_rgb(40, 40, 40))
            .rounding(5.0)
            .inner_margin(15.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&provider.name).size(16.0).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("删除").clicked() {
                            self.delete_provider(api, &provider.name);
                        }
                        if ui.small_button("编辑").clicked() {
                            self.open_edit_dialog(provider);
                        }
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

    fn show_dialog(&mut self,
        ui: &mut egui::Ui,
        api: &ApiClient,
    ) {
        let title = if self.editing_provider.is_some() {
            "编辑服务商"
        } else {
            "添加服务商"
        };

        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.set_min_width(400.0);

                if let Some(ref err) = self.error_message {
                    ui.label(RichText::new(err).color(Color32::RED));
                    ui.add_space(10.0);
                }

                egui::Grid::new("provider_form")
                    .num_columns(2)
                    .spacing([10.0, 10.0])
                    .show(ui, |ui| {
                        ui.label("名称:");
                        ui.text_edit_singleline(&mut self.form.name);
                        ui.end_row();

                        ui.label("类型:");
                        egui::ComboBox::from_id_salt("provider_type")
                            .selected_text(match self.form.provider_type.as_str() {
                                "openai" => "OpenAI",
                                "anthropic" => "Anthropic",
                                "openai-compatible" => "OpenAI 兼容",
                                _ => &self.form.provider_type,
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.form.provider_type, "openai".to_string(), "OpenAI");
                                ui.selectable_value(&mut self.form.provider_type, "anthropic".to_string(), "Anthropic");
                                ui.selectable_value(&mut self.form.provider_type, "openai-compatible".to_string(), "OpenAI 兼容");
                            });
                        ui.end_row();

                        ui.label("Base URL:");
                        ui.text_edit_singleline(&mut self.form.base_url);
                        ui.end_row();

                        ui.label("启用:");
                        ui.checkbox(&mut self.form.enabled, "");
                        ui.end_row();
                    });

                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    if ui.button("取消").clicked() {
                        self.dialog_open = false;
                        self.error_message = None;
                    }
                    if ui.button("保存").clicked() {
                        if self.form.name.is_empty() {
                            self.error_message = Some("名称不能为空".to_string());
                        } else if self.form.base_url.is_empty() {
                            self.error_message = Some("Base URL 不能为空".to_string());
                        } else {
                            self.save_provider(api);
                        }
                    }
                });
            });
    }
}
