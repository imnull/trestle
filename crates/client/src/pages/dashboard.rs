//! 仪表盘页面

use eframe::egui::{self, Color32, RichText, Align};
use crate::api::ServerStatus;
use crate::ui_theme::{self, colors, spacing, icons, card_frame};

#[derive(Debug, Clone, Default)]
pub struct DashboardPage;

impl DashboardPage {
    pub fn show(&mut self, ui: &mut egui::Ui, status: &Option<ServerStatus>) {
        if let Some(s) = status {
            self.show_connected(ui, s);
        } else {
            self.show_disconnected(ui);
        }
    }
    
    fn show_connected(&mut self, ui: &mut egui::Ui, s: &ServerStatus) {
        // 使用垂直布局，统一间距
        ui.spacing_mut().item_spacing = egui::Vec2::new(0.0, spacing::MD);
        
        // ========== 统计卡片区域 ==========
        ui.horizontal(|ui| {
            ui.set_width(ui.available_width());
            
            // 4 个等宽卡片
            let card_width = (ui.available_width() - spacing::LG * 3.0) / 4.0;
            
            self.stat_card_fixed(
                ui, 
                card_width,
                icons::TIME, 
                "运行时间", 
                &format_duration(s.uptime_secs),
                colors::PRIMARY
            );
            ui.add_space(spacing::LG);
            self.stat_card_fixed(
                ui, 
                card_width,
                icons::REQUESTS, 
                "总请求数", 
                &format_number(s.total_requests),
                colors::SUCCESS
            );
            ui.add_space(spacing::LG);
            self.stat_card_fixed(
                ui, 
                card_width,
                icons::TOKENS, 
                "Token 消耗", 
                &format_number(s.total_tokens),
                colors::WARNING
            );
            ui.add_space(spacing::LG);
            self.stat_card_fixed(
                ui, 
                card_width,
                icons::CONNECTIONS, 
                "活跃连接", 
                &s.active_connections.to_string(),
                colors::PRIMARY_LIGHT
            );
        });
        
        ui.add_space(spacing::LG);
        
        // ========== 两列布局 ==========
        ui.horizontal(|ui| {
            let half_width = (ui.available_width() - spacing::LG) / 2.0;
            
            // 左侧：使用指南
            ui.vertical(|ui| {
                ui.set_width(half_width);
                
                // 标题
                ui.label(
                    RichText::new(format!("{} 快速开始", icons::INFO))
                        .size(15.0)
                        .strong()
                        .color(colors::TEXT_PRIMARY)
                );
                
                ui.add_space(spacing::MD);
                
                // 步骤卡片
                card_frame().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    
                    // 步骤 1
                    self.step_item(ui, "①", "添加服务商", "点击左侧「服务商」，添加 OpenAI/Claude 等");
                    
                    ui.add_space(spacing::MD);
                    ui.separator();
                    ui.add_space(spacing::MD);
                    
                    // 步骤 2
                    self.step_item(ui, "②", "配置路由", "进入「路由规则」，设置模型匹配规则");
                    
                    ui.add_space(spacing::MD);
                    ui.separator();
                    ui.add_space(spacing::MD);
                    
                    // 步骤 3
                    self.step_item(ui, "③", "开始使用", "在客户端配置 API Base URL 即可");
                });
                
                ui.add_space(spacing::LG);
                
                // API 端点
                ui.label(
                    RichText::new(format!("{} API 端点", icons::SERVER))
                        .size(15.0)
                        .strong()
                        .color(colors::TEXT_PRIMARY)
                );
                
                ui.add_space(spacing::MD);
                
                card_frame().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    
                    ui.label(
                        RichText::new("OpenAI 兼容端点")
                            .size(12.0)
                            .color(colors::TEXT_SECONDARY)
                    );
                    
                    ui.add_space(spacing::SM);
                    
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut "http://127.0.0.1:31415/v1/chat/completions".to_string())
                                .font(egui::TextStyle::Monospace)
                                .desired_width(ui.available_width() - 80.0)
                        );
                        
                        if ui.button("复制").clicked() {
                            ui.output_mut(|o| o.copied_text = "http://127.0.0.1:31415/v1/chat/completions".to_string());
                        }
                    });
                    
                    ui.add_space(spacing::SM);
                    
                    ui.label(
                        RichText::new("在 ChatGPT 客户端或其他支持自定义 API 的应用中使用上述地址")
                            .size(11.0)
                            .color(colors::TEXT_MUTED)
                    );
                });
            });
            
            ui.add_space(spacing::LG);
            
            // 右侧：服务商状态
            ui.vertical(|ui| {
                ui.set_width(half_width);
                
                ui.label(
                    RichText::new(format!("{} 服务商状态", icons::PROVIDERS))
                        .size(15.0)
                        .strong()
                        .color(colors::TEXT_PRIMARY)
                );
                
                ui.add_space(spacing::MD);
                
                if s.providers.is_empty() {
                    card_frame().show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(spacing::XL);
                            ui.label(
                                RichText::new("!")
                                    .size(32.0)
                                    .color(colors::WARNING)
                            );
                            ui.add_space(spacing::SM);
                            ui.label(
                                RichText::new("暂无服务商")
                                    .size(13.0)
                                    .strong()
                                    .color(colors::TEXT_SECONDARY)
                            );
                            ui.add_space(spacing::XS);
                            ui.label(
                                RichText::new("请前往「服务商」页面添加")
                                    .size(11.0)
                                    .color(colors::TEXT_MUTED)
                            );
                            ui.add_space(spacing::XL);
                        });
                    });
                } else {
                    for provider in &s.providers {
                        self.provider_status_card(ui, provider);
                        ui.add_space(spacing::SM);
                    }
                }
            });
        });
    }
    
    fn stat_card_fixed(&self, ui: &mut egui::Ui, width: f32, icon: &str, label: &str, value: &str, color: Color32) {
        card_frame().show(ui, |ui| {
            ui.set_width(width);
            ui.set_min_height(70.0);
            
            ui.horizontal_centered(|ui| {
                // 图标
                ui.label(
                    RichText::new(icon).size(22.0)
                );
                ui.add_space(spacing::SM);
                
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(label)
                            .size(11.0)
                            .color(colors::TEXT_SECONDARY)
                    );
                    ui.label(
                        RichText::new(value)
                            .size(18.0)
                            .strong()
                            .color(color)
                    );
                });
            });
        });
    }
    
    fn step_item(&self, ui: &mut egui::Ui, num: &str, title: &str, desc: &str) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(num)
                    .size(18.0)
                    .color(colors::PRIMARY)
            );
            ui.add_space(spacing::SM);
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(title)
                        .size(13.0)
                        .strong()
                        .color(colors::TEXT_PRIMARY)
                );
                ui.label(
                    RichText::new(desc)
                        .size(11.0)
                        .color(colors::TEXT_SECONDARY)
                );
            });
        });
    }
    
    fn show_disconnected(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(80.0);
            
            ui.label(
                RichText::new(icons::OFFLINE)
                    .size(48.0)
            );
            
            ui.add_space(spacing::LG);
            
            ui.label(
                RichText::new("服务未连接")
                    .size(20.0)
                    .strong()
                    .color(colors::TEXT_PRIMARY)
            );
            
            ui.add_space(spacing::SM);
            
            ui.label(
                RichText::new("内嵌服务正在启动中，请稍候...")
                    .size(13.0)
                    .color(colors::TEXT_SECONDARY)
            );
            
            ui.add_space(spacing::SM);
            
            ui.label(
                RichText::new("http://127.0.0.1:31415")
                    .size(11.0)
                    .color(colors::TEXT_MUTED)
                    .code()
            );
        });
    }
    
    fn provider_status_card(&self, ui: &mut egui::Ui, provider: &crate::api::ProviderStatus) {
        let status_color = if provider.healthy { colors::SUCCESS } else { colors::ERROR };
        let status_text = if provider.healthy { "正常" } else { "异常" };
        
        card_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(if provider.healthy { icons::ONLINE } else { icons::OFFLINE })
                        .size(12.0)
                        .color(status_color)
                );
                ui.add_space(spacing::XS);
                
                ui.label(
                    RichText::new(&provider.name)
                        .size(13.0)
                        .strong()
                        .color(colors::TEXT_PRIMARY)
                );
                
                if let Some(latency) = provider.latency_ms {
                    ui.label(
                        RichText::new(format!("{}ms", latency))
                            .size(11.0)
                            .color(colors::TEXT_MUTED)
                    );
                }
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui_theme::badge(ui, status_text, status_color);
                });
            });
        });
    }
}

fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86400 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}d", secs / 86400)
    }
}

fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        format!("{}", n)
    }
}
