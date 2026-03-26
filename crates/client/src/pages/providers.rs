//! 服务商管理页面 - 现代化设计

use eframe::egui::{self, Color32, RichText};
use crate::api::ApiClient;
use crate::ui_theme::{self, colors, spacing, icons, card_frame, primary_button, secondary_button, danger_button};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Default)]
pub struct ProvidersPage {
    providers: Arc<Mutex<Vec<crate::api::Provider>>>,
    loaded: bool,
    dialog_open: bool,
    editing_provider: Option<String>,
    form: ProviderForm,
    error_message: Option<String>,
    show_delete_confirm: Option<String>,
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
            // 工具栏
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if primary_button(ui, &format!("{} 添加", icons::ADD)).clicked() {
                        self.open_add_dialog();
                    }
                    ui.add_space(spacing::SM);
                    if secondary_button(ui, &format!("{} 刷新", icons::REFRESH)).clicked() {
                        self.loaded = false;
                    }
                });
            });
            
            ui.add_space(spacing::MD);

            // 错误提示
            if let Some(ref err) = self.error_message {
                card_frame().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(icons::ERROR)
                                .size(18.0)
                        );
                        ui.add_space(spacing::SM);
                        ui.label(
                            RichText::new(err)
                                .size(13.0)
                                .color(colors::ERROR)
                        );
                    });
                });
                ui.add_space(spacing::MD);
            }

            // 服务商列表
            let providers = self.providers.lock().unwrap().clone();
            
            if providers.is_empty() {
                ui_theme::empty_state(
                    ui,
                    icons::PROVIDERS,
                    "暂无服务商",
                    "点击右上角「添加」按钮创建第一个服务商配置"
                );
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for provider in &providers {
                        self.provider_card(ui, api, provider);
                        ui.add_space(spacing::MD);
                    }
                });
            }
        });

        // 对话框
        if self.dialog_open {
            self.show_dialog(ui, api);
        }
        
        // 删除确认对话框
        if let Some(ref name) = self.show_delete_confirm {
            self.show_delete_confirm(ui, api, name.clone());
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
            api_key: String::new(),
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
                if original_name != &provider.name {
                    api.create_provider(&provider)?;
                    api.delete_provider(original_name)?;
                } else {
                    api.update_provider(original_name, &provider)?;
                }
            } else {
                api.create_provider(&provider)?;
            }
            Ok(())
        })();

        match result {
            Ok(_) => {
                self.dialog_open = false;
                self.loaded = false;
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

    fn provider_card(&mut self, ui: &mut egui::Ui, api: &ApiClient, provider: &crate::api::Provider) {
        let type_icon = match provider.provider_type.as_str() {
            "openai" => "🤖",
            "anthropic" => "🧠",
            _ => "🔧",
        };
        
        card_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                // 类型图标
                ui.label(
                    RichText::new(type_icon)
                        .size(28.0)
                );
                
                ui.add_space(spacing::MD);
                
                // 服务商信息
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(&provider.name)
                            .size(16.0)
                            .strong()
                            .color(colors::TEXT_PRIMARY)
                    );
                    ui.add_space(spacing::XS);
                    ui.label(
                        RichText::new(&provider.base_url)
                            .size(12.0)
                            .color(colors::TEXT_MUTED)
                    );
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // 操作按钮
                    if danger_button(ui, &format!("{} 删除", icons::DELETE)).clicked() {
                        self.show_delete_confirm = Some(provider.name.clone());
                    }
                    ui.add_space(spacing::SM);
                    
                    if secondary_button(ui, &format!("{} 编辑", icons::EDIT)).clicked() {
                        self.open_edit_dialog(provider);
                    }
                    ui.add_space(spacing::SM);
                    
                    // 状态标签
                    if provider.enabled {
                        ui_theme::badge(ui, "已启用", colors::SUCCESS);
                    } else {
                        ui_theme::badge(ui, "已禁用", colors::TEXT_MUTED);
                    }
                    ui.add_space(spacing::SM);
                    
                    ui_theme::badge(
                        ui, 
                        match provider.provider_type.as_str() {
                            "openai" => "OpenAI",
                            "anthropic" => "Anthropic",
                            _ => &provider.provider_type,
                        },
                        colors::PRIMARY
                    );
                });
            });
        });
    }

    fn show_dialog(&mut self, ui: &mut egui::Ui, api: &ApiClient) {
        let title = if self.editing_provider.is_some() {
            format!("{} 编辑服务商", icons::EDIT)
        } else {
            format!("{} 添加服务商", icons::ADD)
        };

        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .fixed_size([500.0, 400.0])
            .show(ui.ctx(), |ui| {
                ui.set_min_width(480.0);

                if let Some(ref err) = self.error_message {
                    card_frame().show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(icons::ERROR)
                                    .size(16.0)
                            );
                            ui.add_space(spacing::SM);
                            ui.label(
                                RichText::new(err)
                                    .color(colors::ERROR)
                                    .size(13.0)
                            );
                        });
                    });
                    ui.add_space(spacing::MD);
                }

                ui.vertical(|ui| {
                    // 名称
                    ui.label(
                        RichText::new("名称 *")
                            .size(13.0)
                            .strong()
                            .color(colors::TEXT_PRIMARY)
                    );
                    ui.add_space(spacing::XS);
                    ui.add(
                        egui::TextEdit::singleline(&mut self.form.name)
                            .hint_text("例如: OpenAI、Claude 等")
                            .desired_width(400.0)
                    );
                    ui.add_space(spacing::MD);
                    
                    // 类型
                    ui.label(
                        RichText::new("类型 *")
                            .size(13.0)
                            .strong()
                            .color(colors::TEXT_PRIMARY)
                    );
                    ui.add_space(spacing::XS);
                    egui::ComboBox::from_id_salt("provider_type")
                        .selected_text(
                            match self.form.provider_type.as_str() {
                                "openai" => "🤖 OpenAI",
                                "anthropic" => "🧠 Anthropic",
                                _ => "🔧 OpenAI 兼容",
                            }
                        )
                        .width(400.0)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.form.provider_type, "openai".to_string(), "🤖 OpenAI");
                            ui.selectable_value(&mut self.form.provider_type, "anthropic".to_string(), "🧠 Anthropic");
                            ui.selectable_value(&mut self.form.provider_type, "openai-compatible".to_string(), "🔧 OpenAI 兼容");
                        });
                    ui.add_space(spacing::MD);
                    
                    // Base URL
                    ui.label(
                        RichText::new("Base URL *")
                            .size(13.0)
                            .strong()
                            .color(colors::TEXT_PRIMARY)
                    );
                    ui.add_space(spacing::XS);
                    ui.add(
                        egui::TextEdit::singleline(&mut self.form.base_url)
                            .hint_text("https://api.openai.com/v1")
                            .desired_width(400.0)
                    );
                    ui.add_space(spacing::MD);
                    
                    // 启用状态
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.form.enabled, "");
                        ui.label(
                            RichText::new("启用此服务商")
                                .size(13.0)
                                .color(colors::TEXT_PRIMARY)
                        );
                    });
                });

                ui.add_space(spacing::XL);
                
                ui.horizontal_centered(|ui| {
                    if secondary_button(ui, &format!("{} 取消", icons::CANCEL)).clicked() {
                        self.dialog_open = false;
                        self.error_message = None;
                    }
                    ui.add_space(spacing::MD);
                    if primary_button(ui, &format!("{} 保存", icons::SAVE)).clicked() {
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
    
    fn show_delete_confirm(&mut self, ui: &mut egui::Ui, api: &ApiClient, name: String) {
        egui::Window::new(format!("{} 确认删除", icons::WARNING))
            .collapsible(false)
            .resizable(false)
            .fixed_size([350.0, 200.0])
            .show(ui.ctx(), |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(spacing::LG);
                    
                    ui.label(
                        RichText::new(icons::WARNING)
                            .size(32.0)
                    );
                    
                    ui.add_space(spacing::MD);
                    
                    ui.label(
                        RichText::new(format!("确定要删除服务商「{}」吗？", name))
                            .size(14.0)
                            .strong()
                            .color(colors::TEXT_PRIMARY)
                    );
                    
                    ui.add_space(spacing::SM);
                    
                    ui.label(
                        RichText::new("此操作不可撤销")
                            .size(12.0)
                            .color(colors::TEXT_MUTED)
                    );
                    
                    ui.add_space(spacing::XL);
                    
                    ui.horizontal(|ui| {
                        if secondary_button(ui, "取消").clicked() {
                            self.show_delete_confirm = None;
                        }
                        ui.add_space(spacing::MD);
                        if danger_button(ui, &format!("{} 删除", icons::DELETE)).clicked() {
                            self.delete_provider(api, &name);
                            self.show_delete_confirm = None;
                        }
                    });
                });
            });
    }
}
