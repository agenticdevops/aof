//! File-based memory backend for persistent storage
//!
//! Stores memory entries in a JSON file that persists across agent runs.
//! Supports optional max_entries limit to prevent unbounded file growth.

use aof_core::{AofError, AofResult, MemoryBackend, MemoryEntry, MemoryQuery};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// File-based memory backend
///
/// Stores all memory entries in a JSON file. Changes are written immediately
/// to ensure persistence across agent runs.
///
/// ## Max Entries Limit
///
/// You can configure a maximum number of entries to prevent unbounded file growth.
/// When the limit is reached, the oldest entries (by creation time) are removed
/// to make room for new ones. This is useful for conversation history where you
/// only want to retain the last N interactions.
///
/// ## Example
///
/// ```rust,no_run
/// use aof_memory::FileBackend;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Unlimited entries
/// let backend = FileBackend::new("./memory.json").await?;
///
/// // Limited to 100 most recent entries
/// let backend = FileBackend::with_max_entries("./memory.json", Some(100)).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct FileBackend {
    /// Path to the JSON file
    path: PathBuf,
    /// In-memory cache of entries
    cache: Arc<RwLock<HashMap<String, MemoryEntry>>>,
    /// Maximum number of entries (oldest removed when exceeded)
    max_entries: Option<usize>,
}

impl FileBackend {
    /// Create a new file backend with the given path
    ///
    /// If the file exists, loads existing entries. Otherwise creates empty storage.
    pub async fn new(path: impl Into<PathBuf>) -> AofResult<Self> {
        Self::with_max_entries(path, None).await
    }

    /// Create a new file backend with optional max entries limit
    ///
    /// When `max_entries` is set, the oldest entries (by creation time) are
    /// automatically removed when the limit is exceeded. This prevents
    /// unbounded file growth for conversation history.
    ///
    /// # Arguments
    /// * `path` - Path to the JSON file
    /// * `max_entries` - Maximum number of entries to keep (None = unlimited)
    pub async fn with_max_entries(
        path: impl Into<PathBuf>,
        max_entries: Option<usize>,
    ) -> AofResult<Self> {
        let path = path.into();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                tokio::fs::create_dir_all(parent).await.map_err(|e| {
                    AofError::memory(format!(
                        "Failed to create directory {}: {}",
                        parent.display(),
                        e
                    ))
                })?;
            }
        }

        // Load existing data or create empty
        let mut cache: HashMap<String, MemoryEntry> = if path.exists() {
            let content = tokio::fs::read_to_string(&path).await.map_err(|e| {
                AofError::memory(format!("Failed to read memory file {}: {}", path.display(), e))
            })?;

            if content.trim().is_empty() {
                HashMap::new()
            } else {
                serde_json::from_str(&content).map_err(|e| {
                    AofError::memory(format!(
                        "Failed to parse memory file {}: {}",
                        path.display(),
                        e
                    ))
                })?
            }
        } else {
            HashMap::new()
        };

        // Apply max_entries limit on load (trim oldest if over limit)
        if let Some(max) = max_entries {
            if cache.len() > max {
                Self::trim_oldest_entries(&mut cache, max);
            }
        }

        let backend = Self {
            path,
            cache: Arc::new(RwLock::new(cache)),
            max_entries,
        };

        // Persist if we trimmed entries on load
        if max_entries.is_some() {
            backend.persist().await?;
        }

        Ok(backend)
    }

    /// Trim cache to max_entries by removing oldest entries
    fn trim_oldest_entries(cache: &mut HashMap<String, MemoryEntry>, max: usize) {
        if cache.len() <= max {
            return;
        }

        // Sort entries by timestamp (oldest first)
        let mut entries: Vec<_> = cache.iter().collect();
        entries.sort_by(|a, b| a.1.timestamp.cmp(&b.1.timestamp));

        // Remove oldest entries until we're at max
        let to_remove = cache.len() - max;
        let keys_to_remove: Vec<String> = entries
            .iter()
            .take(to_remove)
            .map(|(k, _)| (*k).clone())
            .collect();

        for key in keys_to_remove {
            cache.remove(&key);
        }
    }

    /// Persist current cache to file
    async fn persist(&self) -> AofResult<()> {
        let cache = self.cache.read().await;
        let content = serde_json::to_string_pretty(&*cache).map_err(|e| {
            AofError::memory(format!("Failed to serialize memory: {}", e))
        })?;
        drop(cache);

        tokio::fs::write(&self.path, content).await.map_err(|e| {
            AofError::memory(format!(
                "Failed to write memory file {}: {}",
                self.path.display(),
                e
            ))
        })?;

        Ok(())
    }

    /// Get the file path
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Get the maximum entries limit
    pub fn max_entries(&self) -> Option<usize> {
        self.max_entries
    }

    /// Get the number of entries
    pub async fn len(&self) -> usize {
        self.cache.read().await.len()
    }

    /// Check if empty
    pub async fn is_empty(&self) -> bool {
        self.cache.read().await.is_empty()
    }
}

#[async_trait]
impl MemoryBackend for FileBackend {
    async fn store(&self, key: &str, entry: MemoryEntry) -> AofResult<()> {
        {
            let mut cache = self.cache.write().await;
            cache.insert(key.to_string(), entry);

            // Enforce max_entries limit
            if let Some(max) = self.max_entries {
                if cache.len() > max {
                    Self::trim_oldest_entries(&mut cache, max);
                }
            }
        }
        self.persist().await
    }

    async fn retrieve(&self, key: &str) -> AofResult<Option<MemoryEntry>> {
        let cache = self.cache.read().await;
        match cache.get(key) {
            Some(entry) => {
                if entry.is_expired() {
                    drop(cache);
                    // Lazy cleanup: delete expired entry
                    let mut cache = self.cache.write().await;
                    cache.remove(key);
                    drop(cache);
                    self.persist().await?;
                    Ok(None)
                } else {
                    Ok(Some(entry.clone()))
                }
            }
            None => Ok(None),
        }
    }

    async fn delete(&self, key: &str) -> AofResult<()> {
        {
            let mut cache = self.cache.write().await;
            cache.remove(key);
        }
        self.persist().await
    }

    async fn list_keys(&self, prefix: Option<&str>) -> AofResult<Vec<String>> {
        let cache = self.cache.read().await;
        let keys: Vec<String> = match prefix {
            Some(p) => cache
                .keys()
                .filter(|k| k.starts_with(p))
                .cloned()
                .collect(),
            None => cache.keys().cloned().collect(),
        };
        Ok(keys)
    }

    async fn clear(&self) -> AofResult<()> {
        {
            let mut cache = self.cache.write().await;
            cache.clear();
        }
        self.persist().await
    }

    async fn search(&self, query: &MemoryQuery) -> AofResult<Vec<MemoryEntry>> {
        let cache = self.cache.read().await;
        let mut results = Vec::new();

        for entry in cache.values() {
            // Check prefix filter
            if let Some(ref prefix) = query.prefix {
                if !entry.key.starts_with(prefix) {
                    continue;
                }
            }

            // Check if entry matches query
            if query.matches(entry) {
                results.push(entry.clone());

                // Check limit
                if let Some(limit) = query.limit {
                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_file_backend_store_retrieve() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("memory.json");

        let backend = FileBackend::new(&path).await.unwrap();

        let entry = MemoryEntry::new("test_key", json!({"data": "test"}));
        backend.store("test_key", entry).await.unwrap();

        let retrieved = backend.retrieve("test_key").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().key, "test_key");

        // Verify file was created
        assert!(path.exists());
    }

    #[tokio::test]
    async fn test_file_backend_persistence() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("memory.json");

        // Store data
        {
            let backend = FileBackend::new(&path).await.unwrap();
            backend
                .store("key1", MemoryEntry::new("key1", json!({"value": 1})))
                .await
                .unwrap();
            backend
                .store("key2", MemoryEntry::new("key2", json!({"value": 2})))
                .await
                .unwrap();
        }

        // Create new backend instance and verify data persisted
        {
            let backend = FileBackend::new(&path).await.unwrap();
            let key1 = backend.retrieve("key1").await.unwrap();
            assert!(key1.is_some());
            assert_eq!(key1.unwrap().value, json!({"value": 1}));

            let key2 = backend.retrieve("key2").await.unwrap();
            assert!(key2.is_some());
            assert_eq!(key2.unwrap().value, json!({"value": 2}));
        }
    }

    #[tokio::test]
    async fn test_file_backend_delete() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("memory.json");

        let backend = FileBackend::new(&path).await.unwrap();

        backend
            .store("key1", MemoryEntry::new("key1", json!(1)))
            .await
            .unwrap();
        backend.delete("key1").await.unwrap();

        let retrieved = backend.retrieve("key1").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_file_backend_list_keys() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("memory.json");

        let backend = FileBackend::new(&path).await.unwrap();

        backend
            .store("user:1", MemoryEntry::new("user:1", json!(1)))
            .await
            .unwrap();
        backend
            .store("user:2", MemoryEntry::new("user:2", json!(2)))
            .await
            .unwrap();
        backend
            .store("admin:1", MemoryEntry::new("admin:1", json!(3)))
            .await
            .unwrap();

        let all_keys = backend.list_keys(None).await.unwrap();
        assert_eq!(all_keys.len(), 3);

        let user_keys = backend.list_keys(Some("user:")).await.unwrap();
        assert_eq!(user_keys.len(), 2);
    }

    #[tokio::test]
    async fn test_file_backend_clear() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("memory.json");

        let backend = FileBackend::new(&path).await.unwrap();

        backend
            .store("key1", MemoryEntry::new("key1", json!(1)))
            .await
            .unwrap();
        backend
            .store("key2", MemoryEntry::new("key2", json!(2)))
            .await
            .unwrap();

        backend.clear().await.unwrap();

        let keys = backend.list_keys(None).await.unwrap();
        assert_eq!(keys.len(), 0);
    }

    #[tokio::test]
    async fn test_file_backend_creates_directory() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nested/dir/memory.json");

        let backend = FileBackend::new(&path).await.unwrap();
        backend
            .store("key1", MemoryEntry::new("key1", json!(1)))
            .await
            .unwrap();

        assert!(path.exists());
    }

    #[tokio::test]
    async fn test_file_backend_max_entries() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("memory.json");

        // Create backend with max 3 entries
        let backend = FileBackend::with_max_entries(&path, Some(3)).await.unwrap();

        // Store 5 entries with small delays to ensure different timestamps
        for i in 1..=5 {
            let entry = MemoryEntry::new(&format!("key{}", i), json!(i));
            backend.store(&format!("key{}", i), entry).await.unwrap();
            // Small delay to ensure different creation timestamps
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Should only have 3 entries (the most recent ones)
        let keys = backend.list_keys(None).await.unwrap();
        assert_eq!(keys.len(), 3);

        // Oldest entries (key1, key2) should be removed
        assert!(backend.retrieve("key1").await.unwrap().is_none());
        assert!(backend.retrieve("key2").await.unwrap().is_none());

        // Newest entries (key3, key4, key5) should still exist
        assert!(backend.retrieve("key3").await.unwrap().is_some());
        assert!(backend.retrieve("key4").await.unwrap().is_some());
        assert!(backend.retrieve("key5").await.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_file_backend_max_entries_on_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("memory.json");

        // Create backend without limit and store 5 entries
        {
            let backend = FileBackend::new(&path).await.unwrap();
            for i in 1..=5 {
                let entry = MemoryEntry::new(&format!("key{}", i), json!(i));
                backend.store(&format!("key{}", i), entry).await.unwrap();
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
            assert_eq!(backend.len().await, 5);
        }

        // Reload with max 2 entries - should trim oldest on load
        {
            let backend = FileBackend::with_max_entries(&path, Some(2)).await.unwrap();
            assert_eq!(backend.len().await, 2);

            // Only newest entries should remain
            assert!(backend.retrieve("key4").await.unwrap().is_some());
            assert!(backend.retrieve("key5").await.unwrap().is_some());
        }
    }
}
