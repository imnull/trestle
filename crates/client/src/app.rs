//! 主应用

use eframe::egui;
use egui::{Color32, RichText};

use crate::pages::{DashboardPage, ProvidersPage, RoutesPage, LogsPage, SettingsPage};
use crate::api::ApiClient;

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
    server_status: Option<ServerStatusInfo>,
    sidebar_width: f32,
}

#[derive(Debug, Clone)]
pub struct ServerStatusInfo {
    pub running: bool,
    pub uptime_secs: u64,
    pub total_requests: u64,
    pub total_tokens: u64,
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
            api: ApiClient::new("http://localhost:31415".to_string()),
            server_status: None,
            sidebar_width: 180.0,
        }
    }

    fn update_status(&mut self) {
        // 异步获取状态 (简化处理)
        if let Ok(rt) = tokio::runtime::Runtime::new() {
            if let Ok(status) = rt.block_on(self.api.get_status()) {
                self.server_status = Some(ServerStatusInfo {
                    running: true,
                    uptime_secs: status.uptime_secs,
                    total_requests: status.total_requests,
                    total_tokens: status.total_tokens,
                });
            } else {
                self.server_status = Some(ServerStatusInfo {
                    running: false,
                    uptime_secs: 0,
                    total_requests: 0,
                    total_tokens: 0,
                });
            }
        }
    }
}

impl eframe::App for TrestleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 定期更新状态
        ctx.request_repaint_after(std::time::Duration::from_secs(2));
        self.update_status();

        // 主布局: 侧边栏 + 内容区
        egui::SidePanel::left("sidebar")
            .default_width(self.sidebar_width)
            .resizable(false)
            .show(ctx, |ui| {
                ui.set_min_width(self.sidebar_width);
                ui.set_max_width(self.sidebar_width);

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
                        if let Some(status) = &self.server_status {
                            if status.running {
                                ui.label(RichText::new("● 运行中").color(Color32::from_rgb(0, 200, 100)));
                            } else {
                                ui.label(RichText::new("○ 已停止").color(Color32::GRAY));
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.add_space(10.0);
                        ui.label(RichText::new("端口: 31415").size(11.0).color(Color32::GRAY));
                    });
                    ui.add_space(10.0);
                });
            });

        // 内容区
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_page {
                Page::Dashboard => self.dashboard.show(ui, &self.server_status),
                Page::Providers => self.providers.show(ui, &self.api),
                Page::Routes => self.routes.show(ui),
                Page::Logs => self.logs.show(ui),
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
