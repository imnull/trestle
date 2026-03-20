//! 路由规则页面

use eframe::egui::{self, Color32, RichText};
use crate::api::ApiClient;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Default)]
pub struct RoutesPage {
    routes: Arc<Mutex<Vec<crate::api::Route>>>,
    loaded: bool,
}

impl RoutesPage {
    pub fn show(&mut self, ui: &mut egui::Ui, api: &ApiClient) {
        if !self.loaded {
            self.load_routes(api);
            self.loaded = true;
        }

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("路由规则").size(24.0).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("➕ 添加规则").clicked() {}
                    if ui.button("⟳ 刷新").clicked() {
                        self.loaded = false;
                    }
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

            let routes = self.routes.lock().unwrap().clone();
            egui::ScrollArea::vertical().show(ui, |ui| {
                for route in &routes {
                    route_row(ui, route);
                    ui.add_space(5.0);
                }

                if routes.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label(RichText::new("暂无路由规则").color(Color32::GRAY));
                    });
                }
            });
        });
    }

    fn load_routes(&mut self, api: &ApiClient) {
        let routes = self.routes.clone();
        let api = api.clone();
        
        std::thread::spawn(move || {
            if let Ok(rt) = tokio::runtime::Runtime::new() {
                if let Ok(data) = rt.block_on(api.get::<Vec<crate::api::Route>>("/api/routes")) {
                    *routes.lock().unwrap() = data;
                }
            }
        });
    }
}

fn route_row(ui: &mut egui::Ui, route: &crate::api::Route) {
    egui::Frame::none()
        .fill(Color32::from_rgb(40, 40, 40))
        .rounding(3.0)
        .inner_margin(8.0)
        .show(ui, |ui| {
            egui::Grid::new(format!("route_{}_{}", route.priority, route.pattern))
                .num_columns(5)
                .spacing([20.0, 5.0])
                .show(ui, |ui| {
                    ui.label(route.priority.to_string());
                    ui.label(RichText::new(&route.pattern).code());
                    ui.label(&route.provider);
                    ui.label(route.model.as_deref().unwrap_or("(原样)"));
                    ui.horizontal(|ui| {
                        if ui.small_button("编辑").clicked() {}
                        if ui.small_button("删除").clicked() {}
                    });
                    ui.end_row();
                });
        });
}
