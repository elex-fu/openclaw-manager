//! 日志文件监控服务
//!
//! 使用 notify crate 监听日志文件变化，并通过 Tauri 事件实时推送

use crate::errors::app_error::AppError;
use crate::services::log_service::{LogEntry, LogParser, LogService};
use notify::{Event, RecommendedWatcher, Watcher};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// 日志变化事件
#[derive(Debug, Clone)]
pub enum LogWatcherEvent {
    /// 新日志条目
    NewEntry(LogEntry),
    /// 文件被截断或重置
    FileReset(String),
    /// 监控错误
    Error(String, String),
}

/// 日志文件监控状态
#[derive(Debug, Clone)]
struct WatchedFile {
    path: PathBuf,
    source: String,
    last_position: u64,
    last_size: u64,
}

/// 日志文件监控器
pub struct LogWatcher {
    watched_files: Arc<RwLock<HashMap<String, WatchedFile>>>,
    parser: LogParser,
    event_sender: mpsc::Sender<LogWatcherEvent>,
    _watcher: Arc<RwLock<Option<RecommendedWatcher>>>,
}

impl LogWatcher {
    /// 创建新的日志监控器
    pub fn new(event_sender: mpsc::Sender<LogWatcherEvent>) -> Self {
        Self {
            watched_files: Arc::new(RwLock::new(HashMap::new())),
            parser: LogParser::new(),
            event_sender,
            _watcher: Arc::new(RwLock::new(None)),
        }
    }

    /// 开始监控文件
    pub async fn watch_file(
        &self,
        path: PathBuf,
        source: String,
    ) -> Result<String, AppError> {
        let file_id = uuid::Uuid::new_v4().to_string();

        // 获取文件当前大小
        let metadata = std::fs::metadata(&path).map_err(|e| AppError::Io(e))?;
        let file_size = metadata.len();

        let watched_file = WatchedFile {
            path: path.clone(),
            source: source.clone(),
            last_position: file_size, // 从当前位置开始监控新内容
            last_size: file_size,
        };

        {
            let mut files = self.watched_files.write().await;
            files.insert(file_id.clone(), watched_file);
        }

        // 如果是已存在的文件，先读取历史内容
        if file_size > 0 {
            self.read_new_content(&path, &source, 0).await?;
        }

        log::info!("开始监控日志文件: {:?} (source: {})", path, source);

        Ok(file_id)
    }

    /// 停止监控文件
    pub async fn unwatch_file(&self, file_id: &str) -> Result<(), AppError> {
        let mut files = self.watched_files.write().await;
        if files.remove(file_id).is_some() {
            log::info!("停止监控日志文件: {}", file_id);
        }
        Ok(())
    }

    /// 处理文件系统事件
    pub async fn handle_event(&self, event: Event) -> Result<(), AppError> {
        match event.kind {
            notify::EventKind::Modify(notify::event::ModifyKind::Data(_))
            | notify::EventKind::Modify(notify::event::ModifyKind::Any) => {
                for path in &event.paths {
                    self.handle_file_change(path).await?;
                }
            }
            notify::EventKind::Remove(notify::event::RemoveKind::File) => {
                for path in &event.paths {
                    self.handle_file_removal(path).await?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// 处理文件变化
    async fn handle_file_change(&self, path: &PathBuf) -> Result<(), AppError> {
        let files = self.watched_files.read().await;

        // 查找匹配的文件
        let file_info = files.iter().find(|(_, f)| f.path == *path).map(|(id, f)| (id.clone(), f.source.clone(), f.last_position, f.last_size));

        if let Some((file_id, source, last_position, last_size)) = file_info {
            drop(files); // 释放读锁

            let metadata = match std::fs::metadata(path) {
                Ok(m) => m,
                Err(e) => {
                    log::warn!("无法获取文件元数据: {:?}, 错误: {}", path, e);
                    return Ok(());
                }
            };

            let new_size = metadata.len();

            // 检查文件是否被截断
            let mut files = self.watched_files.write().await;
            if let Some(file) = files.get_mut(&file_id) {
                if new_size < file.last_size {
                    // 文件被截断或重置
                    log::info!("日志文件被截断: {:?}", path);
                    file.last_position = 0;
                    let _ = self
                        .event_sender
                        .send(LogWatcherEvent::FileReset(file.source.clone()))
                        .await;
                }
                file.last_size = new_size;
            }
            drop(files);

            // 读取新内容
            self.read_new_content(path, &source, last_position)
                .await?;

            // 更新位置
            let mut files = self.watched_files.write().await;
            if let Some(file) = files.get_mut(&file_id) {
                file.last_position = new_size;
            }
        }

        Ok(())
    }

    /// 处理文件删除
    async fn handle_file_removal(&self, path: &PathBuf) -> Result<(), AppError> {
        let mut files = self.watched_files.write().await;
        let to_remove: Vec<String> = files
            .iter()
            .filter(|(_, f)| f.path == *path)
            .map(|(id, _)| id.clone())
            .collect();

        for id in to_remove {
            log::info!("日志文件被删除，停止监控: {:?}", path);
            files.remove(&id);
        }

        Ok(())
    }

    /// 读取文件新内容
    async fn read_new_content(
        &self,
        path: &PathBuf,
        source: &str,
        from_position: u64,
    ) -> Result<(), AppError> {
        use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};

        let file = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(e) => {
                log::warn!("无法打开日志文件: {:?}, 错误: {}", path, e);
                return Ok(());
            }
        };

        let mut reader = BufReader::new(file);
        reader
            .seek(SeekFrom::Start(from_position))
            .map_err(|e| AppError::Io(e))?;

        let mut buffer = String::new();
        reader
            .read_to_string(&mut buffer)
            .map_err(|e| AppError::Io(e))?;

        // 解析每一行并发送事件
        for line in buffer.lines() {
            if line.trim().is_empty() {
                continue;
            }

            if let Some(entry) = self.parser.parse_line(line, source) {
                let _ = self
                    .event_sender
                    .send(LogWatcherEvent::NewEntry(entry))
                    .await;
            }
        }

        Ok(())
    }

    /// 获取监控的文件列表
    pub async fn get_watched_files(&self) -> Vec<(String, PathBuf, String)> {
        let files = self.watched_files.read().await;
        files
            .iter()
            .map(|(id, f)| (id.clone(), f.path.clone(), f.source.clone()))
            .collect()
    }
}

/// 启动文件系统监控
pub async fn start_file_watcher(
    log_watcher: Arc<LogWatcher>,
) -> Result<RecommendedWatcher, AppError> {
    let (tx, mut rx) = mpsc::channel(100);

    // 创建 notify watcher
    let watcher_tx = tx.clone();
    let watcher = notify::recommended_watcher(move |res| {
        let _ = watcher_tx.try_send(res);
    })
    .map_err(|e| AppError::Unknown(format!("Failed to create file watcher: {}", e)))?;

    // 启动处理任务
    let watcher_clone = log_watcher.clone();
    tokio::spawn(async move {
        while let Some(result) = rx.recv().await {
            match result {
                Ok(event) => {
                    if let Err(e) = watcher_clone.handle_event(event).await {
                        log::error!("处理文件事件失败: {:?}", e);
                    }
                }
                Err(e) => {
                    log::error!("文件监控错误: {:?}", e);
                }
            }
        }
    });

    Ok(watcher)
}

/// 日志监控服务
pub struct LogWatcherService {
    log_watcher: Arc<LogWatcher>,
    event_receiver: Arc<RwLock<mpsc::Receiver<LogWatcherEvent>>>,
}

impl LogWatcherService {
    /// 创建新的监控服务
    pub fn new() -> (Self, mpsc::Receiver<LogWatcherEvent>) {
        let (tx, rx) = mpsc::channel(1000);
        let log_watcher = Arc::new(LogWatcher::new(tx));

        // 创建一个dummy receiver用于service结构体
        let (_, dummy_rx) = mpsc::channel(1);

        (
            Self {
                log_watcher,
                event_receiver: Arc::new(RwLock::new(dummy_rx)),
            },
            rx,
        )
    }

    /// 获取监控器引用
    pub fn watcher(&self) -> Arc<LogWatcher> {
        self.log_watcher.clone()
    }

    /// 启动监控服务
    pub async fn start(
        &self,
        log_service: &LogService,
    ) -> Result<RecommendedWatcher, AppError> {
        // 扫描并监控所有已注册的日志文件
        let files = log_service.get_log_files().await;

        for file_info in files {
            let path = PathBuf::from(&file_info.path);
            self.log_watcher
                .watch_file(path, file_info.source)
                .await?;
        }

        // 启动文件系统监控
        let watcher = start_file_watcher(self.log_watcher.clone()).await?;

        Ok(watcher)
    }
}

impl Default for LogWatcherService {
    fn default() -> Self {
        let (service, _) = Self::new();
        service
    }
}

/// 日志监控状态（用于 Tauri 状态管理）
pub struct LogWatcherState {
    pub service: Arc<RwLock<LogWatcherService>>,
    pub file_watcher: Arc<RwLock<Option<RecommendedWatcher>>>,
}

impl LogWatcherState {
    pub fn new() -> Self {
        let (service, _) = LogWatcherService::new();
        Self {
            service: Arc::new(RwLock::new(service)),
            file_watcher: Arc::new(RwLock::new(None)),
        }
    }
}

impl Default for LogWatcherState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_log_watcher() {
        let (tx, mut rx) = mpsc::channel(10);
        let watcher = LogWatcher::new(tx);

        // 创建临时文件
        let mut temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        // 写入一些初始内容
        writeln!(temp_file, "2024-01-15 10:30:45 [INFO] Initial log").unwrap();
        temp_file.flush().unwrap();

        // 开始监控
        let file_id = watcher.watch_file(path.clone(), "test".to_string()).await.unwrap();

        // 模拟文件变化
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(&path)
            .unwrap();
        writeln!(file, "2024-01-15 10:30:46 [ERROR] New error log").unwrap();
        drop(file);

        // 手动触发文件变化处理
        let _metadata = std::fs::metadata(&path).unwrap();
        watcher.read_new_content(&path, "test", 0).await.unwrap();

        // 停止监控
        watcher.unwatch_file(&file_id).await.unwrap();
    }
}
