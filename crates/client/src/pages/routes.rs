//! 路由规则页面 - 现代化设计

use eframe::egui::{self, Color32, RichText};
use crate::api::{ApiClient, Provider};
use crate::ui_theme::{self, colors, spacing, icons, card_frame, primary_button, secondary_button, danger_button};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Default)]
pub struct RoutesPage {
    routes: Arc<Mutex<Vec<crate::api::Route>>>,
    providers: Arc<Mutex<Vec<Provider>>>,
    loaded: bool,
    dialog_open: bool,
    editing_pattern: Option<String>,
    form: RouteForm,
    error_message: Option<String>,
    show_delete_confirm: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct RouteForm {
    pattern: String,
    provider: String,
    model: String,
    priority: u32,
}

impl RoutesPage {
    pub fn show(&mut self, ui: &mut egui::Ui, api: &ApiClient) {
        if !self.loaded {
            self.load_data(api);
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
            
            ui.add_space(spacing::SM);
            
            // 提示信息
            card_frame().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(icons::INFO)
                            .size(14.0)
                    );
                    ui.add_space(spacing::SM);
                    ui.label(
                        RichText::new("使用 * 作为通配符，例如 gpt-4* 匹配所有 gpt-4 开头的模型")
                            .size(11.0)
                            .color(colors::TEXT_SECONDARY)
                    );
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

            // 路由列表
            let routes = self.routes.lock().unwrap().clone();
            
            if routes.is_empty() {
                ui_theme::empty_state(
                    ui,
                    icons::ROUTES,
                    "暂无路由规则",
                    "点击右上角「添加」按钮创建第一个路由规则"
                );
            } else {
                // 表头
                card_frame().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.set_min_width(ui.available_width());
                        
                        ui.allocate_ui_with_layout(
                            egui::Vec2::new(60.0, 20.0),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                ui.label(RichText::new("优先级").strong().size(12.0).color(colors::TEXT_SECONDARY));
                            }
                        );
                        
                        ui.allocate_ui_with_layout(
                            egui::Vec2::new(150.0, 20.0),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                ui.label(RichText::new("匹配规则").strong().size(12.0).color(colors::TEXT_SECONDARY));
                            }
                        );
                        
                        ui.allocate_ui_with_layout(
                            egui::Vec2::new(120.0, 20.0),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                ui.label(RichText::new("服务商").strong().size(12.0).color(colors::TEXT_SECONDARY));
                            }
                        );
                        
                        ui.allocate_ui_with_layout(
                            egui::Vec2::new(150.0, 20.0),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                ui.label(RichText::new("目标模型").strong().size(12.0).color(colors::TEXT_SECONDARY));
                            }
                        );
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new("操作").strong().size(12.0).color(colors::TEXT_SECONDARY));
                        });
                    });
                });

                ui.add_space(spacing::SM);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for route in &routes {
                        self.route_row(ui, api, route);
                        ui.add_space(spacing::SM);
                    }
                });
            }
        });

        if self.dialog_open {
            self.show_dialog(ui, api);
        }
        
        if let Some(ref pattern) = self.show_delete_confirm {
            self.show_delete_confirm_dialog(ui, api, pattern.clone());
        }
    }

    fn open_add_dialog(&mut self) {
        self.editing_pattern = None;
        self.form = RouteForm::default();
        self.form.priority = 10;
        let providers = self.providers.lock().unwrap();
        if let Some(first) = providers.first() {
            self.form.provider = first.name.clone();
        }
        self.dialog_open = true;
        self.error_message = None;
    }

    fn open_edit_dialog(&mut self, route: &crate::api::Route) {
        self.editing_pattern = Some(route.pattern.clone());
        self.form = RouteForm {
            pattern: route.pattern.clone(),
            provider: route.provider.clone(),
            model: route.model.clone().unwrap_or_default(),
            priority: route.priority,
        };
        self.dialog_open = true;
        self.error_message = None;
    }

    fn load_data(&mut self, api: &ApiClient) {
        match api.get_routes() {
            Ok(data) => {
                *self.routes.lock().unwrap() = data;
            }
            Err(e) => {
                self.error_message = Some(format!("加载路由失败: {}", e));
            }
        }
        match api.get_providers() {
            Ok(data) => {
                *self.providers.lock().unwrap() = data;
            }
            Err(_) => {}
        }
    }

    fn save_route(&mut self, api: &ApiClient) {
        let route = crate::api::Route {
            pattern: self.form.pattern.clone(),
            provider: self.form.provider.clone(),
            model: if self.form.model.is_empty() { None } else { Some(self.form.model.clone()) },
            priority: self.form.priority,
        };

        let result: Result<(), anyhow::Error> = (|| {
            if let Some(ref original_pattern) = self.editing_pattern {
                api.update_route(original_pattern, &route)?;
            } else {
                api.create_route(&route)?;
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

    fn delete_route(&mut self, api: &ApiClient, pattern: &str) {
        match api.delete_route(pattern) {
            Ok(_) => {
                self.loaded = false;
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!("删除失败: {}", e));
            }
        }
    }

    fn route_row(&mut self, ui: &mut egui::Ui, api: &ApiClient, route: &crate::api::Route) {
        card_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.set_min_width(ui.available_width());
                
                ui.allocate_ui_with_layout(
                    egui::Vec2::new(60.0, 20.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui_theme::badge(ui, &route.priority.to_string(), colors::PRIMARY);
                    }
                );
                
                ui.allocate_ui_with_layout(
                    egui::Vec2::new(150.0, 20.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.label(
                            RichText::new(&route.pattern)
                                .size(13.0)
                                .code()
                                .color(colors::TEXT_PRIMARY)
                        );
                    }
                );
                
                ui.allocate_ui_with_layout(
                    egui::Vec2::new(120.0, 20.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.label(
                            RichText::new(&route.provider)
                                .size(13.0)
                                .color(colors::TEXT_PRIMARY)
                        );
                    }
                );
                
                ui.allocate_ui_with_layout(
                    egui::Vec2::new(150.0, 20.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        let model_text = route.model.as_deref().unwrap_or("(原样)");
                        ui.label(
                            RichText::new(model_text)
                                .size(13.0)
                                .color(if route.model.is_none() { colors::TEXT_MUTED } else { colors::TEXT_PRIMARY })
                        );
                    }
                );
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if danger_button(ui, &format!("{} 删除", icons::DELETE)).clicked() {
                        self.show_delete_confirm = Some(route.pattern.clone());
                    }
                    ui.add_space(spacing::SM);
                    if secondary_button(ui, &format!("{} 编辑", icons::EDIT)).clicked() {
                        self.open_edit_dialog(route);
                    }
                });
            });
        });
    }

    fn show_dialog(&mut self, ui: &mut egui::Ui, _api: &ApiClient) {
        let title = if self.editing_pattern.is_some() {
            format!("{} 编辑路由", icons::EDIT)
        } else {
            format!("{} 添加路由", icons::ADD)
        };

        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .fixed_size([500.0, 420.0])
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
                    // 匹配规则
                    ui.label(
                        RichText::new("匹配规则 *")
                            .size(13.0)
                            .strong()
                            .color(colors::TEXT_PRIMARY)
                    );
                    ui.add_space(spacing::XS);
                    ui.add(
                        egui::TextEdit::singleline(&mut self.form.pattern)
                            .hint_text("例如: gpt-4*、claude-*")
                            .desired_width(400.0)
                    );
                    ui.add_space(spacing::SM);
                    ui.label(
                        RichText::new("使用 * 作为通配符匹配任意字符")
                            .size(11.0)
                            .color(colors::TEXT_MUTED)
                    );
                    ui.add_space(spacing::MD);
                    
                    // 服务商
                    ui.label(
                        RichText::new("目标服务商 *")
                            .size(13.0)
                            .strong()
                            .color(colors::TEXT_PRIMARY)
                    );
                    ui.add_space(spacing::XS);
                    let providers = self.providers.lock().unwrap();
                    egui::ComboBox::from_id_salt("provider_select")
                        .selected_text(&self.form.provider)
                        .width(400.0)
                        .show_ui(ui, |ui| {
                            for p in providers.iter() {
                                ui.selectable_value(&mut self.form.provider, p.name.clone(), &p.name);
                            }
                        });
                    ui.add_space(spacing::MD);
                    
                    // 目标模型
                    ui.label(
                        RichText::new("目标模型")
                            .size(13.0)
                            .strong()
                            .color(colors::TEXT_PRIMARY)
                    );
                    ui.add_space(spacing::XS);
                    ui.add(
                        egui::TextEdit::singleline(&mut self.form.model)
                            .hint_text("留空表示使用请求中的原始模型名")
                            .desired_width(400.0)
                    );
                    ui.add_space(spacing::MD);
                    
                    // 优先级
                    ui.label(
                        RichText::new("优先级 *")
                            .size(13.0)
                            .strong()
                            .color(colors::TEXT_PRIMARY)
                    );
                    ui.add_space(spacing::XS);
                    ui.add(
                        egui::Slider::new(&mut self.form.priority, 1..=100)
                            .text("数值越小优先级越高")
                    );
                });

                ui.add_space(spacing::XL);
                
                ui.horizontal_centered(|ui| {
                    if secondary_button(ui, &format!("{} 取消", icons::CANCEL)).clicked() {
                        self.dialog_open = false;
                        self.error_message = None;
                    }
                    ui.add_space(spacing::MD);
                    if primary_button(ui, &format!("{} 保存", icons::SAVE)).clicked() {
                        if self.form.pattern.is_empty() {
                            self.error_message = Some("匹配规则不能为空".to_string());
                        } else if self.form.provider.is_empty() {
                            self.error_message = Some("请选择服务商".to_string());
                        } else {
                            self.save_route(_api);
                        }
                    }
                });
            });
    }
    
    fn show_delete_confirm_dialog(&mut self, ui: &mut egui::Ui, api: &ApiClient, pattern: String) {
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
                        RichText::new(format!("确定要删除路由规则「{}」吗？", pattern))
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
                            self.delete_route(api, &pattern);
                            self.show_delete_confirm = None;
                        }
                    });
                });
            });
    }
}
