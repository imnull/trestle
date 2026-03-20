//! 路由规则页面

use eframe::egui::{self, Color32, RichText};
use crate::api::{ApiClient, Provider};
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
            ui.horizontal(|ui| {
                ui.label(RichText::new("[>] 路由规则").size(24.0).strong());
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

            if let Some(ref err) = self.error_message {
                ui.label(RichText::new(format!("错误: {}", err)).color(Color32::RED));
                ui.add_space(10.0);
            }

            // 表头
            egui::Frame::none()
                .fill(Color32::from_rgb(35, 35, 35))
                .rounding(3.0)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.set_min_width(ui.available_width());
                        ui.label(RichText::new("优先级").strong().size(12.0));
                        ui.label(RichText::new("匹配规则").strong().size(12.0));
                        ui.label(RichText::new("服务商").strong().size(12.0));
                        ui.label(RichText::new("目标模型").strong().size(12.0));
                        ui.label(RichText::new("操作").strong().size(12.0));
                    });
                });

            ui.add_space(5.0);

            let routes = self.routes.lock().unwrap().clone();
            egui::ScrollArea::vertical().show(ui, |ui| {
                for route in &routes {
                    self.route_row(ui, api, route);
                    ui.add_space(3.0);
                }

                if routes.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new("暂无路由规则").color(Color32::GRAY));
                            ui.add_space(10.0);
                            ui.label(RichText::new("点击上方按钮添加").color(Color32::GRAY));
                        });
                    });
                }
            });
        });

        if self.dialog_open {
            self.show_dialog(ui, api);
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

    fn route_row(&mut self,
        ui: &mut egui::Ui,
        api: &ApiClient,
        route: &crate::api::Route,
    ) {
        egui::Frame::none()
            .fill(Color32::from_rgb(40, 40, 40))
            .rounding(3.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.set_min_width(ui.available_width());
                    
                    ui.label(RichText::new(route.priority.to_string()).size(12.0));
                    ui.label(RichText::new(&route.pattern).code().size(12.0));
                    ui.label(RichText::new(&route.provider).size(12.0));
                    ui.label(RichText::new(route.model.as_deref().unwrap_or("(原样)")).size(12.0).color(Color32::GRAY));
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("删除").clicked() {
                            self.delete_route(api, &route.pattern);
                        }
                        if ui.small_button("编辑").clicked() {
                            self.open_edit_dialog(route);
                        }
                    });
                });
            });
    }

    fn show_dialog(&mut self,
        ui: &mut egui::Ui,
        _api: &ApiClient,
    ) {
        let title = if self.editing_pattern.is_some() {
            "编辑路由"
        } else {
            "添加路由"
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

                egui::Grid::new("route_form")
                    .num_columns(2)
                    .spacing([10.0, 10.0])
                    .show(ui, |ui| {
                        ui.label("匹配规则:");
                        ui.text_edit_singleline(&mut self.form.pattern);
                        ui.end_row();

                        ui.label("服务商:");
                        let providers = self.providers.lock().unwrap();
                        egui::ComboBox::from_id_salt("provider_select")
                            .selected_text(&self.form.provider)
                            .show_ui(ui, |ui| {
                                for p in providers.iter() {
                                    ui.selectable_value(&mut self.form.provider, p.name.clone(), &p.name);
                                }
                            });
                        ui.end_row();

                        ui.label("目标模型:");
                        ui.text_edit_singleline(&mut self.form.model);
                        ui.end_row();

                        ui.label("优先级:");
                        ui.add(egui::DragValue::new(&mut self.form.priority).range(1..=100));
                        ui.end_row();
                    });

                ui.add_space(10.0);
                ui.label(RichText::new("提示: 模型留空表示使用请求中的原始模型名").color(Color32::GRAY).size(11.0));

                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    if ui.button("取消").clicked() {
                        self.dialog_open = false;
                        self.error_message = None;
                    }
                    if ui.button("保存").clicked() {
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
}
