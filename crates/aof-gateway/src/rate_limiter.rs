//! Rate limiting abstraction using governor crate
//!
//! This module provides a rate limiting abstraction that uses the GCRA (Generic Cell Rate Algorithm)
//! token bucket implementation from the governor crate.

use std::num::NonZeroU32;

use governor::{Quota, RateLimiter as GovernorRateLimiter};
use governor::state::{direct::NotKeyed, InMemoryState};
use governor::clock::DefaultClock;
use serde::{Deserialize, Serialize};

use aof_core::AofError;
use crate::adapters::Platform;

/// Rate limiter for a specific platform
pub struct RateLimiter {
    limiter: GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>,
    platform: Platform,
    config: RateLimitConfig,
}

/// Rate limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per second allowed
    pub requests_per_second: u32,
    /// Burst size (maximum tokens)
    pub burst_size: u32,
}

/// Rate limit statistics
#[derive(Debug, Clone)]
pub struct RateLimitStats {
    /// Platform this limiter handles
    pub platform: Platform,
    /// Configured requests per second
    pub requests_per_second: u32,
    /// Configured burst size
    pub burst_size: u32,
}

impl RateLimiter {
    /// Create rate limiter for platform with specific config
    pub fn new(platform: Platform, config: RateLimitConfig) -> Self {
        let quota = Quota::per_second(
            NonZeroU32::new(config.requests_per_second).unwrap_or(NonZeroU32::new(1).unwrap())
        ).allow_burst(
            NonZeroU32::new(config.burst_size).unwrap_or(NonZeroU32::new(1).unwrap())
        );

        let limiter = GovernorRateLimiter::direct(quota);

        Self {
            limiter,
            platform,
            config,
        }
    }

    /// Wait until rate limiter allows (async, non-blocking)
    pub async fn acquire(&self) -> Result<(), AofError> {
        self.limiter.until_ready().await;
        Ok(())
    }

    /// Check if token available without blocking (returns Err if exhausted)
    pub fn check(&self) -> Result<(), AofError> {
        match self.limiter.check() {
            Ok(_) => Ok(()),
            Err(_) => Err(AofError::runtime("Rate limit exhausted")),
        }
    }

    /// Get current rate limit stats (for monitoring)
    pub fn stats(&self) -> RateLimitStats {
        RateLimitStats {
            platform: self.platform,
            requests_per_second: self.config.requests_per_second,
            burst_size: self.config.burst_size,
        }
    }

    /// Get default config for a platform
    pub fn default_config_for_platform(platform: Platform) -> RateLimitConfig {
        match platform {
            Platform::Slack => RateLimitConfig {
                requests_per_second: 1,
                burst_size: 5,
            },
            Platform::Discord => RateLimitConfig {
                requests_per_second: 10,
                burst_size: 20,
            },
            Platform::Telegram => RateLimitConfig {
                requests_per_second: 30,
                burst_size: 50,
            },
            Platform::WhatsApp => RateLimitConfig {
                // 1000 messages/day ≈ 0.01 msg/sec, round up to 1
                requests_per_second: 1,
                burst_size: 10,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_acquire() {
        let config = RateLimitConfig {
            requests_per_second: 10,
            burst_size: 5,
        };
        let limiter = RateLimiter::new(Platform::Discord, config);

        // Should allow immediate acquisition
        assert!(limiter.acquire().await.is_ok());
    }

    #[test]
    fn test_rate_limiter_check() {
        let config = RateLimitConfig {
            requests_per_second: 10,
            burst_size: 5,
        };
        let limiter = RateLimiter::new(Platform::Discord, config);

        // Should have tokens available
        assert!(limiter.check().is_ok());
    }

    #[test]
    fn test_default_configs() {
        let slack_config = RateLimiter::default_config_for_platform(Platform::Slack);
        assert_eq!(slack_config.requests_per_second, 1);
        assert_eq!(slack_config.burst_size, 5);

        let discord_config = RateLimiter::default_config_for_platform(Platform::Discord);
        assert_eq!(discord_config.requests_per_second, 10);
        assert_eq!(discord_config.burst_size, 20);
    }
}
