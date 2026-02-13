//! Retry logic with exponential backoff for rate limit errors (429)

use std::future::Future;
use std::time::Duration;
use tracing::warn;

use aof_core::AofError;

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: usize,
    /// Base delay (will be multiplied by 2^attempt)
    pub base_delay_ms: u64,
    /// Add jitter to prevent thundering herd
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000, // 1 second base
            jitter: true,
        }
    }
}

/// Execute operation with retry logic for 429 errors
pub async fn retry_with_backoff<F, Fut, T>(
    operation: F,
    config: RetryConfig,
    adapter_id: &str,
) -> Result<T, AofError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<(T, Option<u64>), AofError>>,
{
    for attempt in 0..=config.max_retries {
        match operation().await {
            Ok((result, _)) => return Ok(result),
            Err(e) => {
                // Check if error is rate limit (429) or transient
                let is_rate_limit = e.to_string().contains("429") || e.to_string().contains("rate limit");
                let is_transient = e.to_string().contains("network") || e.to_string().contains("timeout");

                if !is_rate_limit && !is_transient {
                    // Non-retryable error, fail immediately
                    return Err(e);
                }

                if attempt >= config.max_retries {
                    // Exhausted retries
                    return Err(AofError::runtime(format!(
                        "Failed after {} retries: {}",
                        config.max_retries, e
                    )));
                }

                // Calculate backoff delay
                let retry_after = if is_rate_limit {
                    // Try to extract Retry-After from error message
                    extract_retry_after(&e.to_string()).unwrap_or(1) // Default to 1 sec if not found
                } else {
                    // Exponential backoff for transient errors (in milliseconds)
                    let delay_ms = config.base_delay_ms * 2_u64.pow(attempt as u32);
                    std::cmp::max(delay_ms / 1000, 1) // At least 1 second
                };

                // Add jitter if enabled
                let delay_secs = if config.jitter {
                    let jitter_ms = rand::random::<u64>() % 1000;
                    retry_after + (jitter_ms / 1000)
                } else {
                    retry_after
                };

                warn!(
                    adapter_id = %adapter_id,
                    attempt = attempt + 1,
                    max_retries = config.max_retries,
                    delay_secs = delay_secs,
                    error = %e,
                    "Retrying after error"
                );

                tokio::time::sleep(Duration::from_secs(delay_secs)).await;
            }
        }
    }

    Err(AofError::runtime("Retry logic error"))
}

/// Extract Retry-After value from error message
fn extract_retry_after(error_msg: &str) -> Option<u64> {
    // Try to parse "Retry-After: <seconds>" from error message
    if let Some(start) = error_msg.find("Retry-After:") {
        let rest = &error_msg[start + 12..];
        if let Some(end) = rest.find(|c: char| !c.is_numeric()) {
            rest[..end].parse::<u64>().ok()
        } else {
            rest.parse::<u64>().ok()
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.base_delay_ms, 1000);
        assert!(config.jitter);
    }

    #[test]
    fn test_extract_retry_after() {
        assert_eq!(extract_retry_after("Retry-After: 60"), Some(60));
        assert_eq!(extract_retry_after("Error: Retry-After: 30 seconds"), Some(30));
        assert_eq!(extract_retry_after("No retry header"), None);
    }

    #[tokio::test]
    async fn test_retry_with_backoff_success() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        let operation = move || {
            let count = call_count_clone.clone();
            async move {
                let current = count.fetch_add(1, Ordering::SeqCst);
                if current == 0 {
                    Err(AofError::runtime("429 rate limit"))
                } else {
                    Ok(("success".to_string(), None))
                }
            }
        };

        let config = RetryConfig {
            max_retries: 3,
            base_delay_ms: 1, // 1ms base delay for fast tests
            jitter: false,
        };

        let result = retry_with_backoff(operation, config, "test-adapter").await;
        assert!(result.is_ok());
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_retry_with_backoff_exhausted() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        let operation = move || {
            let count = call_count_clone.clone();
            async move {
                count.fetch_add(1, Ordering::SeqCst);
                Err::<(String, Option<u64>), _>(AofError::runtime("429 rate limit"))
            }
        };

        let config = RetryConfig {
            max_retries: 2,
            base_delay_ms: 1, // 1ms base delay for fast tests
            jitter: false,
        };

        let result = retry_with_backoff(operation, config, "test-adapter").await;
        assert!(result.is_err());
        assert!(call_count.load(Ordering::SeqCst) >= 2);
    }
}
