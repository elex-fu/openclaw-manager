//! 重试机制
//!
//! 提供带指数退避的重试功能

#![allow(dead_code)]

use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;

/// 重试配置
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// 最大重试次数
    pub max_attempts: u32,
    /// 初始延迟
    pub initial_delay: Duration,
    /// 最大延迟
    pub max_delay: Duration,
    /// 退避倍数
    pub backoff_multiplier: f64,
    /// 是否启用抖动（随机延迟）
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryConfig {
    /// 快速重试配置（较少延迟）
    pub fn fast() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 1.5,
            jitter: false,
        }
    }

    /// 慢速重试配置（较长延迟，适合网络请求）
    pub fn slow() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

/// 带指数退避的重试异步操作
pub async fn retry_with_backoff<F, Fut, T, E>(
    config: RetryConfig,
    operation: F,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut delay = config.initial_delay;
    let mut last_error = None;

    for attempt in 1..=config.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                log::warn!("Attempt {} failed: {:?}", attempt, e);
                last_error = Some(e);

                if attempt == config.max_attempts {
                    break;
                }

                // 计算实际延迟（考虑抖动）
                let actual_delay = if config.jitter {
                    add_jitter(delay)
                } else {
                    delay
                };

                log::debug!("Retrying after {:?} delay...", actual_delay);
                sleep(actual_delay).await;

                // 指数退避
                delay = std::cmp::min(
                    Duration::from_millis(
                        (delay.as_millis() as f64 * config.backoff_multiplier) as u64
                    ),
                    config.max_delay,
                );
            }
        }
    }

    // 所有重试都失败了
    Err(last_error.unwrap())
}

/// 添加抖动（随机延迟）
fn add_jitter(delay: Duration) -> Duration {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};

    // 基于当前时间生成随机种子
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let mut hasher = DefaultHasher::new();
    nanos.hash(&mut hasher);
    let hash = hasher.finish();

    // 抖动范围：delay * 0.5 到 delay * 1.5
    let jitter_factor = 0.5 + (hash as f64 / u64::MAX as f64);

    Duration::from_millis((delay.as_millis() as f64 * jitter_factor) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_retry_success() {
        let counter = AtomicU32::new(0);

        let result: Result<&str, anyhow::Error> = retry_with_backoff(RetryConfig::fast(), || async {
            let count = counter.fetch_add(1, Ordering::SeqCst);
            if count < 2 {
                Err(anyhow::anyhow!("not yet"))
            } else {
                Ok("success")
            }
        }).await;

        assert!(result.is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_exhausted() {
        let config = RetryConfig {
            max_attempts: 2,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            jitter: false,
        };

        let counter = AtomicU32::new(0);

        let result = retry_with_backoff(config, || async {
            counter.fetch_add(1, Ordering::SeqCst);
            Err::<(), _>("always fails")
        }).await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_jitter() {
        let base_delay = Duration::from_secs(1);
        let jittered = add_jitter(base_delay);

        // 抖动后的延迟应在 0.5s 到 1.5s 之间
        assert!(jittered >= Duration::from_millis(500));
        assert!(jittered <= Duration::from_millis(1500));
    }
}
