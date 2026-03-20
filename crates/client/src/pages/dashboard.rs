//! 仪表盘页面

use eframe::egui::{self, Color32, RichText};
use crate::api::ServerStatus;

#[derive(Debug, Clone, Default)]
pub struct DashboardPage;

impl DashboardPage {
    pub fn show(&mut self, ui: &mut egui::Ui, status: &Option<ServerStatus>) {
        ui.vertical(|ui| {
            // 标题
            ui.horizontal(|ui| {
                ui.label(RichText::new("[*] 仪表盘").size(24.0).strong());
            });
            ui.add_space(20.0);

            if let Some(s) = status {
                // 统计卡片
                egui::Grid::new("stats_grid")
                    .num_columns(3)
                    .spacing([20.0, 10.0])
                    .show(ui, |ui| {
                        stat_card(ui, "运行时间", &format_duration(s.uptime_secs), Color32::from_rgb(66, 133, 244));
                        stat_card(ui, "请求数", &format_number(s.total_requests), Color32::from_rgb(52, 168, 83));
                        stat_card(ui, "Token 消耗", &format_number(s.total_tokens), Color32::from_rgb(251, 188, 5));
                    });

                ui.add_space(20.0);

                // 服务商状态
                if !s.providers.is_empty() {
                    ui.label(RichText::new("服务商状态").size(16.0).strong());
                    ui.add_space(10.0);
                    
                    for provider in &s.providers {
                        provider_status_card(ui, provider);
                        ui.add_space(5.0);
                    }
                }

                ui.add_space(20.0);

                // 快速操作
                ui.label(RichText::new("快速操作").size(16.0).strong());
                ui.add_space(10.0);
                
                egui::Grid::new("actions_grid")
                    .num_columns(3)
                    .spacing([10.0, 10.0])
                    .show(ui, |ui| {
                        if ui.button("[>] 路由配置").clicked() {}
                        if ui.button("[+] 添加服务商").clicked() {}
                        if ui.button("[o] 系统设置").clicked() {}
                    });

                ui.add_space(20.0);

                // 使用说明
                ui.label(RichText::new("使用说明").size(16.0).strong());
                ui.add_space(10.0);
                
                egui::Frame::none()
                    .fill(Color32::from_rgb(35, 35, 35))
                    .rounding(5.0)
                    .inner_margin(15.0)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new("API 端点:").strong().size(12.0));
                            ui.add_space(5.0);
                            
                            code_block(ui, "http://127.0.0.1:31415/v1/chat/completions");
                            ui.add_space(5.0);
                            
                            ui.label(RichText::new("在客户端配置中将 API Base URL 设置为上述地址即可使用。").color(Color32::GRAY).size(11.0));
                        });
                    });

                ui.add_space(20.0);

                // 服务商列表
                if !s.providers.is_empty() {
                    ui.label(RichText::new("已配置服务商").size(16.0).strong());
                    ui.add_space(10.0);
                    
                    egui::Frame::none()
                        .fill(Color32::from_rgb(35, 35, 35))
                        .rounding(5.0)
                        .inner_margin(15.0)
                        .show(ui, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                for provider in &s.providers {
                                    ui.label(RichText::new(format!("[{}] ", provider.name)).color(Color32::from_rgb(100, 180, 255)));
                                }
                            });
                        });
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("! 未连接到服务器").size(18.0).color(Color32::from_rgb(255, 100, 100)));
                        ui.add_space(10.0);
                        ui.label(RichText::new("请确保服务正在运行").color(Color32::GRAY));
                        ui.add_space(5.0);
                        ui.label(RichText::new("http://127.0.0.1:31415").color(Color32::GRAY).code());
                    });
                });
            }
        });
    }
}

fn stat_card(ui: &mut egui::Ui, label: &str, value: &str, color: Color32) {
    egui::Frame::none()
        .fill(color.linear_multiply(0.1))
        .rounding(5.0)
        .inner_margin(15.0)
        .show(ui, |ui| {
            ui.set_min_width(140.0);
            ui.label(RichText::new(label).color(Color32::GRAY).size(12.0));
            ui.add_space(5.0);
            ui.label(RichText::new(value).size(24.0).color(color).strong());
        });
}

fn provider_status_card(ui: &mut egui::Ui, provider: &crate::api::ProviderStatus) {
    egui::Frame::none()
        .fill(Color32::from_rgb(40, 40, 40))
        .rounding(5.0)
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if provider.healthy {
                    ui.label(RichText::new("●").color(Color32::from_rgb(0, 200, 100)));
                } else {
                    ui.label(RichText::new("●").color(Color32::from_rgb(255, 100, 100)));
                }
                ui.label(&provider.name);
                if let Some(latency) = provider.latency_ms {
                    ui.label(RichText::new(format!("{}ms", latency)).color(Color32::GRAY));
                }
            });
        });
}

fn code_block(ui: &mut egui::Ui, text: &str) {
    egui::Frame::none()
        .fill(Color32::from_rgb(30, 30, 30))
        .rounding(3.0)
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.label(RichText::new(text).code().color(Color32::from_rgb(200, 200, 200)));
        });
}

fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{} 秒", secs)
    } else if secs < 3600 {
        format!("{} 分", secs / 60)
    } else if secs < 86400 {
        format!("{} 时 {} 分", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{} 天", secs / 86400)
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
