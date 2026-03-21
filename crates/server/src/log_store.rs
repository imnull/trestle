//! SQLite 日志存储

use rusqlite::{Connection, Result as SqliteResult};
use std::path::PathBuf;
use std::sync::Mutex;
use trestle_core::RequestLog;
use chrono::{DateTime, Utc};

/// SQLite 日志存储
pub struct LogStore {
    conn: Mutex<Connection>,
}

impl LogStore {
    /// 创建或打开日志数据库
    pub fn new(path: Option<PathBuf>) -> SqliteResult<Self> {
        let db_path = path.unwrap_or_else(|| {
            dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("trestle")
                .join("logs.db")
        });

        // 确保目录存在
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let conn = Connection::open(&db_path)?;
        
        // 创建日志表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS request_logs (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                method TEXT NOT NULL,
                path TEXT NOT NULL,
                model TEXT NOT NULL,
                status INTEGER NOT NULL,
                latency_ms INTEGER NOT NULL,
                prompt_tokens INTEGER,
                completion_tokens INTEGER
            )",
            [],
        )?;

        // 创建索引加速查询
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp ON request_logs(timestamp)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_model ON request_logs(model)",
            [],
        )?;

        tracing::info!("SQLite log store initialized at {:?}", db_path);

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// 添加日志
    pub fn add_log(&self, log: &RequestLog) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO request_logs (id, timestamp, method, path, model, status, latency_ms, prompt_tokens, completion_tokens)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            [
                &log.id,
                &log.timestamp.to_rfc3339(),
                &log.method,
                &log.path,
                &log.model,
                &log.status.to_string(),
                &log.latency_ms.to_string(),
                &log.prompt_tokens.map(|t| t.to_string()).unwrap_or_default(),
                &log.completion_tokens.map(|t| t.to_string()).unwrap_or_default(),
            ],
        )?;
        Ok(())
    }

    /// 获取日志列表
    pub fn get_logs(&self, limit: usize, offset: usize) -> SqliteResult<Vec<RequestLog>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, method, path, model, status, latency_ms, prompt_tokens, completion_tokens
             FROM request_logs
             ORDER BY timestamp DESC
             LIMIT ?1 OFFSET ?2"
        )?;

        let logs = stmt.query_map([limit, offset], |row| {
            Ok(RequestLog {
                id: row.get(0)?,
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(1)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                method: row.get(2)?,
                path: row.get(3)?,
                model: row.get(4)?,
                status: row.get(5)?,
                latency_ms: row.get(6)?,
                prompt_tokens: row.get(7)?,
                completion_tokens: row.get(8)?,
            })
        })?.collect::<SqliteResult<Vec<_>>>();

        logs
    }

    /// 获取日志总数
    pub fn count_logs(&self) -> SqliteResult<usize> {
        let conn = self.conn.lock().unwrap();
        let count: usize = conn.query_row("SELECT COUNT(*) FROM request_logs", [], |row| row.get(0))?;
        Ok(count)
    }

    /// 清理旧日志（保留最近 N 天）
    pub fn cleanup_old_logs(&self, retention_days: u64) -> SqliteResult<usize> {
        let cutoff = Utc::now() - chrono::Duration::days(retention_days as i64);
        let conn = self.conn.lock().unwrap();
        let deleted = conn.execute(
            "DELETE FROM request_logs WHERE timestamp < ?1",
            [cutoff.to_rfc3339()],
        )?;
        tracing::info!("Cleaned up {} old log entries", deleted);
        Ok(deleted)
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> SqliteResult<LogStats> {
        let conn = self.conn.lock().unwrap();
        
        let total_requests: u64 = conn.query_row(
            "SELECT COUNT(*) FROM request_logs", 
            [], 
            |row| row.get(0)
        )?;

        let total_tokens: u64 = conn.query_row(
            "SELECT COALESCE(SUM(prompt_tokens), 0) + COALESCE(SUM(completion_tokens), 0) FROM request_logs",
            [],
            |row| row.get(0)
        )?;

        let avg_latency: f64 = conn.query_row(
            "SELECT AVG(latency_ms) FROM request_logs",
            [],
            |row| row.get(0)
        ).unwrap_or(0.0);

        let error_count: u64 = conn.query_row(
            "SELECT COUNT(*) FROM request_logs WHERE status >= 400",
            [],
            |row| row.get(0)
        )?;

        Ok(LogStats {
            total_requests,
            total_tokens,
            avg_latency_ms: avg_latency as u64,
            error_count,
        })
    }
}

/// 日志统计
#[derive(Debug, Clone)]
pub struct LogStats {
    pub total_requests: u64,
    pub total_tokens: u64,
    pub avg_latency_ms: u64,
    pub error_count: u64,
}
