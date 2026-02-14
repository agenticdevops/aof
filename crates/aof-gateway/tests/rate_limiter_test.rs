//! Rate limiter tests

use std::time::Duration;
use aof_gateway::{RateLimiter, RateLimitConfig, Platform};

#[tokio::test]
async fn test_slack_rate_limiter_timing() {
    let config = RateLimitConfig {
        requests_per_second: 1,
        burst_size: 1,
    };
    let limiter = RateLimiter::new(Platform::Slack, config);

    // First request should succeed immediately
    let start = std::time::Instant::now();
    limiter.acquire().await.unwrap();
    let elapsed = start.elapsed();

    // Should be nearly instant (< 100ms)
    assert!(elapsed < Duration::from_millis(100));

    // Second request should block for ~1 second
    let start = std::time::Instant::now();
    limiter.acquire().await.unwrap();
    let elapsed = start.elapsed();

    // Should take at least 800ms (allow some tolerance)
    assert!(elapsed >= Duration::from_millis(800));
}

#[tokio::test]
async fn test_burst_allowance() {
    let config = RateLimitConfig {
        requests_per_second: 1,
        burst_size: 5,
    };
    let limiter = RateLimiter::new(Platform::Discord, config);

    // First 5 requests should succeed rapidly (burst)
    let start = std::time::Instant::now();
    for _ in 0..5 {
        limiter.acquire().await.unwrap();
    }
    let elapsed = start.elapsed();

    // All 5 should complete in < 500ms (burst mode)
    assert!(elapsed < Duration::from_millis(500));

    // 6th request should block
    let start = std::time::Instant::now();
    limiter.acquire().await.unwrap();
    let elapsed = start.elapsed();

    // Should take at least 800ms (rate limit kicks in)
    assert!(elapsed >= Duration::from_millis(800));
}

#[test]
fn test_check_non_blocking() {
    let config = RateLimitConfig {
        requests_per_second: 1,
        burst_size: 1,
    };
    let limiter = RateLimiter::new(Platform::Slack, config);

    // First check should succeed
    assert!(limiter.check().is_ok());

    // Second check should fail immediately (no blocking)
    let start = std::time::Instant::now();
    let result = limiter.check();
    let elapsed = start.elapsed();

    assert!(result.is_err());
    // Should return immediately (< 10ms)
    assert!(elapsed < Duration::from_millis(10));
}

#[test]
fn test_rate_limiter_stats() {
    let config = RateLimitConfig {
        requests_per_second: 10,
        burst_size: 20,
    };
    let limiter = RateLimiter::new(Platform::Discord, config);

    let stats = limiter.stats();
    assert_eq!(stats.platform, Platform::Discord);
    assert_eq!(stats.requests_per_second, 10);
    assert_eq!(stats.burst_size, 20);
}
