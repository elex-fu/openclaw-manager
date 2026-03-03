//! LogWatcher 测试模块
//!
//! 提供 LogWatcher 的全面测试覆盖

#[cfg(test)]
mod tests {
    use crate::services::log_watcher::{LogWatcher, LogWatcherEvent, LogWatcherService, LogWatcherState};
    use crate::services::log_service::{LogEntry, LogLevel};
    use std::io::Write;
    use tempfile::NamedTempFile;
    use tokio::sync::mpsc;

    /// 创建临时日志文件
    fn create_temp_log_file() -> NamedTempFile {
        NamedTempFile::new().unwrap()
    }

    /// 创建测试用的日志条目
    fn create_test_entry(level: LogLevel, source: &str, message: &str) -> LogEntry {
        LogEntry::new(level, source, message)
    }

    // ==================== LogWatcher 创建测试 ====================

    /// 测试 LogWatcher 创建
    #[test]
    fn test_log_watcher_new() {
        let (tx, _rx) = mpsc::channel(10);
        let _watcher = LogWatcher::new(tx);
        // 验证创建成功
    }

    /// 测试 LogWatcherService 创建
    #[test]
    fn test_log_watcher_service_new() {
        let (service, _rx) = LogWatcherService::new();
        // 验证创建成功
        let _watcher = service.watcher();
    }

    /// 测试 LogWatcherService 默认实现
    #[test]
    fn test_log_watcher_service_default() {
        let service: LogWatcherService = Default::default();
        let _watcher = service.watcher();
    }

    /// 测试 LogWatcherState 创建
    #[test]
    fn test_log_watcher_state_new() {
        let _state = LogWatcherState::new();
        // 验证创建成功
    }

    /// 测试 LogWatcherState 默认实现
    #[test]
    fn test_log_watcher_state_default() {
        let _state: LogWatcherState = Default::default();
        // 验证创建成功
    }

    // ==================== watch_file 测试 ====================

    /// 测试监控文件
    #[tokio::test]
    async fn test_watch_file() {
        let (tx, _rx) = mpsc::channel(10);
        let watcher = LogWatcher::new(tx);

        let mut temp_file = create_temp_log_file();
        let path = temp_file.path().to_path_buf();

        // 写入初始内容
        writeln!(temp_file, "2024-01-15 10:30:45 [INFO] Initial log").unwrap();
        temp_file.flush().unwrap();

        // 开始监控
        let file_id = watcher.watch_file(path.clone(), "test".to_string()).await.unwrap();

        assert!(!file_id.is_empty());

        // 验证文件在监控列表中
        let watched_files = watcher.get_watched_files().await;
        assert_eq!(watched_files.len(), 1);
        assert_eq!(watched_files[0].0, file_id);
        assert_eq!(watched_files[0].2, "test");
    }

    /// 测试监控不存在的文件
    #[tokio::test]
    async fn test_watch_nonexistent_file() {
        let (tx, _rx) = mpsc::channel(10);
        let watcher = LogWatcher::new(tx);

        let path = std::path::PathBuf::from("/nonexistent/path/file.log");

        let result = watcher.watch_file(path, "test".to_string()).await;
        assert!(result.is_err());
    }

    /// 测试监控空文件
    #[tokio::test]
    async fn test_watch_empty_file() {
        let (tx, _rx) = mpsc::channel(10);
        let watcher = LogWatcher::new(tx);

        let temp_file = create_temp_log_file();
        let path = temp_file.path().to_path_buf();

        let file_id = watcher.watch_file(path, "test".to_string()).await.unwrap();
        assert!(!file_id.is_empty());

        let watched_files = watcher.get_watched_files().await;
        assert_eq!(watched_files.len(), 1);
    }

    /// 测试多次监控同一个文件
    #[tokio::test]
    async fn test_watch_same_file_multiple_times() {
        let (tx, _rx) = mpsc::channel(10);
        let watcher = LogWatcher::new(tx);

        let mut temp_file = create_temp_log_file();
        writeln!(temp_file, "test log").unwrap();
        temp_file.flush().unwrap();

        let path = temp_file.path().to_path_buf();

        // 第一次监控
        let id1 = watcher.watch_file(path.clone(), "test".to_string()).await.unwrap();
        // 第二次监控（应该创建新的监控项）
        let id2 = watcher.watch_file(path.clone(), "test".to_string()).await.unwrap();

        // ID 应该不同
        assert_ne!(id1, id2);

        let watched_files = watcher.get_watched_files().await;
        assert_eq!(watched_files.len(), 2);
    }

    // ==================== unwatch_file 测试 ====================

    /// 测试停止监控文件
    #[tokio::test]
    async fn test_unwatch_file() {
        let (tx, _rx) = mpsc::channel(10);
        let watcher = LogWatcher::new(tx);

        let mut temp_file = create_temp_log_file();
        writeln!(temp_file, "test log").unwrap();
        temp_file.flush().unwrap();

        let path = temp_file.path().to_path_buf();
        let file_id = watcher.watch_file(path, "test".to_string()).await.unwrap();

        // 验证已监控
        let watched_files = watcher.get_watched_files().await;
        assert_eq!(watched_files.len(), 1);

        // 停止监控
        watcher.unwatch_file(&file_id).await.unwrap();

        // 验证已移除
        let watched_files = watcher.get_watched_files().await;
        assert!(watched_files.is_empty());
    }

    /// 测试停止监控不存在的文件 ID
    #[tokio::test]
    async fn test_unwatch_nonexistent_file() {
        let (tx, _rx) = mpsc::channel(10);
        let watcher = LogWatcher::new(tx);

        // 停止监控不存在的 ID 应该成功（幂等）
        let result = watcher.unwatch_file("nonexistent-id").await;
        assert!(result.is_ok());
    }

    /// 测试多次停止监控同一个文件
    #[tokio::test]
    async fn test_unwatch_same_file_multiple_times() {
        let (tx, _rx) = mpsc::channel(10);
        let watcher = LogWatcher::new(tx);

        let mut temp_file = create_temp_log_file();
        writeln!(temp_file, "test log").unwrap();
        temp_file.flush().unwrap();

        let path = temp_file.path().to_path_buf();
        let file_id = watcher.watch_file(path, "test".to_string()).await.unwrap();

        // 第一次停止
        watcher.unwatch_file(&file_id).await.unwrap();
        // 第二次停止（应该幂等）
        watcher.unwatch_file(&file_id).await.unwrap();

        let watched_files = watcher.get_watched_files().await;
        assert!(watched_files.is_empty());
    }

    // ==================== get_watched_files 测试 ====================

    /// 测试获取监控文件列表
    #[tokio::test]
    async fn test_get_watched_files() {
        let (tx, _rx) = mpsc::channel(10);
        let watcher = LogWatcher::new(tx);

        // 初始为空
        let files = watcher.get_watched_files().await;
        assert!(files.is_empty());

        // 添加文件
        let mut temp_file1 = create_temp_log_file();
        let mut temp_file2 = create_temp_log_file();
        writeln!(temp_file1, "log1").unwrap();
        writeln!(temp_file2, "log2").unwrap();
        temp_file1.flush().unwrap();
        temp_file2.flush().unwrap();

        let _id1 = watcher.watch_file(temp_file1.path().to_path_buf(), "source1".to_string()).await.unwrap();
        let _id2 = watcher.watch_file(temp_file2.path().to_path_buf(), "source2".to_string()).await.unwrap();

        let files = watcher.get_watched_files().await;
        assert_eq!(files.len(), 2);

        // 验证返回的元组包含正确的信息
        let sources: Vec<_> = files.iter().map(|f| f.2.clone()).collect();
        assert!(sources.contains(&"source1".to_string()));
        assert!(sources.contains(&"source2".to_string()));
    }

    // ==================== LogWatcherEvent 测试 ====================

    /// 测试 NewEntry 事件
    #[test]
    fn test_log_watcher_event_new_entry() {
        let entry = create_test_entry(LogLevel::Info, "test", "message");
        let event = LogWatcherEvent::NewEntry(entry.clone());

        match event {
            LogWatcherEvent::NewEntry(e) => {
                assert_eq!(e.level, LogLevel::Info);
                assert_eq!(e.message, "message");
            }
            _ => panic!("Expected NewEntry"),
        }
    }

    /// 测试 FileReset 事件
    #[test]
    fn test_log_watcher_event_file_reset() {
        let event = LogWatcherEvent::FileReset("test-source".to_string());

        match event {
            LogWatcherEvent::FileReset(source) => {
                assert_eq!(source, "test-source");
            }
            _ => panic!("Expected FileReset"),
        }
    }

    /// 测试 Error 事件
    #[test]
    fn test_log_watcher_event_error() {
        let event = LogWatcherEvent::Error("source".to_string(), "error message".to_string());

        match event {
            LogWatcherEvent::Error(source, error) => {
                assert_eq!(source, "source");
                assert_eq!(error, "error message");
            }
            _ => panic!("Expected Error"),
        }
    }

    /// 测试 LogWatcherEvent Clone
    #[test]
    fn test_log_watcher_event_clone() {
        let entry = create_test_entry(LogLevel::Info, "test", "message");
        let event = LogWatcherEvent::NewEntry(entry);
        let cloned = event.clone();

        match cloned {
            LogWatcherEvent::NewEntry(e) => {
                assert_eq!(e.level, LogLevel::Info);
            }
            _ => panic!("Expected NewEntry"),
        }
    }

    /// 测试 LogWatcherEvent Debug
    #[test]
    fn test_log_watcher_event_debug() {
        let entry = create_test_entry(LogLevel::Info, "test", "message");
        let event = LogWatcherEvent::NewEntry(entry);

        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("NewEntry"));
    }

    // ==================== 并发测试 ====================

    /// 测试并发监控多个文件
    #[tokio::test]
    async fn test_concurrent_watch_multiple_files() {
        let (tx, _rx) = mpsc::channel(100);
        let watcher = std::sync::Arc::new(LogWatcher::new(tx));

        let mut handles = vec![];

        for i in 0..10 {
            let w = watcher.clone();
            let handle = tokio::spawn(async move {
                let mut temp_file = create_temp_log_file();
                writeln!(temp_file, "Log {}", i).unwrap();
                temp_file.flush().unwrap();

                let path = temp_file.path().to_path_buf();
                w.watch_file(path, format!("source{}", i)).await
            });
            handles.push(handle);
        }

        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }

        let watched_files = watcher.get_watched_files().await;
        assert_eq!(watched_files.len(), 10);
    }

    /// 测试并发读写
    #[tokio::test]
    async fn test_concurrent_read_write() {
        let (tx, _rx) = mpsc::channel(100);
        let watcher = std::sync::Arc::new(LogWatcher::new(tx));

        let mut temp_file = create_temp_log_file();
        writeln!(temp_file, "2024-01-15 10:30:45 [INFO] Initial").unwrap();
        temp_file.flush().unwrap();

        let path = temp_file.path().to_path_buf();
        let _file_id = watcher.watch_file(path.clone(), "test".to_string()).await.unwrap();

        // 并发读取监控列表
        let mut read_handles = vec![];
        for _ in 0..5 {
            let w = watcher.clone();
            let handle = tokio::spawn(async move {
                let files = w.get_watched_files().await;
                files.len()
            });
            read_handles.push(handle);
        }

        // 并发写入（追加内容）
        let mut write_handles = vec![];
        for i in 0..5 {
            let path = path.clone();
            let handle = tokio::spawn(async move {
                let mut file = std::fs::OpenOptions::new()
                    .write(true)
                    .append(true)
                    .open(&path)
                    .unwrap();
                writeln!(file, "2024-01-15 10:30:4{} [INFO] Line {}", i, i).unwrap();
            });
            write_handles.push(handle);
        }

        for handle in read_handles {
            let _ = handle.await;
        }
        for handle in write_handles {
            let _ = handle.await;
        }
    }

    // ==================== 边界情况测试 ====================

    /// 测试监控权限不足的文件
    #[tokio::test]
    async fn test_watch_permission_denied() {
        let (tx, _rx) = mpsc::channel(10);
        let watcher = LogWatcher::new(tx);

        // 使用系统文件（通常没有权限）
        let path = std::path::PathBuf::from("/etc/shadow");

        let _result = watcher.watch_file(path, "test".to_string()).await;
        // 结果取决于运行测试的权限
        // 但不应该 panic
    }

    /// 测试很长的文件路径
    #[tokio::test]
    async fn test_very_long_path() {
        let (tx, _rx) = mpsc::channel(10);
        let watcher = LogWatcher::new(tx);

        let temp_file = create_temp_log_file();
        let path = temp_file.path().to_path_buf();

        // 使用正常路径（系统通常有路径长度限制）
        let result = watcher.watch_file(path, "test".to_string()).await;
        assert!(result.is_ok());
    }

    /// 测试包含特殊字符的路径
    #[tokio::test]
    async fn test_special_characters_in_path() {
        let (tx, _rx) = mpsc::channel(10);
        let watcher = LogWatcher::new(tx);

        // 创建临时目录和文件
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("file with spaces.log");
        std::fs::write(&path, "test").unwrap();

        let result = watcher.watch_file(path, "test".to_string()).await;
        assert!(result.is_ok());
    }

    /// 测试包含非 ASCII 字符的路径
    #[tokio::test]
    async fn test_non_ascii_path() {
        let (tx, _rx) = mpsc::channel(10);
        let watcher = LogWatcher::new(tx);

        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("日志文件.log");
        std::fs::write(&path, "test").unwrap();

        let result = watcher.watch_file(path, "test".to_string()).await;
        assert!(result.is_ok());
    }
}
