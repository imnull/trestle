//! 设置页面

use eframe::egui::{self, Color32, RichText};
use crate::api::ApiClient;

#[derive(Debug, Clone)]
pub struct SettingsPage {
    host: String,
    port: String,
    theme: String,
    language: String,
    auto_start: bool,
    minimize_to_tray: bool,
    initialized: bool,
    save_status: Option<String>,
}

impl Default for SettingsPage {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: "31415".to_string(),
            theme: "dark".to_string(),
            language: "zh-CN".to_string(),
            auto_start: true,
            minimize_to_tray: true,
            initialized: false,
            save_status: None,
        }
    }
}

impl SettingsPage {
    pub fn show(&mut self, ui: &mut egui::Ui, api: &ApiClient) {
        if !self.initialized {
            self.load_config(api);
            self.initialized = true;
        }

        ui.vertical(|ui| {
            ui.label(RichText::new("[o] 设置").size(24.0).strong());
            ui.add_space(20.0);

            if let Some(ref status) = self.save_status {
                ui.label(RichText::new(status).color(Color32::from_rgb(0, 200, 100)));
                ui.add_space(10.0);
            }

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
                        egui::ComboBox::from_id_salt("theme_select")
                            .selected_text(match self.theme.as_str() {
                                "light" => "浅色",
                                "dark" => "深色",
                                _ => "跟随系统",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.theme, "light".to_string(), "浅色");
                                ui.selectable_value(&mut self.theme, "dark".to_string(), "深色");
                                ui.selectable_value(&mut self.theme, "system".to_string(), "跟随系统");
                            });
                        ui.end_row();

                        ui.label("语言:");
                        egui::ComboBox::from_id_salt("lang_select")
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
                    ui.label("配置目录:");
                    let config_dir = dirs::config_dir()
                        .map(|p: std::path::PathBuf| p.join("trestle").to_string_lossy().to_string())
                        .unwrap_or_else(|| "~/.config/trestle".to_string());
                    ui.label(RichText::new(&config_dir).color(Color32::GRAY));
                });
            });

            ui.add_space(30.0);

            // 操作按钮
            ui.horizontal(|ui| {
                if ui.button("保存").clicked() {
                    self.save_config(api);
                }
                if ui.button("重新加载").clicked() {
                    self.initialized = false;
                    self.save_status = None;
                }
            });
        });
    }

    fn load_config(&mut self, api: &ApiClient) {
        match api.get_config() {
            Ok(config) => {
                self.host = config.server.host.clone();
                self.port = config.server.port.to_string();
                self.theme = config.ui.theme.clone();
                self.language = config.ui.language.clone();
                self.auto_start = config.ui.auto_start;
                self.minimize_to_tray = config.ui.minimize_to_tray;
            }
            Err(e) => {
                eprintln!("Failed to load config: {}", e);
            }
        }
    }

    fn save_config(&mut self, api: &ApiClient) {
        let port: u16 = self.port.parse().unwrap_or(31415);
        
        let config = trestle_core::Config {
            server: trestle_core::ServerConfig {
                host: self.host.clone(),
                port,
            },
            ui: trestle_core::UiConfig {
                theme: self.theme.clone(),
                language: self.language.clone(),
                auto_start: self.auto_start,
                minimize_to_tray: self.minimize_to_tray,
            },
            logging: trestle_core::LoggingConfig::default(),
        };

        let result: Result<(), anyhow::Error> = (|| {
            api.update_config(&config)?;
            api.save_config()?;
            Ok(())
        })();

        match result {
            Ok(_) => {
                self.save_status = Some("配置已保存".to_string());
            }
            Err(e) => {
                self.save_status = Some(format!("保存失败: {}", e));
            }
        }
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
