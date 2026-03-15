//! 配置管理

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 默认端口
pub const DEFAULT_PORT: u16 = 31415;

/// 默认主机
pub const DEFAULT_HOST: &str = "localhost";

/// 主配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 服务配置
    pub server: ServerConfig,
    /// UI 配置
    #[serde(default)]
    pub ui: UiConfig,
    /// 日志配置
    #[serde(default)]
    pub logging: LoggingConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            ui: UiConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Config {
    /// 从文件加载配置
    pub fn from_file(path: &PathBuf) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// 保存配置到文件
    pub fn to_file(&self, path: &PathBuf) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// 获取默认配置路径
    pub fn default_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("trestle");
        config_dir.join("config.toml")
    }
}

/// 服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// 监听主机
    #[serde(default = "default_host")]
    pub host: String,
    /// 监听端口
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_host() -> String {
    DEFAULT_HOST.to_string()
}

fn default_port() -> u16 {
    DEFAULT_PORT
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: DEFAULT_HOST.to_string(),
            port: DEFAULT_PORT,
        }
    }
}

/// UI 配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiConfig {
    /// 主题: light, dark, system
    #[serde(default = "default_theme")]
    pub theme: String,
    /// 语言
    #[serde(default = "default_language")]
    pub language: String,
    /// 开机自启
    #[serde(default)]
    pub auto_start: bool,
    /// 最小化到托盘
    #[serde(default = "default_true")]
    pub minimize_to_tray: bool,
}

fn default_theme() -> String {
    "system".to_string()
}

fn default_language() -> String {
    "zh-CN".to_string()
}

fn default_true() -> bool {
    true
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别: debug, info, warn, error
    #[serde(default = "default_log_level")]
    pub level: String,
    /// 日志保留天数
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_retention_days() -> u32 {
    7
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            retention_days: default_retention_days(),
        }
    }
}
