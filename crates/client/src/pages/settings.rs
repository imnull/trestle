//! 设置页面

use eframe::egui::{self, Color32, RichText};

#[derive(Debug, Clone, Default)]
pub struct SettingsPage {
    host: String,
    port: String,
    theme: String,
    language: String,
    auto_start: bool,
    minimize_to_tray: bool,
}

impl SettingsPage {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        // 初始化默认值
        if self.host.is_empty() {
            self.host = "localhost".to_string();
            self.port = "31415".to_string();
            self.theme = "system".to_string();
            self.language = "zh-CN".to_string();
            self.auto_start = true;
            self.minimize_to_tray = true;
        }

        ui.vertical(|ui| {
            ui.label(RichText::new("设置").size(24.0).strong());
            ui.add_space(20.0);

            // 服务配置
            section(ui, "服务配置", |ui| {
                egui::Grid::new("server_settings")
                    .num_columns(2)
                    .spacing([20.0, 10.0])
                    .show(ui, |ui| {
                        ui.label("监听地址:");
                        ui.text_edit_singleline(&mut self.host);
                        ui.end_row();

                        ui.label("监听端口:");
                        ui.text_edit_singleline(&mut self.port);
                        ui.end_row();
                    });
            });

            ui.add_space(20.0);

            // 界面设置
            section(ui, "界面设置", |ui| {
                egui::Grid::new("ui_settings")
                    .num_columns(2)
                    .spacing([20.0, 10.0])
                    .show(ui, |ui| {
                        ui.label("主题:");
                        egui::ComboBox::from_label("")
                            .selected_text(&self.theme)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.theme, "light".to_string(), "浅色");
                                ui.selectable_value(&mut self.theme, "dark".to_string(), "深色");
                                ui.selectable_value(&mut self.theme, "system".to_string(), "跟随系统");
                            });
                        ui.end_row();

                        ui.label("语言:");
                        egui::ComboBox::from_label("")
                            .selected_text(match self.language.as_str() {
                                "zh-CN" => "简体中文",
                                "en-US" => "English",
                                _ => &self.language,
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.language, "zh-CN".to_string(), "简体中文");
                                ui.selectable_value(&mut self.language, "en-US".to_string(), "English");
                            });
                        ui.end_row();

                        ui.label("");
                        ui.checkbox(&mut self.auto_start, "开机自启");
                        ui.end_row();

                        ui.label("");
                        ui.checkbox(&mut self.minimize_to_tray, "最小化到托盘");
                        ui.end_row();
                    });
            });

            ui.add_space(20.0);

            // 数据管理
            section(ui, "数据管理", |ui| {
                ui.horizontal(|ui| {
                    ui.label("配置文件:");
                    ui.label(RichText::new("~/.config/trestle/config.toml").color(Color32::GRAY));
                    if ui.small_button("打开目录").clicked() {}
                });
            });

            ui.add_space(30.0);

            // 操作按钮
            ui.horizontal(|ui| {
                if ui.button("导出配置").clicked() {}
                if ui.button("导入配置").clicked() {}
                if ui.button("重置所有设置").clicked() {}
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("保存").clicked() {}
                });
            });
        });
    }
}

fn section(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    ui.label(RichText::new(title).size(14.0).strong());
    ui.add_space(10.0);
    egui::Frame::none()
        .fill(Color32::from_rgb(35, 35, 35))
        .rounding(5.0)
        .inner_margin(15.0)
        .show(ui, add_contents);
    ui.add_space(5.0);
}
