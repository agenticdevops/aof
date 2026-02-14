use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use rand::Rng;

/// Retry policy with exponential backoff
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum retry attempts (default: 5)
    pub max_attempts: usize,
    /// Base delay (default: 1 second)
    pub base_delay: Duration,
    /// Maximum delay (default: 60 seconds)
    pub max_delay: Duration,
    /// Add jitter to delays (default: true)
    pub jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            jitter: true,
        }
    }
}

impl RetryPolicy {
    /// Calculate delay for a given attempt number (0-indexed)
    pub fn delay_for_attempt(&self, attempt: usize) -> Duration {
        if attempt == 0 {
            return Duration::ZERO;
        }

        // Exponential backoff: base_delay * 2^attempt
        let exponential_delay = self.base_delay * 2_u32.pow(attempt as u32);

        // Cap at max_delay
        let capped_delay = exponential_delay.min(self.max_delay);

        // Add jitter if enabled
        if self.jitter {
            let jitter_amount = rand::thread_rng().gen_range(0..=10); // 0-10% jitter
            let jitter_multiplier = 1.0 + (jitter_amount as f64 / 100.0);
            let with_jitter = capped_delay.mul_f64(jitter_multiplier);
            with_jitter.min(self.max_delay)
        } else {
            capped_delay
        }
    }

    /// Execute operation with retry policy
    pub async fn execute<F, T, E>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> Pin<Box<dyn Future<Output = Result<T, E>> + Send>> + Send,
    {
        let mut attempt = 0;

        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(err) => {
                    attempt += 1;

                    if attempt >= self.max_attempts {
                        return Err(err);
                    }

                    let delay = self.delay_for_attempt(attempt);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_retry_zero_failures_no_delay() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.delay_for_attempt(0), Duration::ZERO);
    }

    #[test]
    fn test_retry_exponential_growth() {
        let policy = RetryPolicy {
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(100),
            jitter: false,
            ..Default::default()
        };

        // Attempt 1: 1 * 2^1 = 2s
        assert_eq!(policy.delay_for_attempt(1), Duration::from_secs(2));

        // Attempt 2: 1 * 2^2 = 4s
        assert_eq!(policy.delay_for_attempt(2), Duration::from_secs(4));

        // Attempt 3: 1 * 2^3 = 8s
        assert_eq!(policy.delay_for_attempt(3), Duration::from_secs(8));

        // Attempt 4: 1 * 2^4 = 16s
        assert_eq!(policy.delay_for_attempt(4), Duration::from_secs(16));
    }

    #[test]
    fn test_retry_capped_at_max_delay() {
        let policy = RetryPolicy {
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(10),
            jitter: false,
            ..Default::default()
        };

        // Attempt 4: 1 * 2^4 = 16s, but capped at 10s
        assert_eq!(policy.delay_for_attempt(4), Duration::from_secs(10));

        // Attempt 10: would be huge, but capped at 10s
        assert_eq!(policy.delay_for_attempt(10), Duration::from_secs(10));
    }

    #[test]
    fn test_retry_jitter_adds_variance() {
        let policy = RetryPolicy {
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(100),
            jitter: true,
            ..Default::default()
        };

        // With jitter, delays should vary slightly
        let delays: Vec<Duration> = (0..5)
            .map(|_| policy.delay_for_attempt(3))
            .collect();

        // All should be >= 8s and <= 8.8s (8s * 1.1)
        for delay in delays {
            assert!(delay >= Duration::from_secs(8));
            assert!(delay <= Duration::from_millis(8800));
        }
    }

    #[tokio::test]
    async fn test_retry_execute_succeeds_immediately() {
        let policy = RetryPolicy::default();
        let counter = Arc::new(AtomicUsize::new(0));

        let result = policy
            .execute(|| {
                let c = counter.clone();
                Box::pin(async move {
                    c.fetch_add(1, Ordering::Relaxed);
                    Ok::<_, &str>(42)
                })
            })
            .await;

        assert_eq!(result, Ok(42));
        assert_eq!(counter.load(Ordering::Relaxed), 1); // Only called once
    }

    #[tokio::test]
    async fn test_retry_execute_retries_on_failure() {
        let policy = RetryPolicy {
            max_attempts: 3,
            base_delay: Duration::from_millis(10),
            jitter: false,
            ..Default::default()
        };
        let counter = Arc::new(AtomicUsize::new(0));

        let result = policy
            .execute(|| {
                let c = counter.clone();
                Box::pin(async move {
                    let count = c.fetch_add(1, Ordering::Relaxed);
                    if count < 2 {
                        Err("not yet")
                    } else {
                        Ok(42)
                    }
                })
            })
            .await;

        assert_eq!(result, Ok(42));
        assert_eq!(counter.load(Ordering::Relaxed), 3); // Called 3 times
    }

    #[tokio::test]
    async fn test_retry_execute_fails_after_max_attempts() {
        let policy = RetryPolicy {
            max_attempts: 3,
            base_delay: Duration::from_millis(10),
            jitter: false,
            ..Default::default()
        };
        let counter = Arc::new(AtomicUsize::new(0));

        let result = policy
            .execute(|| {
                let c = counter.clone();
                Box::pin(async move {
                    c.fetch_add(1, Ordering::Relaxed);
                    Err::<i32, _>("always fails")
                })
            })
            .await;

        assert_eq!(result, Err("always fails"));
        assert_eq!(counter.load(Ordering::Relaxed), 3); // Max attempts
    }
}
