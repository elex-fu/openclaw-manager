use crate::errors::app_error::ApiResponse;
use serde::{Deserialize, Serialize};
use sysinfo::System;

/// System resource information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResources {
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub disk: DiskInfo,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    /// CPU usage percentage (0-100)
    pub usage: f32,
    /// Number of CPU cores
    pub cores: usize,
    /// CPU name
    pub name: String,
    /// CPU frequency in MHz
    pub frequency: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    /// Used memory in MB
    pub used: u64,
    /// Total memory in MB
    pub total: u64,
    /// Memory usage percentage (0-100)
    pub usage: f32,
    /// Available memory in MB
    pub available: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    /// Used disk space in GB
    pub used: u64,
    /// Total disk space in GB
    pub total: u64,
    /// Disk usage percentage (0-100)
    pub usage: f32,
    /// Free disk space in GB
    pub free: u64,
}

/// Get current system resource usage
#[tauri::command]
pub fn get_system_resources() -> ApiResponse<SystemResources> {
    let mut sys = System::new_all();
    sys.refresh_all();

    // Wait a bit for CPU usage calculation
    std::thread::sleep(std::time::Duration::from_millis(100));
    sys.refresh_cpu();

    // CPU information
    let cpu_cores = sys.cpus().len();
    let cpu_name = sys.cpus().first().map(|c| c.brand().to_string()).unwrap_or_default();
    let cpu_frequency = sys.cpus().first().map(|c| c.frequency()).unwrap_or(0);

    // Calculate average CPU usage across all cores
    let cpu_usage: f32 = if cpu_cores > 0 {
        sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / cpu_cores as f32
    } else {
        0.0
    };

    // Memory information (convert from KB to MB)
    let total_memory = sys.total_memory() / 1024;
    let used_memory = sys.used_memory() / 1024;
    let available_memory = sys.available_memory() / 1024;
    let memory_usage = if total_memory > 0 {
        (used_memory as f32 / total_memory as f32) * 100.0
    } else {
        0.0
    };

    // Disk information (find the disk with most space, typically the main disk)
    let mut total_disk: u64 = 0;
    let mut used_disk: u64 = 0;
    let mut free_disk: u64 = 0;

    let disks = sysinfo::Disks::new_with_refreshed_list();
    for disk in &disks {
        let disk_total = disk.total_space();
        let disk_available = disk.available_space();
        let disk_used = disk_total.saturating_sub(disk_available);

        // Use the largest disk (usually the main system disk)
        if disk_total > total_disk {
            total_disk = disk_total;
            used_disk = disk_used;
            free_disk = disk_available;
        }
    }

    // Convert bytes to GB
    let total_disk_gb = total_disk / (1024 * 1024 * 1024);
    let used_disk_gb = used_disk / (1024 * 1024 * 1024);
    let free_disk_gb = free_disk / (1024 * 1024 * 1024);

    let disk_usage = if total_disk > 0 {
        (used_disk as f32 / total_disk as f32) * 100.0
    } else {
        0.0
    };

    let resources = SystemResources {
        cpu: CpuInfo {
            usage: cpu_usage,
            cores: cpu_cores,
            name: cpu_name,
            frequency: cpu_frequency,
        },
        memory: MemoryInfo {
            used: used_memory,
            total: total_memory,
            usage: memory_usage,
            available: available_memory,
        },
        disk: DiskInfo {
            used: used_disk_gb,
            total: total_disk_gb,
            usage: disk_usage,
            free: free_disk_gb,
        },
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    };

    ApiResponse::success(resources)
}

/// Get CPU usage history (for charting)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuHistoryPoint {
    pub timestamp: u64,
    pub usage: f32,
}

#[tauri::command]
pub fn get_cpu_history() -> ApiResponse<Vec<CpuHistoryPoint>> {
    let mut sys = System::new_all();
    let mut history = Vec::new();

    // Collect 60 data points over 30 seconds (one every 500ms)
    for i in 0..60 {
        sys.refresh_cpu();
        let usage = sys.global_cpu_info().cpu_usage();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        history.push(CpuHistoryPoint {
            timestamp,
            usage,
        });

        if i < 59 {
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }

    ApiResponse::success(history)
}

/// Activity log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: String,
    pub timestamp: u64,
    pub activity_type: String, // 'install' | 'config' | 'service' | 'error'
    pub message: String,
    pub details: Option<String>,
}

/// Get recent activities (mock implementation - in production this would come from a database)
#[tauri::command]
pub fn get_recent_activities(limit: Option<u32>) -> ApiResponse<Vec<Activity>> {
    // This is a mock implementation
    // In a real implementation, this would read from a log database or file
    let activities = vec![
        Activity {
            id: "1".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() - 60,
            activity_type: "service".to_string(),
            message: "Gateway 服务已启动".to_string(),
            details: Some("服务PID: 12345".to_string()),
        },
        Activity {
            id: "2".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() - 300,
            activity_type: "config".to_string(),
            message: "模型配置已更新".to_string(),
            details: Some("模型: GPT-4".to_string()),
        },
        Activity {
            id: "3".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() - 600,
            activity_type: "install".to_string(),
            message: "OpenClaw 安装完成".to_string(),
            details: Some("版本: 0.1.0".to_string()),
        },
    ];

    let limit = limit.unwrap_or(10) as usize;
    let limited_activities: Vec<Activity> = activities.into_iter().take(limit).collect();

    ApiResponse::success(limited_activities)
}

/// Diagnostic alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticAlert {
    pub id: String,
    pub severity: String, // 'info' | 'warning' | 'error'
    pub title: String,
    pub message: String,
    pub fixable: bool,
    pub category: String,
}

/// Get diagnostic alerts
#[tauri::command]
pub fn get_diagnostic_alerts() -> ApiResponse<Vec<DiagnosticAlert>> {
    // This is a mock implementation
    // In a real implementation, this would check various system conditions
    let alerts = vec![
        DiagnosticAlert {
            id: "1".to_string(),
            severity: "warning".to_string(),
            title: "磁盘空间不足".to_string(),
            message: "磁盘使用率超过 80%，建议清理空间".to_string(),
            fixable: false,
            category: "system".to_string(),
        },
        DiagnosticAlert {
            id: "2".to_string(),
            severity: "info".to_string(),
            title: "配置备份提醒".to_string(),
            message: "建议定期备份配置以防数据丢失".to_string(),
            fixable: true,
            category: "config".to_string(),
        },
    ];

    ApiResponse::success(alerts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_system_resources() {
        let response = get_system_resources();
        assert!(response.success);
        assert!(response.data.is_some());

        let resources = response.data.unwrap();
        assert!(resources.cpu.cores > 0);
        assert!(resources.memory.total > 0);
        assert!(resources.timestamp > 0);
    }

    #[test]
    fn test_get_recent_activities() {
        let response = get_recent_activities(Some(5));
        assert!(response.success);
        assert!(response.data.is_some());
        assert!(!response.data.unwrap().is_empty());
    }

    #[test]
    fn test_get_diagnostic_alerts() {
        let response = get_diagnostic_alerts();
        assert!(response.success);
        assert!(response.data.is_some());
    }
}
