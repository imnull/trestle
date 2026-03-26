//! UI 主题和样式常量

use eframe::egui::{self, Color32, Rounding, Stroke, Vec2};

/// 主题颜色
pub mod colors {
    use super::Color32;
    
    // 主色调
    pub const PRIMARY: Color32 = Color32::from_rgb(59, 130, 246);      // 蓝色
    pub const PRIMARY_DARK: Color32 = Color32::from_rgb(37, 99, 235);
    pub const PRIMARY_LIGHT: Color32 = Color32::from_rgb(96, 165, 250);
    
    // 成功/警告/错误
    pub const SUCCESS: Color32 = Color32::from_rgb(34, 197, 94);       // 绿色
    pub const WARNING: Color32 = Color32::from_rgb(251, 191, 36);      // 黄色
    pub const ERROR: Color32 = Color32::from_rgb(239, 68, 68);         // 红色
    
    // 背景色
    pub const BG_DARKEST: Color32 = Color32::from_rgb(15, 23, 42);     // 最深背景
    pub const BG_DARK: Color32 = Color32::from_rgb(30, 41, 59);        // 深色背景
    pub const BG_CARD: Color32 = Color32::from_rgb(51, 65, 85);        // 卡片背景
    pub const BG_HOVER: Color32 = Color32::from_rgb(71, 85, 105);      // 悬停背景
    
    // 文字色
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(248, 250, 252);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(148, 163, 184);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(100, 116, 139);
    
    // 边框
    pub const BORDER: Color32 = Color32::from_rgb(75, 85, 99);
}

/// 圆角常量
pub mod rounding {
    use super::Rounding;
    
    pub const SMALL: Rounding = Rounding::same(4.0);
    pub const MEDIUM: Rounding = Rounding::same(8.0);
    pub const LARGE: Rounding = Rounding::same(12.0);
    pub const XLARGE: Rounding = Rounding::same(16.0);
}

/// 间距常量
pub mod spacing {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 16.0;
    pub const XL: f32 = 24.0;
    pub const XXL: f32 = 32.0;
}

/// 图标（使用 Unicode 符号，确保在 Symbol 字体中可用）
pub mod icons {
    // 使用基础 Unicode 符号，兼容 Symbol 字体
    pub const DASHBOARD: &str = "◆";      // 菱形 - 仪表盘
    pub const PROVIDERS: &str = "●";      // 圆点 - 服务商
    pub const ROUTES: &str = "▶";       // 箭头 - 路由
    pub const LOGS: &str = "≡";        // 三条线 - 日志
    pub const SETTINGS: &str = "⚙";     // 齿轮 - 设置
    
    pub const ADD: &str = "+";
    pub const EDIT: &str = "✎";
    pub const DELETE: &str = "×";
    pub const REFRESH: &str = "↻";
    pub const SAVE: &str = "✓";
    pub const CANCEL: &str = "✕";
    pub const CHECK: &str = "✓";
    pub const WARNING: &str = "!";       // 警告
    pub const ERROR: &str = "✕";
    pub const INFO: &str = "i";          // 信息
    pub const SEARCH: &str = "⌕";
    pub const FILTER: &str = "▼";
    pub const MORE: &str = "...";        // 更多
    
    pub const SERVER: &str = "◆";       // 服务器（菱形）
    pub const ONLINE: &str = "●";       // 在线
    pub const OFFLINE: &str = "○";      // 离线
    pub const TIME: &str = "◔";       // 时间
    pub const REQUESTS: &str = "↗";     // 请求（上升箭头）
    pub const TOKENS: &str = "#";       // Token（井号）
    pub const CONNECTIONS: &str = "∞";   // 连接（无穷符号）
}

/// 设置卡片样式
pub fn card_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(colors::BG_CARD)
        .rounding(rounding::MEDIUM)
        .inner_margin(egui::Margin::same(spacing::LG))
}

/// 设置悬停卡片样式
pub fn card_frame_hover() -> egui::Frame {
    egui::Frame::none()
        .fill(colors::BG_HOVER)
        .rounding(rounding::MEDIUM)
        .inner_margin(egui::Margin::same(spacing::LG))
        .stroke(Stroke::new(1.0, colors::PRIMARY))
}

/// 主按钮样式
pub fn primary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(
            egui::RichText::new(text)
                .size(14.0)
                .strong()
        )
        .fill(colors::PRIMARY)
        .rounding(rounding::SMALL)
        .min_size(Vec2::new(100.0, 36.0))
    )
}

/// 次级按钮样式
pub fn secondary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(
            egui::RichText::new(text)
                .size(14.0)
                .color(colors::TEXT_PRIMARY)
        )
        .fill(colors::BG_CARD)
        .rounding(rounding::SMALL)
        .min_size(Vec2::new(100.0, 36.0))
    )
}

/// 危险按钮样式
pub fn danger_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(
            egui::RichText::new(text)
                .size(14.0)
                .strong()
        )
        .fill(colors::ERROR.linear_multiply(0.8))
        .rounding(rounding::SMALL)
        .min_size(Vec2::new(80.0, 32.0))
    )
}

/// 图标按钮
pub fn icon_button(ui: &mut egui::Ui, icon: &str, tooltip: &str) -> egui::Response {
    ui.add(
        egui::Button::new(
            egui::RichText::new(icon).size(16.0)
        )
        .fill(Color32::TRANSPARENT)
    )
    .on_hover_text(tooltip)
}

/// 页面标题
pub fn page_title(ui: &mut egui::Ui, title: &str, subtitle: &str) {
    ui.vertical(|ui| {
        ui.label(
            egui::RichText::new(title)
                .size(28.0)
                .strong()
                .color(colors::TEXT_PRIMARY)
        );
        ui.add_space(spacing::XS);
        ui.label(
            egui::RichText::new(subtitle)
                .size(14.0)
                .color(colors::TEXT_SECONDARY)
        );
    });
    ui.add_space(spacing::XL);
}

/// 空状态提示
pub fn empty_state(ui: &mut egui::Ui, icon: &str, title: &str, description: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(spacing::XXL * 2.0);
        ui.label(
            egui::RichText::new(icon)
                .size(48.0)
        );
        ui.add_space(spacing::LG);
        ui.label(
            egui::RichText::new(title)
                .size(18.0)
                .strong()
                .color(colors::TEXT_SECONDARY)
        );
        ui.add_space(spacing::SM);
        ui.label(
            egui::RichText::new(description)
                .size(14.0)
                .color(colors::TEXT_MUTED)
        );
    });
}

/// 统计卡片
pub fn stat_card(ui: &mut egui::Ui, icon: &str, label: &str, value: &str, color: Color32) {
    card_frame()
        .outer_margin(egui::Margin::same(0.0))
        .show(ui, |ui| {
            ui.set_min_width(160.0);
            ui.set_min_height(80.0);
            ui.horizontal_centered(|ui| {
                // 图标
                ui.label(
                    egui::RichText::new(icon).size(28.0)
                );
                ui.add_space(spacing::MD);
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(label)
                            .size(12.0)
                            .color(colors::TEXT_SECONDARY)
                    );
                    ui.add_space(spacing::XS);
                    ui.label(
                        egui::RichText::new(value)
                            .size(22.0)
                            .strong()
                            .color(color)
                    );
                });
            });
        });
}

/// 输入框样式
pub fn styled_text_edit(ui: &mut egui::Ui, text: &mut String, placeholder: &str) -> egui::Response {
    ui.add(
        egui::TextEdit::singleline(text)
            .hint_text(placeholder)
            .min_size(Vec2::new(200.0, 36.0))
    )
}

/// 分隔线
pub fn divider(ui: &mut egui::Ui) {
    ui.add_space(spacing::MD);
    ui.add(
        egui::Separator::default()
            .spacing(spacing::MD)
            .shrink(0.0)
    );
    ui.add_space(spacing::MD);
}

/// 标签样式
pub fn badge(ui: &mut egui::Ui, text: &str, color: Color32) {
    egui::Frame::none()
        .fill(color)
        .rounding(rounding::SMALL)
        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(text)
                    .size(11.0)
                    .strong()
                    .color(Color32::WHITE)
            );
        });
}
