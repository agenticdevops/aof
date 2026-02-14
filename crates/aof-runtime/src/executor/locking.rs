//! Resource locking for serializing destructive operations
//!
//! This module provides distributed resource locking via Redis with TTL-based auto-expiry
//! and file-based fallback for development/testing environments.
//!
//! # Redis-based Locking
//!
//! Uses Redis SET NX EX (atomic set-if-not-exists with expiry) and Lua scripts for
//! ownership verification on extend/release operations.
//!
//! # File-based Fallback
//!
//! When Redis is unavailable, uses file-based locks stored in configurable directory
//! with TTL tracked in lock file content.

use aof_core::error::AofError;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::fs;
use tokio::time::sleep;
use redis::aio::Connection;
use redis::{AsyncCommands, Client, RedisError};

/// Configuration for lock management
#[derive(Clone, Debug)]
pub struct LockConfig {
    /// Redis URL (optional; if None, uses file-based fallback)
    pub redis_url: Option<String>,
    /// Directory for file-based locks (fallback)
    pub lock_dir: Option<PathBuf>,
    /// Default TTL for locks (seconds)
    pub ttl: u64,
    /// Default timeout for acquire_with_wait (seconds)
    pub timeout: u64,
}

impl Default for LockConfig {
    fn default() -> Self {
        Self {
            redis_url: Some("redis://localhost:6379".to_string()),
            lock_dir: Some(PathBuf::from("/tmp/aof-locks")),
            ttl: 30,
            timeout: 60,
        }
    }
}

/// Redis-based resource lock
pub struct ResourceLock {
    client: Arc<Client>,
    resource_id: String,
    agent_id: String,
    ttl: u64,
    timeout: u64,
}

impl ResourceLock {
    /// Create a new Redis-based lock
    pub async fn new(
        client: Arc<Client>,
        resource_id: impl Into<String>,
        agent_id: impl Into<String>,
        ttl: u64,
        timeout: u64,
    ) -> Result<Self, AofError> {
        Ok(Self {
            client,
            resource_id: resource_id.into(),
            agent_id: agent_id.into(),
            ttl,
            timeout,
        })
    }

    /// Acquire lock immediately (non-blocking)
    /// Returns true if acquired, false if already locked
    pub async fn acquire(&self) -> Result<bool, AofError> {
        let key = format!("aof:lock:{}", self.resource_id);
        let value = self.agent_id.clone();
        let ttl_secs = self.ttl as usize;

        let mut conn = self.client.get_async_connection()
            .await
            .map_err(|e| AofError::lock_failed(format!("Redis connection failed: {}", e)))?;

        let result: bool = redis::cmd("SET")
            .arg(&key)
            .arg(&value)
            .arg("NX")
            .arg("EX")
            .arg(ttl_secs)
            .query_async(&mut conn)
            .await
            .map_err(|e| AofError::lock_failed(format!("SET NX EX failed: {}", e)))?;

        Ok(result)
    }

    /// Extend lock TTL (verify ownership first)
    /// Returns true if extended, false if not owner
    pub async fn extend(&self) -> Result<bool, AofError> {
        let key = format!("aof:lock:{}", self.resource_id);
        let value = self.agent_id.clone();
        let ttl_secs = self.ttl as usize;

        let lua_script = redis::Script::new(
            r#"
            if redis.call("GET", KEYS[1]) == ARGV[1] then
                return redis.call("EXPIRE", KEYS[1], ARGV[2])
            else
                return 0
            end
            "#,
        );

        let mut conn = self.client.get_async_connection()
            .await
            .map_err(|e| AofError::lock_failed(format!("Redis connection failed: {}", e)))?;

        let result: i32 = lua_script
            .key(&key)
            .arg(&value)
            .arg(ttl_secs)
            .invoke_async(&mut conn)
            .await
            .map_err(|e| AofError::lock_failed(format!("EXPIRE script failed: {}", e)))?;

        Ok(result == 1)
    }

    /// Release lock (verify ownership first)
    /// Returns true if released, false if not owner
    pub async fn release(&self) -> Result<bool, AofError> {
        let key = format!("aof:lock:{}", self.resource_id);
        let value = self.agent_id.clone();

        let lua_script = redis::Script::new(
            r#"
            if redis.call("GET", KEYS[1]) == ARGV[1] then
                return redis.call("DEL", KEYS[1])
            else
                return 0
            end
            "#,
        );

        let mut conn = self.client.get_async_connection()
            .await
            .map_err(|e| AofError::lock_failed(format!("Redis connection failed: {}", e)))?;

        let result: i32 = lua_script
            .key(&key)
            .arg(&value)
            .invoke_async(&mut conn)
            .await
            .map_err(|e| AofError::lock_failed(format!("DEL script failed: {}", e)))?;

        Ok(result == 1)
    }

    /// Acquire lock with blocking wait
    /// Returns true if acquired, false if timeout
    pub async fn acquire_with_wait(&self) -> Result<bool, AofError> {
        let start = SystemTime::now();
        let timeout_duration = Duration::from_secs(self.timeout);

        loop {
            if self.acquire().await? {
                return Ok(true);
            }

            if start.elapsed().unwrap_or_default() > timeout_duration {
                return Ok(false);
            }

            sleep(Duration::from_millis(100)).await;
        }
    }

    /// Check if lock exists (for any owner)
    pub async fn is_locked(&self) -> Result<bool, AofError> {
        let key = format!("aof:lock:{}", self.resource_id);
        let mut conn = self.client.get_async_connection()
            .await
            .map_err(|e| AofError::lock_failed(format!("Redis connection failed: {}", e)))?;

        let exists: bool = conn.exists(&key)
            .await
            .map_err(|e| AofError::lock_failed(format!("EXISTS check failed: {}", e)))?;

        Ok(exists)
    }
}

/// File-based resource lock (fallback for development/testing)
pub struct FileLock {
    lock_dir: PathBuf,
    resource_id: String,
    agent_id: String,
    ttl: u64,
    timeout: u64,
}

impl FileLock {
    /// Create a new file-based lock
    pub async fn new(
        lock_dir: PathBuf,
        resource_id: impl Into<String>,
        agent_id: impl Into<String>,
        ttl: u64,
        timeout: u64,
    ) -> Result<Self, AofError> {
        // Create lock directory if it doesn't exist
        fs::create_dir_all(&lock_dir)
            .await
            .map_err(|e| AofError::lock_failed(format!("Failed to create lock dir: {}", e)))?;

        Ok(Self {
            lock_dir,
            resource_id: resource_id.into(),
            agent_id: agent_id.into(),
            ttl,
            timeout,
        })
    }

    fn lock_file_path(&self) -> PathBuf {
        self.lock_dir.join(format!("{}.lock", self.resource_id))
    }

    fn lock_content(&self) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        format!("{}:{}:{}", self.agent_id, now, self.ttl)
    }

    fn parse_lock_content(content: &str) -> Option<(String, u64, u64)> {
        let parts: Vec<&str> = content.split(':').collect();
        if parts.len() == 3 {
            let agent_id = parts[0].to_string();
            let timestamp = parts[1].parse::<u64>().ok()?;
            let ttl = parts[2].parse::<u64>().ok()?;
            Some((agent_id, timestamp, ttl))
        } else {
            None
        }
    }

    fn is_expired(timestamp: u64, ttl: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now > timestamp + ttl
    }

    /// Acquire lock
    /// Returns true if acquired, false if already locked (and not expired)
    pub async fn acquire(&self) -> Result<bool, AofError> {
        let lock_path = self.lock_file_path();

        // Ensure directory exists
        fs::create_dir_all(lock_path.parent().unwrap_or(&self.lock_dir))
            .await
            .map_err(|e| AofError::lock_failed(format!("Failed to ensure lock dir exists: {}", e)))?;

        // Try to read existing lock
        if let Ok(content) = fs::read_to_string(&lock_path).await {
            if let Some((_, timestamp, ttl)) = Self::parse_lock_content(&content) {
                if !Self::is_expired(timestamp, ttl) {
                    // Lock is still valid
                    return Ok(false);
                }
            }
        }

        // Write lock file directly
        fs::write(&lock_path, self.lock_content())
            .await
            .map_err(|e| AofError::lock_failed(format!("Failed to write lock: {}", e)))?;

        Ok(true)
    }

    /// Release lock
    /// Returns true if released, false if not owner
    pub async fn release(&self) -> Result<bool, AofError> {
        let lock_path = self.lock_file_path();

        if let Ok(content) = fs::read_to_string(&lock_path).await {
            if let Some((agent_id, _, _)) = Self::parse_lock_content(&content) {
                if agent_id == self.agent_id {
                    fs::remove_file(&lock_path)
                        .await
                        .map_err(|e| AofError::lock_failed(format!("Failed to remove lock: {}", e)))?;
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Extend lock TTL
    /// Returns true if extended, false if not owner
    pub async fn extend(&self) -> Result<bool, AofError> {
        let lock_path = self.lock_file_path();

        if let Ok(content) = fs::read_to_string(&lock_path).await {
            if let Some((agent_id, _, _)) = Self::parse_lock_content(&content) {
                if agent_id == self.agent_id {
                    fs::write(&lock_path, self.lock_content())
                        .await
                        .map_err(|e| AofError::lock_failed(format!("Failed to write lock: {}", e)))?;
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Acquire lock with blocking wait
    /// Returns true if acquired, false if timeout
    pub async fn acquire_with_wait(&self) -> Result<bool, AofError> {
        let start = SystemTime::now();
        let timeout_duration = Duration::from_secs(self.timeout);

        loop {
            if self.acquire().await? {
                return Ok(true);
            }

            if start.elapsed().unwrap_or_default() > timeout_duration {
                return Ok(false);
            }

            sleep(Duration::from_millis(100)).await;
        }
    }

    /// Check if lock exists
    pub async fn is_locked(&self) -> Result<bool, AofError> {
        let lock_path = self.lock_file_path();

        if let Ok(content) = fs::read_to_string(&lock_path).await {
            if let Some((_, timestamp, ttl)) = Self::parse_lock_content(&content) {
                return Ok(!Self::is_expired(timestamp, ttl));
            }
        }

        Ok(false)
    }
}

/// Lock manager factory (Redis with file-based fallback)
pub enum LockManager {
    Redis(ResourceLock),
    File(FileLock),
}

impl LockManager {
    /// Create new lock manager (try Redis, fallback to file)
    pub async fn new(
        config: LockConfig,
        resource_id: impl Into<String>,
        agent_id: impl Into<String>,
    ) -> Result<Self, AofError> {
        let resource_id = resource_id.into();
        let agent_id = agent_id.into();
        let ttl = config.ttl;
        let timeout = config.timeout;

        // Try Redis first
        if let Some(redis_url) = config.redis_url {
            match Client::open(redis_url.clone()) {
                Ok(client) => {
                    // Test connection
                    if client.get_async_connection().await.is_ok() {
                        return Ok(LockManager::Redis(ResourceLock::new(
                            Arc::new(client),
                            resource_id,
                            agent_id,
                            ttl,
                            timeout,
                        ).await?));
                    } else {
                        tracing::warn!("Redis connection test failed, falling back to file-based locks");
                    }
                }
                Err(e) => {
                    tracing::warn!("Redis client creation failed, falling back to file-based locks: {}", e);
                }
            }
        }

        // Fallback to file-based locking
        let lock_dir = config.lock_dir.unwrap_or_else(|| PathBuf::from("/tmp/aof-locks"));
        let file_lock = FileLock::new(lock_dir, resource_id, agent_id, ttl, timeout).await?;
        Ok(LockManager::File(file_lock))
    }

    /// Acquire lock
    pub async fn acquire(&self) -> Result<bool, AofError> {
        match self {
            LockManager::Redis(lock) => lock.acquire().await,
            LockManager::File(lock) => lock.acquire().await,
        }
    }

    /// Extend lock TTL
    pub async fn extend(&self) -> Result<bool, AofError> {
        match self {
            LockManager::Redis(lock) => lock.extend().await,
            LockManager::File(lock) => lock.extend().await,
        }
    }

    /// Release lock
    pub async fn release(&self) -> Result<bool, AofError> {
        match self {
            LockManager::Redis(lock) => lock.release().await,
            LockManager::File(lock) => lock.release().await,
        }
    }

    /// Acquire with wait
    pub async fn acquire_with_wait(&self) -> Result<bool, AofError> {
        match self {
            LockManager::Redis(lock) => lock.acquire_with_wait().await,
            LockManager::File(lock) => lock.acquire_with_wait().await,
        }
    }

    /// Check if locked
    pub async fn is_locked(&self) -> Result<bool, AofError> {
        match self {
            LockManager::Redis(lock) => lock.is_locked().await,
            LockManager::File(lock) => lock.is_locked().await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_file_lock_acquire() {
        let lock = FileLock::new(PathBuf::from("/tmp/aof-test-locks"), "test-resource", "agent-001", 5, 10)
            .await
            .unwrap();

        assert!(lock.acquire().await.unwrap());
        assert!(!lock.acquire().await.unwrap()); // Second acquire should fail
        assert!(lock.release().await.unwrap());
        assert!(!lock.release().await.unwrap()); // Second release should fail
    }

    #[tokio::test]
    async fn test_file_lock_ownership() {
        let lock1 = FileLock::new(PathBuf::from("/tmp/aof-test-locks"), "test-resource-2", "agent-001", 5, 10)
            .await
            .unwrap();
        let lock2 = FileLock::new(PathBuf::from("/tmp/aof-test-locks"), "test-resource-2", "agent-002", 5, 10)
            .await
            .unwrap();

        assert!(lock1.acquire().await.unwrap());
        assert!(!lock2.release().await.unwrap()); // Different agent can't release
        assert!(lock1.release().await.unwrap());
    }

    #[tokio::test]
    async fn test_file_lock_extend() {
        let lock = FileLock::new(PathBuf::from("/tmp/aof-test-locks"), "test-resource-3", "agent-001", 5, 10)
            .await
            .unwrap();

        assert!(lock.acquire().await.unwrap());
        assert!(lock.extend().await.unwrap());
        assert!(lock.is_locked().await.unwrap());
        assert!(lock.release().await.unwrap());
    }

    #[tokio::test]
    async fn test_file_lock_wait() {
        let lock = FileLock::new(PathBuf::from("/tmp/aof-test-locks"), "test-resource-4", "agent-001", 2, 3)
            .await
            .unwrap();

        assert!(lock.acquire().await.unwrap());

        let lock2 = FileLock::new(PathBuf::from("/tmp/aof-test-locks"), "test-resource-4", "agent-002", 2, 3)
            .await
            .unwrap();

        // Should timeout after 3 seconds
        let start = std::time::Instant::now();
        let acquired = lock2.acquire_with_wait().await.unwrap();
        let elapsed = start.elapsed();

        // First attempt fails (locked), then waits
        // Lock expires after 2 seconds, so should acquire on next attempt
        // Total should be > 2 seconds but < 5 seconds
        assert!(acquired || elapsed.as_secs() >= 2);

        let _ = lock.release().await;
    }

    #[tokio::test]
    async fn test_file_lock_is_locked() {
        let lock = FileLock::new(PathBuf::from("/tmp/aof-test-locks"), "test-resource-5", "agent-001", 5, 10)
            .await
            .unwrap();

        assert!(!lock.is_locked().await.unwrap());
        assert!(lock.acquire().await.unwrap());
        assert!(lock.is_locked().await.unwrap());
        assert!(lock.release().await.unwrap());
        assert!(!lock.is_locked().await.unwrap());
    }

    #[test]
    fn test_parse_lock_content() {
        let content = "agent-001:1234567890:30";
        let (agent_id, timestamp, ttl) = FileLock::parse_lock_content(content).unwrap();
        assert_eq!(agent_id, "agent-001");
        assert_eq!(timestamp, 1234567890);
        assert_eq!(ttl, 30);
    }

    #[test]
    fn test_lock_expiry() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert!(!FileLock::is_expired(now - 10, 30)); // 20 seconds old, 30 second TTL = not expired
        assert!(FileLock::is_expired(now - 40, 30)); // 40 seconds old, 30 second TTL = expired
    }
}
