//! 主应用 - 现代化界面设计

use eframe::egui;
use eframe::egui::Color32;
use egui::RichText;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::pages::{DashboardPage, ProvidersPage, RoutesPage, LogsPage, SettingsPage};
use crate::api::{ApiClient, ServerStatus};
use crate::ui_theme::{self, colors, spacing, icons};

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

impl Page {
    fn title(&self) -> &'static str {
        match self {
            Page::Dashboard => "仪表盘",
            Page::Providers => "服务商",
            Page::Routes => "路由规则",
            Page::Logs => "请求日志",
            Page::Settings => "设置",
        }
    }
    
    fn icon(&self) -> &'static str {
        match self {
            Page::Dashboard => icons::DASHBOARD,
            Page::Providers => icons::PROVIDERS,
            Page::Routes => icons::ROUTES,
            Page::Logs => icons::LOGS,
            Page::Settings => icons::SETTINGS,
        }
    }
    
    fn description(&self) -> &'static str {
        match self {
            Page::Dashboard => "查看服务状态和统计信息",
            Page::Providers => "管理 AI 服务商配置",
            Page::Routes => "配置模型路由规则",
            Page::Logs => "查看请求历史记录",
            Page::Settings => "应用和系统设置",
        }
    }
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
        // 设置视觉主题和字体
        Self::setup_theme(&cc.egui_ctx);
        Self::setup_fonts(&cc.egui_ctx);

        Self {
            current_page: Page::default(),
            dashboard: DashboardPage::default(),
            providers: ProvidersPage::default(),
            routes: RoutesPage::default(),
            logs: LogsPage::default(),
            settings: SettingsPage::default(),
            api: ApiClient::new("http://127.0.0.1:31415".to_string()),
            server_status: Arc::new(Mutex::new(None)),
            last_update: None,
            connected: false,
        }
    }

    fn setup_theme(ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        
        // 设置间距
        style.spacing.item_spacing = egui::Vec2::new(spacing::MD, spacing::MD);
        style.spacing.window_margin = egui::Margin::same(spacing::LG);
        style.spacing.button_padding = egui::Vec2::new(spacing::MD, spacing::SM);
        
        // 设置圆角
        style.visuals.widgets.active.rounding = ui_theme::rounding::SMALL;
        style.visuals.widgets.inactive.rounding = ui_theme::rounding::SMALL;
        style.visuals.widgets.hovered.rounding = ui_theme::rounding::SMALL;
        style.visuals.widgets.open.rounding = ui_theme::rounding::SMALL;
        style.visuals.window_rounding = ui_theme::rounding::MEDIUM;
        
        // 设置颜色 - 深色主题
        style.visuals.dark_mode = true;
        style.visuals.panel_fill = colors::BG_DARK;
        style.visuals.window_fill = colors::BG_DARK;
        style.visuals.override_text_color = Some(colors::TEXT_PRIMARY);
        
        ctx.set_style(style);
    }

    fn setup_fonts(ctx: &egui::Context) {
        use egui::FontFamily;
        
        let mut fonts = egui::FontDefinitions::default();

        // ========== 1. 首先加载 Emoji/符号 字体 ==========
        // Apple Color Emoji 是彩色位图字体，egui 可能无法正确渲染
        // 使用 Symbol 字体作为备选，它包含基本符号
        let symbol_fonts = if cfg!(target_os = "macos") {
            vec![
                "/System/Library/Fonts/Symbol.ttf",  // 基础符号字体（小，矢量）
                "/System/Library/Fonts/Apple Symbols.ttf",
                "/System/Library/Fonts/Apple Color Emoji.ttc",  // 最后尝试彩色 emoji
            ]
        } else if cfg!(target_os = "windows") {
            vec![
                "C:\\Windows\\Fonts\\seguiemj.ttf",
                "C:\\Windows\\Fonts\\segoeuiemoji.ttf",
                "C:\\Windows\\Fonts\\symbol.ttf",
            ]
        } else {
            vec![
                "/usr/share/fonts/truetype/noto/NotoEmoji-Regular.ttf",
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            ]
        };

        let mut symbol_loaded = false;
        for font_path in &symbol_fonts {
            match std::fs::read(font_path) {
                Ok(font_data) => {
                    println!("[Font] Loading symbol font: {} ({} bytes)", font_path, font_data.len());
                    fonts.font_data.insert(
                        "SymbolFont".to_owned(),
                        std::sync::Arc::new(egui::FontData::from_owned(font_data)),
                    );
                    // 符号字体放在最前面优先匹配
                    for family in [FontFamily::Proportional, FontFamily::Monospace] {
                        if let Some(keys) = fonts.families.get_mut(&family) {
                            keys.insert(0, "SymbolFont".to_owned());
                        }
                    }
                    tracing::info!("Loaded symbol font from: {}", font_path);
                    symbol_loaded = true;
                    break;
                }
                Err(e) => {
                    println!("[Font] Failed to load symbol font {}: {}", font_path, e);
                }
            }
        }
        
        if !symbol_loaded {
            println!("[Font] WARNING: Failed to load any symbol font!");
            tracing::warn!("Failed to load any symbol font");
        }

        // ========== 2. 加载中文字体（放在 emoji 之后）==========
        let chinese_fonts = if cfg!(target_os = "macos") {
            vec![
                "/System/Library/Fonts/PingFang.ttc",
                "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
                "/System/Library/Fonts/Hiragino Sans GB.ttc",
                "/Library/Fonts/Arial Unicode.ttf",
            ]
        } else if cfg!(target_os = "windows") {
            vec![
                "C:\\Windows\\Fonts\\msyh.ttc",
                "C:\\Windows\\Fonts\\simhei.ttf",
            ]
        } else {
            vec![
                "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
                "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
            ]
        };

        for font_path in &chinese_fonts {
            match std::fs::read(font_path) {
                Ok(font_data) => {
                    println!("[Font] Loading Chinese font: {} ({} bytes)", font_path, font_data.len());
                    fonts.font_data.insert(
                        "ChineseFont".to_owned(),
                        std::sync::Arc::new(egui::FontData::from_owned(font_data)),
                    );
                    // 中文字体放在 emoji 之后，默认字体之前
                    for family in [FontFamily::Proportional, FontFamily::Monospace] {
                        if let Some(keys) = fonts.families.get_mut(&family) {
                            // 插入到索引 1 的位置（emoji 之后）
                            if keys.len() > 1 {
                                keys.insert(1, "ChineseFont".to_owned());
                            } else {
                                keys.push("ChineseFont".to_owned());
                            }
                        }
                    }
                    tracing::info!("Loaded Chinese font from: {}", font_path);
                    break;
                }
                Err(e) => {
                    println!("[Font] Failed to load Chinese font {}: {}", font_path, e);
                }
            }
        }

        // 打印最终的字体回退链
        println!("[Font] Font fallback chain: {:?}", 
            fonts.families.get(&FontFamily::Proportional).unwrap_or(&vec![]));
        
        ctx.set_fonts(fonts);
    }

    fn update_status(&mut self) {
        if let Some(last) = self.last_update {
            if last.elapsed() < Duration::from_secs(2) {
                return;
            }
        }
        self.last_update = Some(Instant::now());

        match self.api.get::<ServerStatus>("/api/status") {
            Ok(s) => {
                *self.server_status.lock().unwrap() = Some(s);
                self.connected = true;
            }
            Err(_) => {
                *self.server_status.lock().unwrap() = None;
                self.connected = false;
            }
        }
    }
}

impl eframe::App for TrestleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_secs(2));
        self.update_status();

        // 侧边栏
        egui::SidePanel::left("sidebar")
            .default_width(200.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.set_min_width(200.0);
                ui.set_max_width(200.0);
                ui.spacing_mut().item_spacing = egui::Vec2::new(0.0, 0.0);
                
                ui.add_space(spacing::LG);
                
                // Logo 区域
                ui.horizontal(|ui| {
                    ui.add_space(spacing::LG);
                    ui.label(
                        RichText::new(icons::SERVER)
                            .size(28.0)
                    );
                    ui.add_space(spacing::SM);
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("Trestle")
                                .size(18.0)
                                .strong()
                                .color(colors::TEXT_PRIMARY)
                        );
                        ui.label(
                            RichText::new("AI 代理管理")
                                .size(11.0)
                                .color(colors::TEXT_MUTED)
                        );
                    });
                });
                
                ui.add_space(spacing::XL);
                
                // 导航菜单
                for page in [Page::Dashboard, Page::Providers, Page::Routes, Page::Logs, Page::Settings] {
                    let is_active = self.current_page == page;
                    
                    let (bg_color, text_color) = if is_active {
                        (colors::PRIMARY, Color32::WHITE)
                    } else {
                        (Color32::TRANSPARENT, colors::TEXT_SECONDARY)
                    };
                    
                    let response = ui.add(
                        egui::Button::new(
                            RichText::new(format!("{} {}", page.icon(), page.title()))
                                .size(14.0)
                                .color(text_color)
                        )
                        .fill(bg_color)
                        .rounding(ui_theme::rounding::SMALL)
                        .min_size(egui::Vec2::new(180.0, 40.0))
                    );
                    
                    if response.clicked() {
                        self.current_page = page;
                    }
                    
                    ui.add_space(spacing::XS);
                }
                
                // 底部状态栏
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.add_space(spacing::LG);
                    
                    ui_theme::card_frame().show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if self.connected {
                                ui.label(
                                    RichText::new(icons::ONLINE)
                                        .size(12.0)
                                );
                                ui.add_space(spacing::XS);
                                ui.label(
                                    RichText::new("运行中")
                                        .size(12.0)
                                        .color(colors::SUCCESS)
                                );
                            } else {
                                ui.label(
                                    RichText::new(icons::OFFLINE)
                                        .size(12.0)
                                );
                                ui.add_space(spacing::XS);
                                ui.label(
                                    RichText::new("未连接")
                                        .size(12.0)
                                        .color(colors::TEXT_MUTED)
                                );
                            }
                        });
                        ui.add_space(spacing::XS);
                        ui.label(
                            RichText::new("端口: 31415")
                                .size(11.0)
                                .color(colors::TEXT_MUTED)
                        );
                    });
                    
                    ui.add_space(spacing::LG);
                });
            });

        // 主内容区
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(colors::BG_DARKEST)
                    .inner_margin(egui::Margin::symmetric(spacing::LG, spacing::MD))
            )
            .show(ctx, |ui| {
                let status = self.server_status.lock().unwrap().clone();
                
                // 页面标题区域
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width());
                    ui.label(
                        RichText::new(self.current_page.title())
                            .size(20.0)
                            .strong()
                            .color(colors::TEXT_PRIMARY)
                    );
                    ui.add_space(spacing::XS);
                    ui.label(
                        RichText::new(self.current_page.description())
                            .size(12.0)
                            .color(colors::TEXT_SECONDARY)
                    );
                });
                
                ui.add_space(spacing::MD);
                
                // 页面内容
                match self.current_page {
                    Page::Dashboard => self.dashboard.show(ui, &status),
                    Page::Providers => self.providers.show(ui, &self.api),
                    Page::Routes => self.routes.show(ui, &self.api),
                    Page::Logs => self.logs.show(ui, &self.api),
                    Page::Settings => self.settings.show(ui, &self.api),
                }
            });
    }
}
