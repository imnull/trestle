//! 错误类型

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TrestleError {
    #[error("配置错误: {0}")]
    Config(String),

    #[error("请求错误: {0}")]
    Request(String),

    #[error("上游服务错误: {0}")]
    Upstream(String),

    #[error("路由错误: {0}")]
    Route(String),

    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP 错误: {0}")]
    Http(String),
}

pub type Result<T> = std::result::Result<T, TrestleError>;
