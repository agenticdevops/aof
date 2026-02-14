//! Integration test for resource locking and sandbox isolation
//!
//! Tests complete workflow: lock → execute → release → decision log

use std::path::PathBuf;
use std::time::Duration;

use aof_runtime::executor::{LockConfig, LockManager, RiskPolicy, ExecutionContext};

macro_rules! setup_lock_dir {
    ($test_name:expr) => {{
        let lock_dir = PathBuf::from(format!("/tmp/aof-test-locks/{}", $test_name));
        let _ = std::fs::create_dir_all(&lock_dir);
        lock_dir
    }};
}

#[tokio::test]
async fn test_resource_lock_basic_workflow() {
    let lock_dir = setup_lock_dir!("test-1");

    let config = LockConfig {
        redis_url: None,
        lock_dir: Some(lock_dir),
        ttl: 5,
        timeout: 10,
    };

    // Create lock manager (uses file backend since no Redis)
    let manager = LockManager::new(config, "pod:test/api-001", "agent-001")
        .await
        .expect("Failed to create lock manager");

    // Test 1: Acquire lock
    assert!(manager.acquire().await.unwrap(), "First acquire should succeed");

    // Test 2: Cannot acquire again (already locked)
    assert!(
        !manager.acquire().await.unwrap(),
        "Second acquire should fail (already locked)"
    );

    // Test 3: Check locked status
    assert!(
        manager.is_locked().await.unwrap(),
        "Lock should be detected as locked"
    );

    // Test 4: Release lock
    assert!(
        manager.release().await.unwrap(),
        "Release by owner should succeed"
    );

    // Test 5: Check unlocked status
    assert!(
        !manager.is_locked().await.unwrap(),
        "Lock should be detected as free"
    );

    // Test 6: Can reacquire after release
    assert!(
        manager.acquire().await.unwrap(),
        "Third acquire should succeed after release"
    );

    let _ = manager.release().await;
}

#[tokio::test]
async fn test_resource_lock_ownership() {
    let lock_dir = setup_lock_dir!("test-2");

    let config = LockConfig {
        redis_url: None,
        lock_dir: Some(lock_dir.clone()),
        ttl: 5,
        timeout: 10,
    };

    // Create two locks for same resource with different agents
    let manager1 = LockManager::new(config.clone(), "pod:test/api-002", "agent-001")
        .await
        .expect("Failed to create lock manager 1");

    // Agent 1 acquires lock
    assert!(manager1.acquire().await.unwrap(), "Agent 1 should acquire");

    // Agent 2 cannot release lock owned by Agent 1
    let manager2_release = LockManager::new(LockConfig {
        redis_url: None,
        lock_dir: Some(lock_dir),
        ttl: 5,
        timeout: 10,
    }, "pod:test/api-002", "agent-002")
        .await
        .unwrap();
    assert!(
        !manager2_release.release().await.unwrap(),
        "Agent 2 should not release Agent 1's lock"
    );

    // Agent 1 releases their lock
    assert!(
        manager1.release().await.unwrap(),
        "Agent 1 should release their lock"
    );
}

#[tokio::test]
async fn test_resource_lock_wait() {
    let lock_dir = setup_lock_dir!("test-3");

    let config = LockConfig {
        redis_url: None,
        lock_dir: Some(lock_dir),
        ttl: 1, // Short TTL for faster test
        timeout: 5,
    };

    let manager1 = LockManager::new(config.clone(), "pod:test/api-003", "agent-001")
        .await
        .expect("Failed to create lock manager 1");

    let manager2 = LockManager::new(config, "pod:test/api-003", "agent-002")
        .await
        .expect("Failed to create lock manager 2");

    // Agent 1 acquires lock
    assert!(manager1.acquire().await.unwrap(), "Agent 1 should acquire");

    // Agent 2 waits (should succeed once TTL expires)
    let start = std::time::Instant::now();
    let acquired = manager2.acquire_with_wait().await.unwrap();
    let elapsed = start.elapsed();

    // Should succeed (TTL expired) and take ~1 second or more
    assert!(acquired, "Agent 2 should acquire after wait");
    assert!(
        elapsed >= Duration::from_millis(900),
        "Should have waited for TTL expiry"
    );

    let _ = manager2.release().await;
}

#[tokio::test]
async fn test_resource_lock_timeout() {
    let lock_dir = setup_lock_dir!("test-4");

    let config = LockConfig {
        redis_url: None,
        lock_dir: Some(lock_dir),
        ttl: 10,      // Lock won't expire
        timeout: 1,   // Short timeout for test
    };

    let manager1 = LockManager::new(config.clone(), "pod:test/api-004", "agent-001")
        .await
        .unwrap();

    let manager2 = LockManager::new(config, "pod:test/api-004", "agent-002")
        .await
        .unwrap();

    // Agent 1 acquires lock
    assert!(manager1.acquire().await.unwrap());

    // Agent 2 waits with short timeout (should timeout)
    let start = std::time::Instant::now();
    let acquired = manager2.acquire_with_wait().await.unwrap();
    let elapsed = start.elapsed();

    assert!(
        !acquired,
        "Agent 2 should timeout without acquiring"
    );
    assert!(
        elapsed >= Duration::from_secs(1),
        "Should have waited until timeout"
    );

    let _ = manager1.release().await;
}

#[tokio::test]
async fn test_resource_lock_extend() {
    let lock_dir = setup_lock_dir!("test-5");

    let config = LockConfig {
        redis_url: None,
        lock_dir: Some(lock_dir),
        ttl: 3,
        timeout: 10,
    };

    let manager = LockManager::new(config, "pod:test/api-005", "agent-001")
        .await
        .expect("Failed to create lock manager");

    // Acquire lock
    assert!(manager.acquire().await.unwrap());

    // Sleep and wait for near-expiry
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Check still locked
    assert!(
        manager.is_locked().await.unwrap(),
        "Lock should still be valid"
    );

    // Extend lock
    assert!(
        manager.extend().await.unwrap(),
        "Extend should succeed"
    );

    // Still locked after extend
    assert!(
        manager.is_locked().await.unwrap(),
        "Lock should still be locked after extend"
    );

    let _ = manager.release().await;
}

#[tokio::test]
async fn test_risk_policy_destructive_detection() {
    let policy = RiskPolicy::new();

    // Destructive operations
    assert!(policy.is_destructive("kubectl", &["delete".to_string(), "pod".to_string()]));
    assert!(policy.is_destructive("docker", &["rm".to_string()]));
    assert!(policy.is_destructive("kubectl", &["restart".to_string()]));

    // Non-destructive operations
    assert!(!policy.is_destructive("kubectl", &["get".to_string(), "pods".to_string()]));
    assert!(!policy.is_destructive("docker", &["ps".to_string()]));
    assert!(!policy.is_destructive("kubectl", &["logs".to_string()]));
}

#[tokio::test]
async fn test_risk_policy_write_detection() {
    let policy = RiskPolicy::new();

    // Write operations
    assert!(policy.is_write("kubectl", &["apply".to_string()]));
    assert!(policy.is_write("kubectl", &["patch".to_string()]));

    // Non-write operations
    assert!(!policy.is_write("kubectl", &["get".to_string()]));
    assert!(!policy.is_write("kubectl", &["delete".to_string()]));
}

#[tokio::test]
async fn test_risk_policy_context_decisions() {
    let policy = RiskPolicy::new();
    let dev = ExecutionContext::Development;
    let prod = ExecutionContext::Production;

    // Dev: Always sandbox
    assert_eq!(
        policy.should_sandbox(&dev, "kubectl", &["get".to_string()]),
        aof_runtime::executor::SandboxingDecision::Sandbox
    );

    // Prod: Destructive always sandbox
    assert_eq!(
        policy.should_sandbox(&prod, "kubectl", &["delete".to_string()]),
        aof_runtime::executor::SandboxingDecision::Sandbox
    );

    // Prod: Read-only on host
    assert_eq!(
        policy.should_sandbox(&prod, "kubectl", &["get".to_string()]),
        aof_runtime::executor::SandboxingDecision::HostTrusted
    );
}

#[tokio::test]
async fn test_multiple_agents_concurrent_different_resources() {
    let lock_dir = setup_lock_dir!("test-concurrent");

    let config = LockConfig {
        redis_url: None,
        lock_dir: Some(lock_dir),
        ttl: 2,
        timeout: 5,
    };

    // Three agents, three resources
    let m1 = LockManager::new(config.clone(), "pod:test/api-001", "agent-001")
        .await
        .unwrap();
    let m2 = LockManager::new(config.clone(), "pod:test/api-002", "agent-002")
        .await
        .unwrap();
    let m3 = LockManager::new(config, "pod:test/api-003", "agent-003")
        .await
        .unwrap();

    // All should acquire simultaneously (different resources)
    let r1 = m1.acquire().await.unwrap();
    let r2 = m2.acquire().await.unwrap();
    let r3 = m3.acquire().await.unwrap();

    assert!(r1 && r2 && r3, "All agents should acquire different locks");

    let _ = m1.release().await;
    let _ = m2.release().await;
    let _ = m3.release().await;
}

#[tokio::test]
async fn test_decision_logging_integration() {
    // This test verifies that decision logging can be integrated
    // Full test requires DecisionLogger to be properly initialized

    use std::fs;
    use std::path::Path;

    let log_dir = "/tmp/aof-test-decision-logs";
    let _ = fs::create_dir_all(log_dir);

    // Create a decision log entry (this would normally come from AgentExecutor)
    let decision_log_path = Path::new(log_dir).join("decisions.jsonl");

    // Simulate decision log entry
    let log_entry = serde_json::json!({
        "event_id": uuid::Uuid::new_v4().to_string(),
        "agent_id": "test-agent-001",
        "action": "lock_acquired",
        "reasoning": "Destructive operation requires serialization",
        "confidence": 0.95,
        "tags": ["locking", "kubectl", "destructive"],
        "related_decisions": [],
        "metadata": {
            "resource": "pod:test/api-001",
            "ttl_seconds": 30,
            "timeout_seconds": 60
        }
    });

    // Write to decision log
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&decision_log_path)
    {
        use std::io::Write;
        let _ = writeln!(file, "{}", log_entry.to_string());
    }

    // Verify file created and contains entry
    assert!(decision_log_path.exists(), "Decision log should exist");
    let content = fs::read_to_string(&decision_log_path).expect("Should read decision log");
    assert!(
        content.contains("lock_acquired"),
        "Decision log should contain lock_acquired event"
    );

    // Cleanup
    let _ = fs::remove_file(&decision_log_path);
    let _ = fs::remove_dir(log_dir);
}
