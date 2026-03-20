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
    last_update: Option<Instant>,
    connected: bool,
}

impl TrestleApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 加载中文字体
        Self::setup_chinese_fonts(&cc.egui_ctx);

        Self {
            current_page: Page::default(),
            dashboard: DashboardPage::default(),
            providers: ProvidersPage::default(),
            routes: RoutesPage::default(),
            logs: LogsPage::default(),
            settings: SettingsPage::default(),
            api: ApiClient::new("http://127.0.0.1:31415".to_string()),
            server_status: Arc::new(Mutex::new(None)),
            last_update: None,  // 首次立即执行
            connected: false,
        }
    }

    fn setup_chinese_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        // 尝试加载系统中文字体
        let chinese_fonts = [
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
            "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        ];

        for font_path in &chinese_fonts {
            if let Ok(font_data) = std::fs::read(font_path) {
                fonts.font_data.insert(
                    "ChineseFont".to_owned(),
                    std::sync::Arc::new(egui::FontData::from_owned(font_data)),
                );
                // 将中文字体添加到所有字体的首选位置
                for family in [&egui::FontFamily::Proportional, &egui::FontFamily::Monospace] {
                    if let Some(keys) = fonts.families.get_mut(family) {
                        keys.insert(0, "ChineseFont".to_owned());
                    }
                }
                println!("Loaded Chinese font from: {}", font_path);
                break;
            }
        }

        // 加载 emoji 字体
        let emoji_fonts = [
            "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf",
            "/usr/share/fonts/truetype/noto/NotoEmoji-Regular.ttf",
            "/usr/share/fonts/truetype/twitter-twemoji/Twemoji.ttf",
        ];

        for font_path in &emoji_fonts {
            if let Ok(font_data) = std::fs::read(font_path) {
                fonts.font_data.insert(
                    "EmojiFont".to_owned(),
                    std::sync::Arc::new(egui::FontData::from_owned(font_data)),
                );
                // 将 emoji 字体添加到字体族
                for family in [&egui::FontFamily::Proportional, &egui::FontFamily::Monospace] {
                    if let Some(keys) = fonts.families.get_mut(family) {
                        keys.push("EmojiFont".to_owned());
                    }
                }
                println!("Loaded Emoji font from: {}", font_path);
                break;
            }
        }

        ctx.set_fonts(fonts);
    }

    fn update_status(&mut self) {
        // 首次立即执行，之后每 2 秒更新一次
        if let Some(last) = self.last_update {
            if last.elapsed() < Duration::from_secs(2) {
                return;
            }
        }
        self.last_update = Some(Instant::now());

        // 使用阻塞 API 获取状态
        match self.api.get::<ServerStatus>("/api/status") {
            Ok(s) => {
                println!("DEBUG: Got status, connected=true");
                *self.server_status.lock().unwrap() = Some(s);
                self.connected = true;
            }
            Err(e) => {
                println!("DEBUG: Failed to get status: {}", e);
                *self.server_status.lock().unwrap() = None;
                self.connected = false;
            }
        }
    }
}

impl eframe::App for TrestleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 请求持续重绘以定期检查状态
        ctx.request_repaint_after(std::time::Duration::from_secs(2));

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
                    ui.label(RichText::new("[Trestle]").size(20.0).strong());
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
        // 使用基本 ASCII 符号
        let nav_items = [
            (Page::Dashboard, "[*] 仪表盘"),
            (Page::Providers, "[+] 服务商"),
            (Page::Routes, "[>] 路由"),
            (Page::Logs, "[#] 日志"),
            (Page::Settings, "[o] 设置"),
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
