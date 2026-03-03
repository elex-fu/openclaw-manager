//! 日志服务
//!
//! 提供日志文件读取、解析和筛选功能

use crate::errors::app_error::AppError;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    /// 从字符串解析日志级别
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "ERROR" | "ERR" | "FATAL" | "CRITICAL" => Some(LogLevel::Error),
            "WARN" | "WARNING" => Some(LogLevel::Warn),
            "INFO" => Some(LogLevel::Info),
            "DEBUG" => Some(LogLevel::Debug),
            "TRACE" => Some(LogLevel::Trace),
            _ => None,
        }
    }

    /// 获取日志级别的优先级（用于筛选）
    pub fn priority(&self) -> u8 {
        match self {
            LogLevel::Error => 0,
            LogLevel::Warn => 1,
            LogLevel::Info => 2,
            LogLevel::Debug => 3,
            LogLevel::Trace => 4,
        }
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

/// 日志来源
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogSource {
    OpenClaw,
    Plugin(String),
    System,
    Manager,
}

impl LogSource {
    pub fn as_str(&self) -> String {
        match self {
            LogSource::OpenClaw => "openclaw".to_string(),
            LogSource::Plugin(name) => format!("plugin:{}", name),
            LogSource::System => "system".to_string(),
            LogSource::Manager => "manager".to_string(),
        }
    }
}

/// 日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    pub timestamp: i64,
    pub level: LogLevel,
    pub source: String,
    pub message: String,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl LogEntry {
    pub fn new(level: LogLevel, source: &str, message: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            level,
            source: source.to_string(),
            message: message.to_string(),
            metadata: None,
        }
    }
}

/// 日志筛选条件
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogFilter {
    pub levels: Vec<LogLevel>,
    pub search_query: Option<String>,
    pub sources: Option<Vec<String>>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
}

impl LogFilter {
    /// 检查日志条目是否匹配筛选条件
    pub fn matches(&self, entry: &LogEntry) -> bool {
        // 检查日志级别
        if !self.levels.is_empty() && !self.levels.contains(&entry.level) {
            return false;
        }

        // 检查日志源
        if let Some(sources) = &self.sources {
            if !sources.contains(&entry.source) {
                return false;
            }
        }

        // 检查时间范围
        if let Some(start) = self.start_time {
            if entry.timestamp < start {
                return false;
            }
        }
        if let Some(end) = self.end_time {
            if entry.timestamp > end {
                return false;
            }
        }

        // 检查搜索关键字
        if let Some(query) = &self.search_query {
            let query_lower = query.to_lowercase();
            if !entry.message.to_lowercase().contains(&query_lower)
                && !entry.source.to_lowercase().contains(&query_lower)
            {
                return false;
            }
        }

        true
    }
}

/// 日志文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFileInfo {
    pub path: String,
    pub source: String,
    pub size: u64,
    pub modified: i64,
}

/// 日志解析器
pub struct LogParser {
    // 标准格式: 2024-01-15 10:30:45 [INFO] message here
    standard_regex: Regex,
    // ISO格式: 2024-01-15T10:30:45+00:00 [INFO] message here
    iso_regex: Regex,
    // JSON格式解析
}

impl Default for LogParser {
    fn default() -> Self {
        Self::new()
    }
}

impl LogParser {
    pub fn new() -> Self {
        Self {
            standard_regex: Regex::new(
                r"^(\d{4}-\d{2}-\d{2}\s+\d{2}:\d{2}:\d{2}(?:\.\d+)?)\s*\[(\w+)\]\s*(.*)$"
            ).unwrap(),
            iso_regex: Regex::new(
                r"^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:[+-]\d{2}:\d{2}|Z)?)\s*\[(\w+)\]\s*(.*)$"
            ).unwrap(),
        }
    }

    /// 解析单行日志
    pub fn parse_line(&self, line: &str, source: &str) -> Option<LogEntry> {
        // 尝试标准格式
        if let Some(captures) = self.standard_regex.captures(line) {
            let timestamp_str = captures.get(1)?.as_str();
            let level_str = captures.get(2)?.as_str();
            let message = captures.get(3)?.as_str();

            let timestamp = self.parse_timestamp(timestamp_str)?;
            let level = LogLevel::from_str(level_str).unwrap_or(LogLevel::Info);

            return Some(LogEntry {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp,
                level,
                source: source.to_string(),
                message: message.to_string(),
                metadata: None,
            });
        }

        // 尝试ISO格式
        if let Some(captures) = self.iso_regex.captures(line) {
            let timestamp_str = captures.get(1)?.as_str();
            let level_str = captures.get(2)?.as_str();
            let message = captures.get(3)?.as_str();

            let timestamp = self.parse_iso_timestamp(timestamp_str)?;
            let level = LogLevel::from_str(level_str).unwrap_or(LogLevel::Info);

            return Some(LogEntry {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp,
                level,
                source: source.to_string(),
                message: message.to_string(),
                metadata: None,
            });
        }

        // 尝试JSON格式
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(obj) = json_value.as_object() {
                let timestamp = obj
                    .get("timestamp")
                    .and_then(|v| v.as_str())
                    .and_then(|s| self.parse_iso_timestamp(s))
                    .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());

                let level = obj
                    .get("level")
                    .and_then(|v| v.as_str())
                    .and_then(LogLevel::from_str)
                    .unwrap_or(LogLevel::Info);

                let message = obj
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or(line)
                    .to_string();

                let mut metadata = HashMap::new();
                for (key, value) in obj {
                    if key != "timestamp" && key != "level" && key != "message" {
                        metadata.insert(key.clone(), value.clone());
                    }
                }

                return Some(LogEntry {
                    id: uuid::Uuid::new_v4().to_string(),
                    timestamp,
                    level,
                    source: source.to_string(),
                    message,
                    metadata: if metadata.is_empty() { None } else { Some(metadata) },
                });
            }
        }

        // 无法解析，返回原始行作为INFO级别
        Some(LogEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            level: LogLevel::Info,
            source: source.to_string(),
            message: line.to_string(),
            metadata: None,
        })
    }

    /// 解析标准时间戳
    fn parse_timestamp(&self, s: &str) -> Option<i64> {
        chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f")
            .or_else(|_| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S"))
            .ok()
            .map(|dt| dt.and_utc().timestamp_millis())
    }

    /// 解析ISO时间戳
    fn parse_iso_timestamp(&self, s: &str) -> Option<i64> {
        chrono::DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|dt| dt.timestamp_millis())
            .or_else(|| {
                chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f")
                    .ok()
                    .map(|dt| dt.and_utc().timestamp_millis())
            })
    }
}

/// 日志服务
pub struct LogService {
    parser: LogParser,
    log_files: Arc<RwLock<Vec<LogFileInfo>>>,
    max_cached_entries: usize,
}

impl Default for LogService {
    fn default() -> Self {
        Self::new()
    }
}

impl LogService {
    pub fn new() -> Self {
        Self {
            parser: LogParser::new(),
            log_files: Arc::new(RwLock::new(Vec::new())),
            max_cached_entries: 10000,
        }
    }

    /// 注册日志文件
    pub async fn register_log_file(&self, path: PathBuf, source: String) -> Result<(), AppError> {
        let metadata = std::fs::metadata(&path)
            .map_err(|e| AppError::Io(e))?;

        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);

        let info = LogFileInfo {
            path: path.to_string_lossy().to_string(),
            source,
            size: metadata.len(),
            modified,
        };

        let mut files = self.log_files.write().await;
        files.push(info);

        Ok(())
    }

    /// 获取已注册的日志文件列表
    pub async fn get_log_files(&self) -> Vec<LogFileInfo> {
        self.log_files.read().await.clone()
    }

    /// 读取日志文件的最近条目
    pub async fn get_recent_logs(
        &self,
        limit: usize,
        filter: &LogFilter,
    ) -> Result<Vec<LogEntry>, AppError> {
        let files = self.log_files.read().await.clone();
        let mut all_entries = Vec::new();

        for file_info in files {
            let entries = self.read_log_file(&file_info.path, &file_info.source).await?;
            all_entries.extend(entries);
        }

        // 按时间戳排序
        all_entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // 应用筛选
        let filtered: Vec<LogEntry> = all_entries
            .into_iter()
            .filter(|entry| filter.matches(entry))
            .take(limit)
            .collect();

        Ok(filtered)
    }

    /// 读取单个日志文件
    async fn read_log_file(
        &self,
        path: &str,
        source: &str,
    ) -> Result<Vec<LogEntry>, AppError> {
        let file = File::open(path).map_err(|e| AppError::Io(e))?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| AppError::Io(e))?;
            if let Some(entry) = self.parser.parse_line(&line, source) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    /// 从文件末尾读取指定行数（用于大文件）
    pub async fn read_last_lines(
        &self,
        path: &str,
        source: &str,
        n: usize,
    ) -> Result<Vec<LogEntry>, AppError> {
        let mut file = File::open(path).map_err(|e| AppError::Io(e))?;
        let file_size = file.metadata().map_err(|e| AppError::Io(e))?.len();

        // 从文件末尾开始读取
        let mut buffer = Vec::new();
        let mut pos = file_size as i64;
        let chunk_size = 8192i64; // 8KB chunks

        while pos > 0 && buffer.len() < n * 200 {
            // 假设每行平均200字节
            let read_size = std::cmp::min(chunk_size, pos);
            pos -= read_size;

            file.seek(SeekFrom::Start(pos as u64))
                .map_err(|e| AppError::Io(e))?;

            let mut chunk = vec![0u8; read_size as usize];
            file.read_exact(&mut chunk).map_err(|e| AppError::Io(e))?;

            // 将新读取的数据插入到buffer前面
            let mut new_buffer = chunk;
            new_buffer.extend(buffer);
            buffer = new_buffer;

            // 统计行数
            let line_count = buffer.iter().filter(|&&b| b == b'\n').count();
            if line_count >= n {
                break;
            }
        }

        // 解析buffer中的行
        let content = String::from_utf8_lossy(&buffer);
        let lines: Vec<&str> = content.lines().collect();
        let start_idx = lines.len().saturating_sub(n);

        let mut entries = Vec::new();
        for line in &lines[start_idx..] {
            if let Some(entry) = self.parser.parse_line(line, source) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    /// 导出日志
    pub async fn export_logs(
        &self,
        filter: &LogFilter,
        format: ExportFormat,
        output_path: &str,
    ) -> Result<(), AppError> {
        let entries = self.get_recent_logs(self.max_cached_entries, filter).await?;

        match format {
            ExportFormat::Text => {
                let mut content = String::new();
                for entry in entries {
                    let timestamp = chrono::DateTime::from_timestamp_millis(entry.timestamp)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_default();
                    content.push_str(&format!(
                        "[{}] [{:?}] [{}] {}\n",
                        timestamp, entry.level, entry.source, entry.message
                    ));
                }
                std::fs::write(output_path, content).map_err(|e| AppError::Io(e))?;
            }
            ExportFormat::Json => {
                let json = serde_json::to_string_pretty(&entries)
                    .map_err(|e| AppError::Serialization(e.to_string()))?;
                std::fs::write(output_path, json).map_err(|e| AppError::Io(e))?;
            }
            ExportFormat::Csv => {
                let mut content = String::from("timestamp,level,source,message\n");
                for entry in entries {
                    let timestamp = chrono::DateTime::from_timestamp_millis(entry.timestamp)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_default();
                    let escaped_message = entry.message.replace('"', "\"\"");
                    content.push_str(&format!(
                        "\"{}\",\"{:?}\",\"{}\",\"{}\"\n",
                        timestamp, entry.level, entry.source, escaped_message
                    ));
                }
                std::fs::write(output_path, content).map_err(|e| AppError::Io(e))?;
            }
        }

        Ok(())
    }

    /// 获取默认的日志目录
    pub fn get_default_log_dir() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".openclaw").join("logs"))
    }

    /// 扫描并注册默认日志文件
    pub async fn scan_default_logs(&self) -> Result<(), AppError> {
        if let Some(log_dir) = Self::get_default_log_dir() {
            if log_dir.exists() {
                self.scan_log_directory(&log_dir).await?;
            }
        }

        // 也扫描应用自己的日志
        if let Some(app_data_dir) = dirs::data_dir() {
            let app_log_dir = app_data_dir.join("openclaw-manager").join("logs");
            if app_log_dir.exists() {
                self.scan_log_directory(&app_log_dir).await?;
            }
        }

        Ok(())
    }

    /// 扫描日志目录
    async fn scan_log_directory(&self, dir: &PathBuf) -> Result<(), AppError> {
        if !dir.is_dir() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir).map_err(|e| AppError::Io(e))? {
            let entry = entry.map_err(|e| AppError::Io(e))?;
            let path = entry.path();

            if path.is_file() {
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");

                let source = if file_name.contains("openclaw") {
                    "openclaw".to_string()
                } else if file_name.contains("plugin") {
                    "plugin".to_string()
                } else if file_name.contains("manager") {
                    "manager".to_string()
                } else {
                    "system".to_string()
                };

                self.register_log_file(path, source).await?;
            }
        }

        Ok(())
    }
}

/// 导出格式
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Text,
    Json,
    Csv,
}

/// 日志服务状态（用于共享状态）
pub struct LogServiceState {
    pub service: Arc<RwLock<LogService>>,
}

impl LogServiceState {
    pub fn new() -> Self {
        Self {
            service: Arc::new(RwLock::new(LogService::new())),
        }
    }
}

impl Default for LogServiceState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("ERROR"), Some(LogLevel::Error));
        assert_eq!(LogLevel::from_str("error"), Some(LogLevel::Error));
        assert_eq!(LogLevel::from_str("WARN"), Some(LogLevel::Warn));
        assert_eq!(LogLevel::from_str("INFO"), Some(LogLevel::Info));
        assert_eq!(LogLevel::from_str("DEBUG"), Some(LogLevel::Debug));
        assert_eq!(LogLevel::from_str("unknown"), None);
    }

    #[test]
    fn test_log_parser_standard_format() {
        let parser = LogParser::new();
        let line = "2024-01-15 10:30:45 [INFO] This is a test message";
        let entry = parser.parse_line(line, "test").unwrap();

        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.source, "test");
        assert_eq!(entry.message, "This is a test message");
    }

    #[test]
    fn test_log_parser_iso_format() {
        let parser = LogParser::new();
        let line = "2024-01-15T10:30:45+00:00 [ERROR] Something went wrong";
        let entry = parser.parse_line(line, "test").unwrap();

        assert_eq!(entry.level, LogLevel::Error);
        assert_eq!(entry.message, "Something went wrong");
    }

    #[test]
    fn test_log_parser_json_format() {
        let parser = LogParser::new();
        let line = r#"{"timestamp": "2024-01-15T10:30:45Z", "level": "WARN", "message": "JSON log message"}"#;
        let entry = parser.parse_line(line, "test").unwrap();

        assert_eq!(entry.level, LogLevel::Warn);
        assert_eq!(entry.message, "JSON log message");
    }

    #[test]
    fn test_log_filter_matches() {
        let filter = LogFilter {
            levels: vec![LogLevel::Error, LogLevel::Warn],
            search_query: Some("test".to_string()),
            sources: None,
            start_time: None,
            end_time: None,
        };

        let matching_entry = LogEntry::new(LogLevel::Error, "source", "this is a test");
        let non_matching_level = LogEntry::new(LogLevel::Info, "source", "this is a test");
        let non_matching_query = LogEntry::new(LogLevel::Error, "source", "no match here");

        assert!(filter.matches(&matching_entry));
        assert!(!filter.matches(&non_matching_level));
        assert!(!filter.matches(&non_matching_query));
    }
}
