//! LogService 测试模块
//!
//! 提供 LogService 的全面测试覆盖

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::services::log_service::{LogEntry, LogFilter, LogLevel, LogParser, LogService, LogFileInfo, LogSource, ExportFormat, LogServiceState};
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// 创建测试用的日志条目
    fn create_test_entry(level: LogLevel, source: &str, message: &str) -> LogEntry {
        LogEntry::new(level, source, message)
    }

    /// 创建临时日志文件
    fn create_temp_log_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "{}", content).unwrap();
        file.flush().unwrap();
        file
    }

    // ==================== LogService 创建测试 ====================

    /// 测试 LogService 创建
    #[test]
    fn test_log_service_new() {
        let _service = LogService::new();
        // 验证服务创建成功
    }

    /// 测试 LogService 默认实现
    #[test]
    fn test_log_service_default() {
        let _service: LogService = Default::default();
        // 验证服务创建成功
    }

    // ==================== 日志文件注册测试 ====================

    /// 测试注册日志文件
    #[tokio::test]
    async fn test_register_log_file() {
        let service = LogService::new();
        let temp_file = create_temp_log_file("Test log content");
        let path = temp_file.path().to_path_buf();

        let result = service.register_log_file(path, "test".to_string()).await;
        assert!(result.is_ok());

        let files = service.get_log_files().await;
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].source, "test");
    }

    /// 测试注册不存在的文件
    #[tokio::test]
    async fn test_register_nonexistent_file() {
        let service = LogService::new();
        let path = std::path::PathBuf::from("/nonexistent/path/file.log");

        let result = service.register_log_file(path, "test".to_string()).await;
        assert!(result.is_err());
    }

    /// 测试获取已注册的日志文件列表
    #[tokio::test]
    async fn test_get_log_files() {
        let service = LogService::new();

        // 注册多个文件
        let file1 = create_temp_log_file("Log 1");
        let file2 = create_temp_log_file("Log 2");

        service.register_log_file(file1.path().to_path_buf(), "source1".to_string()).await.unwrap();
        service.register_log_file(file2.path().to_path_buf(), "source2".to_string()).await.unwrap();

        let files = service.get_log_files().await;
        assert_eq!(files.len(), 2);

        let sources: Vec<_> = files.iter().map(|f| f.source.clone()).collect();
        assert!(sources.contains(&"source1".to_string()));
        assert!(sources.contains(&"source2".to_string()));
    }

    // ==================== 读取日志测试 ====================

    /// 测试读取日志文件
    #[tokio::test]
    async fn test_read_log_file() {
        let service = LogService::new();
        let log_content = "2024-01-15 10:30:45 [INFO] Test message\n2024-01-15 10:30:46 [ERROR] Error message";
        let temp_file = create_temp_log_file(log_content);

        service.register_log_file(temp_file.path().to_path_buf(), "test".to_string()).await.unwrap();

        let filter = LogFilter::default();
        let entries = service.get_recent_logs(10, &filter).await.unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, LogLevel::Error); // 按时间倒序
        assert_eq!(entries[1].level, LogLevel::Info);
    }

    /// 测试读取最近日志（带限制）
    #[tokio::test]
    async fn test_get_recent_logs_with_limit() {
        let service = LogService::new();
        let mut log_content = String::new();
        for i in 0..20 {
            log_content.push_str(&format!("2024-01-15 10:30:{:02} [INFO] Message {}\n", i % 60, i));
        }
        let temp_file = create_temp_log_file(&log_content);

        service.register_log_file(temp_file.path().to_path_buf(), "test".to_string()).await.unwrap();

        let filter = LogFilter::default();
        let entries = service.get_recent_logs(5, &filter).await.unwrap();

        assert_eq!(entries.len(), 5);
    }

    /// 测试读取空日志文件
    #[tokio::test]
    async fn test_read_empty_log_file() {
        let service = LogService::new();
        let temp_file = create_temp_log_file("");

        service.register_log_file(temp_file.path().to_path_buf(), "test".to_string()).await.unwrap();

        let filter = LogFilter::default();
        let entries = service.get_recent_logs(10, &filter).await.unwrap();

        // 空文件可能返回空或包含一个空条目（取决于解析器行为）
        // 主要验证不 panic
        assert!(entries.len() <= 1);
    }

    /// 测试读取文件末尾行
    #[tokio::test]
    async fn test_read_last_lines() {
        let service = LogService::new();
        let mut log_content = String::new();
        for i in 0..100 {
            log_content.push_str(&format!("2024-01-15 10:30:{:02} [INFO] Message {}\n", i % 60, i));
        }
        let temp_file = create_temp_log_file(&log_content);

        let entries = service.read_last_lines(
            temp_file.path().to_str().unwrap(), "test", 10).await.unwrap();

        assert_eq!(entries.len(), 10);
    }

    // ==================== 日志解析测试 ====================

    /// 测试标准格式日志解析
    #[test]
    fn test_parse_standard_format() {
        let parser = LogParser::new();
        let line = "2024-01-15 10:30:45 [INFO] This is a test message";
        let entry = parser.parse_line(line, "test").unwrap();

        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.source, "test");
        assert_eq!(entry.message, "This is a test message");
    }

    /// 测试 ISO 格式日志解析
    #[test]
    fn test_parse_iso_format() {
        let parser = LogParser::new();
        let line = "2024-01-15T10:30:45+00:00 [ERROR] Something went wrong";
        let entry = parser.parse_line(line, "test").unwrap();

        assert_eq!(entry.level, LogLevel::Error);
        assert_eq!(entry.message, "Something went wrong");
    }

    /// 测试带毫秒的日志解析
    #[test]
    fn test_parse_with_milliseconds() {
        let parser = LogParser::new();
        let line = "2024-01-15 10:30:45.123 [DEBUG] Debug message";
        let entry = parser.parse_line(line, "test").unwrap();

        assert_eq!(entry.level, LogLevel::Debug);
    }

    /// 测试 JSON 格式日志解析
    #[test]
    fn test_parse_json_format() {
        let parser = LogParser::new();
        let line = r#"{"timestamp": "2024-01-15T10:30:45Z", "level": "WARN", "message": "JSON log message", "extra": "data"}"#;
        let entry = parser.parse_line(line, "test").unwrap();

        assert_eq!(entry.level, LogLevel::Warn);
        assert_eq!(entry.message, "JSON log message");
        assert!(entry.metadata.is_some());
        assert!(entry.metadata.as_ref().unwrap().contains_key("extra"));
    }

    /// 测试无效格式日志解析（应返回 INFO 级别）
    #[test]
    fn test_parse_invalid_format() {
        let parser = LogParser::new();
        let line = "Just a plain text message without format";
        let entry = parser.parse_line(line, "test").unwrap();

        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.message, line);
    }

    /// 测试空行解析
    #[test]
    fn test_parse_empty_line() {
        let parser = LogParser::new();
        let line = "";
        let entry = parser.parse_line(line, "test");

        // 空行应该返回 Some 或 None 取决于实现
        // 当前实现会返回带有空消息的条目
        assert!(entry.is_some());
    }

    // ==================== 日志级别测试 ====================

    /// 测试 LogLevel 从字符串解析
    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("ERROR"), Some(LogLevel::Error));
        assert_eq!(LogLevel::from_str("error"), Some(LogLevel::Error));
        assert_eq!(LogLevel::from_str("ERR"), Some(LogLevel::Error));
        assert_eq!(LogLevel::from_str("FATAL"), Some(LogLevel::Error));
        assert_eq!(LogLevel::from_str("CRITICAL"), Some(LogLevel::Error));

        assert_eq!(LogLevel::from_str("WARN"), Some(LogLevel::Warn));
        assert_eq!(LogLevel::from_str("WARNING"), Some(LogLevel::Warn));

        assert_eq!(LogLevel::from_str("INFO"), Some(LogLevel::Info));

        assert_eq!(LogLevel::from_str("DEBUG"), Some(LogLevel::Debug));

        assert_eq!(LogLevel::from_str("TRACE"), Some(LogLevel::Trace));

        assert_eq!(LogLevel::from_str("UNKNOWN"), None);
        assert_eq!(LogLevel::from_str(""), None);
    }

    /// 测试 LogLevel 优先级
    #[test]
    fn test_log_level_priority() {
        assert!(LogLevel::Error.priority() < LogLevel::Warn.priority());
        assert!(LogLevel::Warn.priority() < LogLevel::Info.priority());
        assert!(LogLevel::Info.priority() < LogLevel::Debug.priority());
        assert!(LogLevel::Debug.priority() < LogLevel::Trace.priority());
    }

    /// 测试 LogLevel 默认
    #[test]
    fn test_log_level_default() {
        let level: LogLevel = Default::default();
        assert!(matches!(level, LogLevel::Info));
    }

    // ==================== 日志过滤测试 ====================

    /// 测试空过滤器（匹配所有）
    #[test]
    fn test_empty_filter_matches_all() {
        let filter = LogFilter::default();
        let entry = create_test_entry(LogLevel::Debug, "any", "any message");
        assert!(filter.matches(&entry));
    }

    /// 测试按级别过滤
    #[test]
    fn test_filter_by_level() {
        let filter = LogFilter {
            levels: vec![LogLevel::Error, LogLevel::Warn],
            search_query: None,
            sources: None,
            start_time: None,
            end_time: None,
        };

        let error_entry = create_test_entry(LogLevel::Error, "test", "error");
        let warn_entry = create_test_entry(LogLevel::Warn, "test", "warn");
        let info_entry = create_test_entry(LogLevel::Info, "test", "info");

        assert!(filter.matches(&error_entry));
        assert!(filter.matches(&warn_entry));
        assert!(!filter.matches(&info_entry));
    }

    /// 测试按来源过滤
    #[test]
    fn test_filter_by_source() {
        let filter = LogFilter {
            levels: vec![],
            search_query: None,
            sources: Some(vec!["openclaw".to_string(), "manager".to_string()]),
            start_time: None,
            end_time: None,
        };

        let openclaw_entry = create_test_entry(LogLevel::Info, "openclaw", "message");
        let manager_entry = create_test_entry(LogLevel::Info, "manager", "message");
        let other_entry = create_test_entry(LogLevel::Info, "other", "message");

        assert!(filter.matches(&openclaw_entry));
        assert!(filter.matches(&manager_entry));
        assert!(!filter.matches(&other_entry));
    }

    /// 测试按时间范围过滤
    #[test]
    fn test_filter_by_time_range() {
        let filter = LogFilter {
            levels: vec![],
            search_query: None,
            sources: None,
            start_time: Some(1000),
            end_time: Some(2000),
        };

        let in_range = LogEntry {
            id: "1".to_string(),
            timestamp: 1500,
            level: LogLevel::Info,
            source: "test".to_string(),
            message: "message".to_string(),
            metadata: None,
        };

        let too_early = LogEntry {
            id: "2".to_string(),
            timestamp: 500,
            level: LogLevel::Info,
            source: "test".to_string(),
            message: "message".to_string(),
            metadata: None,
        };

        let too_late = LogEntry {
            id: "3".to_string(),
            timestamp: 2500,
            level: LogLevel::Info,
            source: "test".to_string(),
            message: "message".to_string(),
            metadata: None,
        };

        assert!(filter.matches(&in_range));
        assert!(!filter.matches(&too_early));
        assert!(!filter.matches(&too_late));
    }

    /// 测试仅开始时间过滤
    #[test]
    fn test_filter_by_start_time_only() {
        let filter = LogFilter {
            levels: vec![],
            search_query: None,
            sources: None,
            start_time: Some(1000),
            end_time: None,
        };

        let after = LogEntry {
            id: "1".to_string(),
            timestamp: 1500,
            level: LogLevel::Info,
            source: "test".to_string(),
            message: "message".to_string(),
            metadata: None,
        };

        let before = LogEntry {
            id: "2".to_string(),
            timestamp: 500,
            level: LogLevel::Info,
            source: "test".to_string(),
            message: "message".to_string(),
            metadata: None,
        };

        assert!(filter.matches(&after));
        assert!(!filter.matches(&before));
    }

    /// 测试仅结束时间过滤
    #[test]
    fn test_filter_by_end_time_only() {
        let filter = LogFilter {
            levels: vec![],
            search_query: None,
            sources: None,
            start_time: None,
            end_time: Some(2000),
        };

        let before = LogEntry {
            id: "1".to_string(),
            timestamp: 1500,
            level: LogLevel::Info,
            source: "test".to_string(),
            message: "message".to_string(),
            metadata: None,
        };

        let after = LogEntry {
            id: "2".to_string(),
            timestamp: 2500,
            level: LogLevel::Info,
            source: "test".to_string(),
            message: "message".to_string(),
            metadata: None,
        };

        assert!(filter.matches(&before));
        assert!(!filter.matches(&after));
    }

    /// 测试按搜索查询过滤（消息内容）
    #[test]
    fn test_filter_by_search_query_message() {
        let filter = LogFilter {
            levels: vec![],
            search_query: Some("error".to_string()),
            sources: None,
            start_time: None,
            end_time: None,
        };

        let matching = create_test_entry(LogLevel::Info, "test", "This is an error message");
        let non_matching = create_test_entry(LogLevel::Info, "test", "This is fine");

        assert!(filter.matches(&matching));
        assert!(!filter.matches(&non_matching));
    }

    /// 测试按搜索查询过滤（来源）
    #[test]
    fn test_filter_by_search_query_source() {
        let filter = LogFilter {
            levels: vec![],
            search_query: Some("openclaw".to_string()),
            sources: None,
            start_time: None,
            end_time: None,
        };

        let matching = create_test_entry(LogLevel::Info, "openclaw", "message");
        let non_matching = create_test_entry(LogLevel::Info, "other", "message");

        assert!(filter.matches(&matching));
        assert!(!filter.matches(&non_matching));
    }

    /// 测试搜索查询大小写不敏感
    #[test]
    fn test_filter_search_case_insensitive() {
        let filter = LogFilter {
            levels: vec![],
            search_query: Some("ERROR".to_string()),
            sources: None,
            start_time: None,
            end_time: None,
        };

        let lower = create_test_entry(LogLevel::Info, "test", "this is an error");
        let upper = create_test_entry(LogLevel::Info, "test", "This is an ERROR");

        assert!(filter.matches(&lower));
        assert!(filter.matches(&upper));
    }

    /// 测试组合过滤条件
    #[test]
    fn test_combined_filters() {
        let filter = LogFilter {
            levels: vec![LogLevel::Error, LogLevel::Warn],
            search_query: Some("critical".to_string()),
            sources: Some(vec!["openclaw".to_string()]),
            start_time: Some(1000),
            end_time: Some(2000),
        };

        // 完全匹配
        let full_match = LogEntry {
            id: "1".to_string(),
            timestamp: 1500,
            level: LogLevel::Error,
            source: "openclaw".to_string(),
            message: "Critical error occurred".to_string(),
            metadata: None,
        };

        // 级别不匹配
        let wrong_level = LogEntry {
            id: "2".to_string(),
            timestamp: 1500,
            level: LogLevel::Info,
            source: "openclaw".to_string(),
            message: "Critical error occurred".to_string(),
            metadata: None,
        };

        // 来源不匹配
        let wrong_source = LogEntry {
            id: "3".to_string(),
            timestamp: 1500,
            level: LogLevel::Error,
            source: "other".to_string(),
            message: "Critical error occurred".to_string(),
            metadata: None,
        };

        // 搜索不匹配
        let wrong_search = LogEntry {
            id: "4".to_string(),
            timestamp: 1500,
            level: LogLevel::Error,
            source: "openclaw".to_string(),
            message: "Some other message".to_string(),
            metadata: None,
        };

        // 时间不匹配
        let wrong_time = LogEntry {
            id: "5".to_string(),
            timestamp: 500,
            level: LogLevel::Error,
            source: "openclaw".to_string(),
            message: "Critical error occurred".to_string(),
            metadata: None,
        };

        assert!(filter.matches(&full_match));
        assert!(!filter.matches(&wrong_level));
        assert!(!filter.matches(&wrong_source));
        assert!(!filter.matches(&wrong_search));
        assert!(!filter.matches(&wrong_time));
    }

    // ==================== 日志导出测试 ====================

    /// 测试导出为文本格式
    #[tokio::test]
    async fn test_export_logs_text() {
        let service = LogService::new();
        let log_content = "2024-01-15 10:30:45 [INFO] Test message\n2024-01-15 10:30:46 [ERROR] Error message";
        let temp_file = create_temp_log_file(log_content);

        service.register_log_file(temp_file.path().to_path_buf(), "test".to_string()).await.unwrap();

        let output_file = NamedTempFile::new().unwrap();
        let filter = LogFilter::default();

        service.export_logs(&filter, ExportFormat::Text, output_file.path().to_str().unwrap()).await.unwrap();

        let content = std::fs::read_to_string(output_file.path()).unwrap();
        // 导出应该成功并包含内容
        assert!(!content.is_empty());
        // 验证文件被创建且有内容
        assert!(output_file.path().exists());
    }

    /// 测试导出为 JSON 格式
    #[tokio::test]
    async fn test_export_logs_json() {
        let service = LogService::new();
        let log_content = "2024-01-15 10:30:45 [INFO] Test message";
        let temp_file = create_temp_log_file(log_content);

        service.register_log_file(temp_file.path().to_path_buf(), "test".to_string()).await.unwrap();

        let output_file = NamedTempFile::new().unwrap();
        let filter = LogFilter::default();

        service.export_logs(&filter, ExportFormat::Json, output_file.path().to_str().unwrap()).await.unwrap();

        let content = std::fs::read_to_string(output_file.path()).unwrap();
        let entries: Vec<LogEntry> = serde_json::from_str(&content).unwrap();
        assert!(!entries.is_empty());
    }

    /// 测试导出为 CSV 格式
    #[tokio::test]
    async fn test_export_logs_csv() {
        let service = LogService::new();
        let log_content = "2024-01-15 10:30:45 [INFO] Test message";
        let temp_file = create_temp_log_file(log_content);

        service.register_log_file(temp_file.path().to_path_buf(), "test".to_string()).await.unwrap();

        let output_file = NamedTempFile::new().unwrap();
        let filter = LogFilter::default();

        service.export_logs(&filter, ExportFormat::Csv, output_file.path().to_str().unwrap()).await.unwrap();

        let content = std::fs::read_to_string(output_file.path()).unwrap();
        // 验证 CSV 头部
        assert!(content.contains("timestamp,level,source,message"));
        // 验证文件被创建且有内容
        assert!(!content.is_empty());
    }

    /// 测试导出带引号的 CSV 内容
    #[tokio::test]
    async fn test_export_logs_csv_with_quotes() {
        let service = LogService::new();
        let log_content = r#"2024-01-15 10:30:45 [INFO] Message with "quotes""#;
        let temp_file = create_temp_log_file(log_content);

        service.register_log_file(temp_file.path().to_path_buf(), "test".to_string()).await.unwrap();

        let output_file = NamedTempFile::new().unwrap();
        let filter = LogFilter::default();

        service.export_logs(&filter, ExportFormat::Csv, output_file.path().to_str().unwrap()).await.unwrap();

        let content = std::fs::read_to_string(output_file.path()).unwrap();
        // 引号应该被转义
        assert!(content.contains("\"\""));
    }

    /// 测试带过滤条件的导出
    #[tokio::test]
    async fn test_export_logs_with_filter() {
        let service = LogService::new();
        let log_content = "2024-01-15 10:30:45 [INFO] Info message\n2024-01-15 10:30:46 [ERROR] Error message";
        let temp_file = create_temp_log_file(log_content);

        service.register_log_file(temp_file.path().to_path_buf(), "test".to_string()).await.unwrap();

        let output_file = NamedTempFile::new().unwrap();
        let filter = LogFilter {
            levels: vec![LogLevel::Error],
            search_query: None,
            sources: None,
            start_time: None,
            end_time: None,
        };

        service.export_logs(&filter, ExportFormat::Text, output_file.path().to_str().unwrap()).await.unwrap();

        let content = std::fs::read_to_string(output_file.path()).unwrap();
        assert!(content.contains("Error message"));
        assert!(!content.contains("Info message"));
    }

    // ==================== 日志条目测试 ====================

    /// 测试 LogEntry 创建
    #[test]
    fn test_log_entry_new() {
        let entry = LogEntry::new(LogLevel::Info, "test", "message");

        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.source, "test");
        assert_eq!(entry.message, "message");
        assert!(!entry.id.is_empty());
        assert!(entry.timestamp > 0);
        assert!(entry.metadata.is_none());
    }

    /// 测试 LogEntry 带元数据
    #[test]
    fn test_log_entry_with_metadata() {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("key".to_string(), serde_json::json!("value"));
        metadata.insert("number".to_string(), serde_json::json!(42));

        let entry = LogEntry {
            id: "test-id".to_string(),
            timestamp: 1234567890,
            level: LogLevel::Info,
            source: "test".to_string(),
            message: "test message".to_string(),
            metadata: Some(metadata),
        };

        assert_eq!(entry.id, "test-id");
        assert_eq!(entry.timestamp, 1234567890);
        assert!(entry.metadata.is_some());
        let meta = entry.metadata.unwrap();
        assert_eq!(meta["key"], "value");
        assert_eq!(meta["number"], 42);
    }

    // ==================== LogSource 测试 ====================

    /// 测试 LogSource 转换
    #[test]
    fn test_log_source_as_str() {
        assert_eq!(LogSource::OpenClaw.as_str(), "openclaw");
        assert_eq!(LogSource::System.as_str(), "system");
        assert_eq!(LogSource::Manager.as_str(), "manager");
        assert_eq!(LogSource::Plugin("my-plugin".to_string()).as_str(), "plugin:my-plugin");
    }

    // ==================== LogFileInfo 测试 ====================

    /// 测试 LogFileInfo 创建
    #[test]
    fn test_log_file_info() {
        let info = LogFileInfo {
            path: "/var/log/test.log".to_string(),
            source: "test".to_string(),
            size: 1024,
            modified: 1234567890,
        };

        assert_eq!(info.path, "/var/log/test.log");
        assert_eq!(info.source, "test");
        assert_eq!(info.size, 1024);
        assert_eq!(info.modified, 1234567890);
    }

    // ==================== ExportFormat 测试 ====================

    /// 测试 ExportFormat 变体
    #[test]
    fn test_export_format_variants() {
        let text = ExportFormat::Text;
        let json = ExportFormat::Json;
        let csv = ExportFormat::Csv;

        assert!(matches!(text, ExportFormat::Text));
        assert!(matches!(json, ExportFormat::Json));
        assert!(matches!(csv, ExportFormat::Csv));
    }

    // ==================== 扫描日志目录测试 ====================

    /// 测试获取默认日志目录
    #[test]
    fn test_get_default_log_dir() {
        let dir = LogService::get_default_log_dir();
        // 取决于环境，可能返回 Some 或 None
        // 但不应该 panic
    }

    /// 测试扫描日志目录
    #[tokio::test]
    async fn test_scan_log_directory() {
        let service = LogService::new();

        // 创建临时目录
        let temp_dir = tempfile::tempdir().unwrap();

        // 创建一些日志文件
        let log1 = temp_dir.path().join("openclaw.log");
        let log2 = temp_dir.path().join("plugin_test.log");
        let log3 = temp_dir.path().join("other.txt");

        std::fs::write(&log1, "log1 content").unwrap();
        std::fs::write(&log2, "log2 content").unwrap();
        std::fs::write(&log3, "not a log").unwrap();

        // 扫描目录
        // 注意：scan_log_directory 是私有方法，我们通过 register_log_file 来测试
        service.register_log_file(log1, "test".to_string()).await.unwrap();
        service.register_log_file(log2, "test".to_string()).await.unwrap();

        let files = service.get_log_files().await;
        assert_eq!(files.len(), 2);
    }

    // ==================== 错误处理测试 ====================

    /// 测试读取不存在的日志文件
    #[tokio::test]
    async fn test_read_nonexistent_log_file() {
        let service = LogService::new();
        let filter = LogFilter::default();

        // 不注册任何文件，直接读取
        let entries = service.get_recent_logs(10, &filter).await.unwrap();
        assert!(entries.is_empty());
    }

    /// 测试导出到无效路径
    #[tokio::test]
    async fn test_export_to_invalid_path() {
        let service = LogService::new();
        let log_content = "2024-01-15 10:30:45 [INFO] Test message";
        let temp_file = create_temp_log_file(log_content);

        service.register_log_file(temp_file.path().to_path_buf(), "test".to_string()).await.unwrap();

        let filter = LogFilter::default();
        let result = service.export_logs(&filter, ExportFormat::Text, "/invalid/path/file.txt").await;

        assert!(result.is_err());
    }

    // ==================== LogServiceState 测试 ====================

    /// 测试 LogServiceState 创建
    #[test]
    fn test_log_service_state_new() {
        let state = LogServiceState::new();
        // 验证创建成功
    }

    /// 测试 LogServiceState 默认实现
    #[test]
    fn test_log_service_state_default() {
        let state: LogServiceState = Default::default();
        // 验证创建成功
    }

    // ==================== 边界情况测试 ====================

    /// 测试大量日志条目
    #[tokio::test]
    async fn test_large_number_of_logs() {
        let service = LogService::new();
        let mut log_content = String::new();

        for i in 0..1000 {
            log_content.push_str(&format!("2024-01-15 10:30:{:02} [INFO] Message {}\n", i % 60, i));
        }

        let temp_file = create_temp_log_file(&log_content);
        service.register_log_file(temp_file.path().to_path_buf(), "test".to_string()).await.unwrap();

        let filter = LogFilter::default();
        let entries = service.get_recent_logs(100, &filter).await.unwrap();

        assert_eq!(entries.len(), 100);
    }

    /// 测试特殊字符消息
    #[tokio::test]
    async fn test_special_characters_in_message() {
        let service = LogService::new();
        let log_content = r#"2024-01-15 10:30:45 [INFO] Special chars: äöü € 日本語 🎉"#;
        let temp_file = create_temp_log_file(log_content);

        service.register_log_file(temp_file.path().to_path_buf(), "test".to_string()).await.unwrap();

        let filter = LogFilter::default();
        let entries = service.get_recent_logs(10, &filter).await.unwrap();

        assert_eq!(entries.len(), 1);
        assert!(entries[0].message.contains("äöü"));
    }

    /// 测试多行日志消息
    #[tokio::test]
    async fn test_multiline_log_message() {
        let service = LogService::new();
        // 注意：实际日志解析器可能不支持真正的多行消息
        // 这取决于具体实现
        let log_content = "2024-01-15 10:30:45 [INFO] Line 1\n2024-01-15 10:30:46 [INFO] Line 2";
        let temp_file = create_temp_log_file(log_content);

        service.register_log_file(temp_file.path().to_path_buf(), "test".to_string()).await.unwrap();

        let filter = LogFilter::default();
        let entries = service.get_recent_logs(10, &filter).await.unwrap();

        assert_eq!(entries.len(), 2);
    }
}
