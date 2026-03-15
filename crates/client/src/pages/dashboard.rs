//! 仪表盘页面

use eframe::egui::{self, Color32, RichText};
use crate::app::ServerStatusInfo;

#[derive(Debug, Clone, Default)]
pub struct DashboardPage;

impl DashboardPage {
    pub fn show(&mut self, ui: &mut egui::Ui, status: &Option<ServerStatusInfo>) {
        ui.vertical(|ui| {
            // 标题
            ui.horizontal(|ui| {
                ui.label(RichText::new("仪表盘").size(24.0).strong());
            });
            ui.add_space(20.0);

            // 统计卡片
            if let Some(s) = status {
                egui::Grid::new("stats_grid")
                    .num_columns(4)
                    .spacing([20.0, 10.0])
                    .show(ui, |ui| {
                        stat_card(ui, "运行时间", &format_duration(s.uptime_secs), Color32::from_rgb(66, 133, 244));
                        stat_card(ui, "请求数", &format_number(s.total_requests), Color32::from_rgb(52, 168, 83));
                        stat_card(ui, "Token 消耗", &format_number(s.total_tokens), Color32::from_rgb(251, 188, 5));
                    });
            } else {
                ui.label("服务未连接");
            }

            ui.add_space(30.0);

            // 实时请求流 (占位)
            ui.label(RichText::new("实时请求流").size(16.0).strong());
            ui.add_space(10.0);
            egui::Frame::none()
                .fill(Color32::from_rgb(30, 30, 30))
                .rounding(5.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.set_min_height(200.0);
                    ui.label(RichText::new("等待请求...").color(Color32::GRAY));
                });
        });
    }
}

fn stat_card(ui: &mut egui::Ui, label: &str, value: &str, color: Color32) {
    egui::Frame::none()
        .fill(color.linear_multiply(0.1))
        .rounding(5.0)
        .inner_margin(15.0)
        .show(ui, |ui| {
            ui.set_min_width(120.0);
            ui.label(RichText::new(label).color(Color32::GRAY).size(12.0));
            ui.add_space(5.0);
            ui.label(RichText::new(value).size(24.0).color(color).strong());
        });
}

fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}秒", secs)
    } else if secs < 3600 {
        format!("{}分", secs / 60)
    } else if secs < 86400 {
        format!("{}时{}分", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}天", secs / 86400)
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
