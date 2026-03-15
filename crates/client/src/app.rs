//! 主应用

use eframe::egui;
use egui::RichText;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::pages::{DashboardPage, ProvidersPage, RoutesPage, LogsPage, SettingsPage};
use crate::api::{ApiClient, ServerStatus};

/// 页面枚举
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Page {
    #[default]
    Dashboard,
    Providers,
    Routes,
    Logs,
    Settings,
}

/// 主应用
pub struct TrestleApp {
    current_page: Page,
    dashboard: DashboardPage,
    providers: ProvidersPage,
    routes: RoutesPage,
    logs: LogsPage,
    settings: SettingsPage,
    api: ApiClient,
    server_status: Arc<Mutex<Option<ServerStatus>>>,
    last_update: Instant,
    connected: bool,
}

impl TrestleApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            current_page: Page::default(),
            dashboard: DashboardPage::default(),
            providers: ProvidersPage::default(),
            routes: RoutesPage::default(),
            logs: LogsPage::default(),
            settings: SettingsPage::default(),
            api: ApiClient::new("http://127.0.0.1:31415".to_string()),
            server_status: Arc::new(Mutex::new(None)),
            last_update: Instant::now(),
            connected: false,
        }
    }

    fn update_status(&mut self) {
        // 每 2 秒更新一次
        if self.last_update.elapsed() < Duration::from_secs(2) {
            return;
        }
        self.last_update = Instant::now();

        let api = self.api.clone();
        let status = self.server_status.clone();

        // 异步获取状态
        std::thread::spawn(move || {
            if let Ok(rt) = tokio::runtime::Runtime::new() {
                match rt.block_on(api.get::<ServerStatus>("/api/status")) {
                    Ok(s) => {
                        *status.lock().unwrap() = Some(s);
                    }
                    Err(e) => {
                        eprintln!("Failed to get status: {}", e);
                        *status.lock().unwrap() = None;
                    }
                }
            }
        });

        // 检查连接状态
        self.connected = self.server_status.lock().unwrap().is_some();
    }
}

impl eframe::App for TrestleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 定期更新状态
        self.update_status();

        // 主布局: 侧边栏 + 内容区
        egui::SidePanel::left("sidebar")
            .default_width(180.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.set_min_width(180.0);
                ui.set_max_width(180.0);

                // Logo
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.add_space(10.0);
                    ui.label(RichText::new("⚡ Trestle").size(20.0).strong());
                });
                ui.add_space(20.0);

                // 导航菜单
                self.show_nav(ui);

                // 底部状态
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.add_space(10.0);
                        if self.connected {
                            ui.label(RichText::new("● 运行中").color(egui::Color32::from_rgb(0, 200, 100)));
                        } else {
                            ui.label(RichText::new("○ 未连接").color(egui::Color32::GRAY));
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.add_space(10.0);
                        ui.label(RichText::new("端口: 31415").size(11.0).color(egui::Color32::GRAY));
                    });
                    ui.add_space(10.0);
                });
            });

        // 内容区
        egui::CentralPanel::default().show(ctx, |ui| {
            let status = self.server_status.lock().unwrap().clone();
            match self.current_page {
                Page::Dashboard => self.dashboard.show(ui, &status),
                Page::Providers => self.providers.show(ui, &self.api),
                Page::Routes => self.routes.show(ui, &self.api),
                Page::Logs => self.logs.show(ui, &self.api),
                Page::Settings => self.settings.show(ui),
            }
        });
    }
}

impl TrestleApp {
    fn show_nav(&mut self, ui: &mut egui::Ui) {
        let nav_items = [
            (Page::Dashboard, "📊 仪表盘"),
            (Page::Providers, "🔌 服务商"),
            (Page::Routes, "🛤 路由"),
            (Page::Logs, "📜 日志"),
            (Page::Settings, "⚙ 设置"),
        ];

        for (page, label) in nav_items {
            let is_active = self.current_page == page;
            let response = ui.selectable_label(is_active, label);

            if response.clicked() {
                self.current_page = page;
            }
        }
    }
}
