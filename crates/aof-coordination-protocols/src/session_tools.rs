//! Session tools for agent-to-agent communication
//!
//! Provides async message queues between agent pairs using tokio mpsc channels.
//! Each agent has ONE inbound queue that all other agents write to.

use crate::error::CoordinationProtocolError;
use crate::events::SessionMessage;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tracing::debug;

/// Session tools for managing agent-to-agent message queues
///
/// # Architecture
///
/// - Each agent has ONE inbound mpsc channel
/// - All senders write to that agent's channel
/// - Messages are fire-and-forget (non-blocking try_send)
/// - Expired messages are filtered on drain
/// - Bounded queues prevent memory bloat
///
/// # Example
///
/// ```rust,no_run
/// use aof_coordination_protocols::SessionTools;
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let session_tools = SessionTools::new(100, Duration::from_secs(30 * 60));
///
///     session_tools.register_agent("agent-a").await?;
///     session_tools.register_agent("agent-b").await?;
///
///     // ... send and drain messages ...
///     Ok(())
/// }
/// ```
pub struct SessionTools {
    /// Map: agent_id -> Sender for that agent's inbound queue
    inbound_senders: Arc<RwLock<HashMap<String, mpsc::Sender<SessionMessage>>>>,
    /// Map: agent_id -> Receiver for that agent's inbound queue
    inbound_receivers: Arc<RwLock<HashMap<String, mpsc::Receiver<SessionMessage>>>>,
    /// Queue capacity per agent (default: 100)
    capacity: usize,
    /// Message TTL (default: 30 minutes)
    ttl: Duration,
}

impl SessionTools {
    /// Create a new SessionTools instance
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum messages per agent queue
    /// * `ttl` - Message time-to-live
    pub fn new(capacity: usize, ttl: Duration) -> Self {
        Self {
            inbound_senders: Arc::new(RwLock::new(HashMap::new())),
            inbound_receivers: Arc::new(RwLock::new(HashMap::new())),
            capacity,
            ttl,
        }
    }

    /// Register an agent to receive messages
    ///
    /// Creates an inbound mpsc channel for the agent.
    /// Idempotent: registering the same agent twice is a no-op.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - Unique agent identifier
    pub async fn register_agent(&self, agent_id: &str) -> Result<(), CoordinationProtocolError> {
        let mut senders = self.inbound_senders.write().await;
        let mut receivers = self.inbound_receivers.write().await;

        // Idempotent: skip if already registered
        if senders.contains_key(agent_id) {
            debug!("Agent {} already registered, skipping", agent_id);
            return Ok(());
        }

        // Create bounded mpsc channel
        let (tx, rx) = mpsc::channel(self.capacity);
        senders.insert(agent_id.to_string(), tx);
        receivers.insert(agent_id.to_string(), rx);

        debug!("Registered agent {} with capacity {}", agent_id, self.capacity);
        Ok(())
    }

    /// Unregister an agent
    ///
    /// Removes the agent's sender and receiver. Any pending messages are dropped.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - Agent to unregister
    pub async fn unregister_agent(&self, agent_id: &str) {
        let mut senders = self.inbound_senders.write().await;
        let mut receivers = self.inbound_receivers.write().await;

        senders.remove(agent_id);
        receivers.remove(agent_id);

        debug!("Unregistered agent {}", agent_id);
    }

    /// Send a message to an agent (fire-and-forget)
    ///
    /// Uses try_send to avoid blocking. Returns QueueFull error if the
    /// recipient's queue is at capacity.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send
    pub async fn send_message(&self, message: SessionMessage) -> Result<(), CoordinationProtocolError> {
        let senders = self.inbound_senders.read().await;

        // Look up the recipient's sender
        let sender = senders.get(&message.to_agent).ok_or_else(|| {
            CoordinationProtocolError::AgentNotFound(message.to_agent.clone())
        })?;

        // Fire-and-forget: try_send (non-blocking)
        sender.try_send(message.clone()).map_err(|e| match e {
            mpsc::error::TrySendError::Full(_) => CoordinationProtocolError::QueueFull {
                from: message.from_agent.clone(),
                to: message.to_agent.clone(),
                capacity: self.capacity,
            },
            mpsc::error::TrySendError::Closed(_) => {
                CoordinationProtocolError::AgentNotFound(message.to_agent.clone())
            }
        })?;

        debug!(
            "Sent {:?} message from {} to {}",
            message.message_type, message.from_agent, message.to_agent
        );

        Ok(())
    }

    /// Drain all pending messages for an agent
    ///
    /// Uses try_recv in a loop to collect all messages without blocking.
    /// Filters out expired messages (based on TTL).
    ///
    /// # Arguments
    ///
    /// * `agent_id` - Agent whose messages to drain
    ///
    /// # Returns
    ///
    /// Vector of non-expired messages
    pub async fn drain_messages(&self, agent_id: &str) -> Vec<SessionMessage> {
        let mut receivers = self.inbound_receivers.write().await;

        let receiver = match receivers.get_mut(agent_id) {
            Some(rx) => rx,
            None => {
                debug!("Agent {} not found for draining", agent_id);
                return Vec::new();
            }
        };

        let mut messages = Vec::new();

        // Non-blocking drain using try_recv
        loop {
            match receiver.try_recv() {
                Ok(msg) => {
                    // Filter out expired messages
                    if msg.is_expired() {
                        debug!("Dropping expired message {} for {}", msg.id, agent_id);
                    } else {
                        messages.push(msg);
                    }
                }
                Err(mpsc::error::TryRecvError::Empty) => break,
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    debug!("Channel disconnected for agent {}", agent_id);
                    break;
                }
            }
        }

        debug!("Drained {} messages for agent {}", messages.len(), agent_id);
        messages
    }

    /// Get count of pending messages for an agent (approximate)
    ///
    /// Note: This is a best-effort count and may be stale due to concurrent access.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - Agent to check
    pub async fn pending_count(&self, agent_id: &str) -> usize {
        let receivers = self.inbound_receivers.read().await;

        // We can't peek at channel length directly, so we return 0
        // This is a limitation of tokio mpsc - no len() method
        // For actual count, caller must drain_messages
        if receivers.contains_key(agent_id) {
            0 // Placeholder - actual count requires draining
        } else {
            0
        }
    }

    /// Get list of all registered agents
    pub async fn registered_agents(&self) -> Vec<String> {
        let senders = self.inbound_senders.read().await;
        senders.keys().cloned().collect()
    }

    /// Get the configured capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get the configured TTL
    pub fn ttl(&self) -> Duration {
        self.ttl
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::MessageType;
    use chrono::Duration as ChronoDuration;

    #[tokio::test]
    async fn test_register_and_send_message() {
        let session_tools = SessionTools::new(100, Duration::from_secs(30 * 60));

        session_tools.register_agent("agent-a").await.unwrap();
        session_tools.register_agent("agent-b").await.unwrap();

        let message = SessionMessage::new(
            "agent-a",
            "agent-b",
            MessageType::Announcement,
            "Hello from A",
            ChronoDuration::minutes(30),
        );

        session_tools.send_message(message.clone()).await.unwrap();

        let messages = session_tools.drain_messages("agent-b").await;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].from_agent, "agent-a");
        assert_eq!(messages[0].content, "Hello from A");
    }

    #[tokio::test]
    async fn test_send_to_unregistered_agent() {
        let session_tools = SessionTools::new(100, Duration::from_secs(30 * 60));

        session_tools.register_agent("agent-a").await.unwrap();

        let message = SessionMessage::new(
            "agent-a",
            "agent-unknown",
            MessageType::Announcement,
            "Test",
            ChronoDuration::minutes(30),
        );

        let result = session_tools.send_message(message).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CoordinationProtocolError::AgentNotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_queue_capacity_exceeded() {
        let session_tools = SessionTools::new(2, Duration::from_secs(30 * 60)); // Small capacity

        session_tools.register_agent("agent-a").await.unwrap();
        session_tools.register_agent("agent-b").await.unwrap();

        // Fill the queue
        for i in 0..2 {
            let message = SessionMessage::new(
                "agent-a",
                "agent-b",
                MessageType::Announcement,
                format!("Message {}", i),
                ChronoDuration::minutes(30),
            );
            session_tools.send_message(message).await.unwrap();
        }

        // This should fail (queue full)
        let message = SessionMessage::new(
            "agent-a",
            "agent-b",
            MessageType::Announcement,
            "Overflow",
            ChronoDuration::minutes(30),
        );
        let result = session_tools.send_message(message).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CoordinationProtocolError::QueueFull { .. }
        ));
    }

    #[tokio::test]
    async fn test_drain_empty_queue() {
        let session_tools = SessionTools::new(100, Duration::from_secs(30 * 60));

        session_tools.register_agent("agent-a").await.unwrap();

        let messages = session_tools.drain_messages("agent-a").await;
        assert_eq!(messages.len(), 0);
    }

    #[tokio::test]
    async fn test_message_expiry() {
        let session_tools = SessionTools::new(100, Duration::from_millis(10));

        session_tools.register_agent("agent-a").await.unwrap();
        session_tools.register_agent("agent-b").await.unwrap();

        let message = SessionMessage::new(
            "agent-a",
            "agent-b",
            MessageType::Announcement,
            "Expires soon",
            ChronoDuration::milliseconds(1), // Very short TTL
        );

        session_tools.send_message(message).await.unwrap();

        // Sleep to ensure message expires
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Drain should filter out expired message
        let messages = session_tools.drain_messages("agent-b").await;
        assert_eq!(messages.len(), 0);
    }

    #[tokio::test]
    async fn test_unregister_drops_queue() {
        let session_tools = SessionTools::new(100, Duration::from_secs(30 * 60));

        session_tools.register_agent("agent-a").await.unwrap();
        session_tools.register_agent("agent-b").await.unwrap();

        let message = SessionMessage::new(
            "agent-a",
            "agent-b",
            MessageType::Announcement,
            "Test",
            ChronoDuration::minutes(30),
        );

        session_tools.send_message(message.clone()).await.unwrap();

        // Unregister agent-b
        session_tools.unregister_agent("agent-b").await;

        // Sending should now fail
        let result = session_tools.send_message(message).await;
        assert!(result.is_err());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_fire_and_forget_no_blocking() {
        let session_tools = SessionTools::new(100, Duration::from_secs(30 * 60));

        session_tools.register_agent("agent-a").await.unwrap();
        session_tools.register_agent("agent-b").await.unwrap();

        // Send many messages rapidly
        let mut handles = vec![];
        for i in 0..50 {
            let session_tools_clone = session_tools.clone();
            let handle = tokio::spawn(async move {
                let message = SessionMessage::new(
                    "agent-a",
                    "agent-b",
                    MessageType::Announcement,
                    format!("Message {}", i),
                    ChronoDuration::minutes(30),
                );
                session_tools_clone.send_message(message).await
            });
            handles.push(handle);
        }

        // Wait with timeout
        let timeout = Duration::from_secs(1);
        let result = tokio::time::timeout(timeout, async {
            for handle in handles {
                let _ = handle.await;
            }
        })
        .await;

        assert!(result.is_ok(), "Fire-and-forget should not block");
    }

    #[tokio::test]
    async fn test_multiple_senders_single_receiver() {
        let session_tools = SessionTools::new(100, Duration::from_secs(30 * 60));

        session_tools.register_agent("agent-a").await.unwrap();
        session_tools.register_agent("agent-b").await.unwrap();
        session_tools.register_agent("agent-c").await.unwrap();
        session_tools.register_agent("agent-d").await.unwrap();

        // A, B, C all send to D
        for from in &["agent-a", "agent-b", "agent-c"] {
            let message = SessionMessage::new(
                *from,
                "agent-d",
                MessageType::Announcement,
                format!("Hello from {}", from),
                ChronoDuration::minutes(30),
            );
            session_tools.send_message(message).await.unwrap();
        }

        let messages = session_tools.drain_messages("agent-d").await;
        assert_eq!(messages.len(), 3);

        // Verify all senders are represented
        let senders: Vec<_> = messages.iter().map(|m| m.from_agent.as_str()).collect();
        assert!(senders.contains(&"agent-a"));
        assert!(senders.contains(&"agent-b"));
        assert!(senders.contains(&"agent-c"));
    }

    #[tokio::test]
    async fn test_registered_agents_list() {
        let session_tools = SessionTools::new(100, Duration::from_secs(30 * 60));

        session_tools.register_agent("agent-a").await.unwrap();
        session_tools.register_agent("agent-b").await.unwrap();
        session_tools.register_agent("agent-c").await.unwrap();

        let agents = session_tools.registered_agents().await;
        assert_eq!(agents.len(), 3);
        assert!(agents.contains(&"agent-a".to_string()));
        assert!(agents.contains(&"agent-b".to_string()));
        assert!(agents.contains(&"agent-c".to_string()));
    }

    #[tokio::test]
    async fn test_idempotent_registration() {
        let session_tools = SessionTools::new(100, Duration::from_secs(30 * 60));

        session_tools.register_agent("agent-a").await.unwrap();
        session_tools.register_agent("agent-a").await.unwrap(); // Second registration

        let agents = session_tools.registered_agents().await;
        assert_eq!(agents.len(), 1); // Still only one agent
    }
}

// Implement Clone for SessionTools (for test helper)
impl Clone for SessionTools {
    fn clone(&self) -> Self {
        Self {
            inbound_senders: Arc::clone(&self.inbound_senders),
            inbound_receivers: Arc::clone(&self.inbound_receivers),
            capacity: self.capacity,
            ttl: self.ttl,
        }
    }
}
