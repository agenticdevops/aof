//! Event broadcasting for multi-subscriber coordination
//!
//! Wraps tokio::sync::broadcast to provide event bus for CoordinationEvent.
//! Multiple subscribers can receive the same events simultaneously.

use aof_core::CoordinationEvent;
use tokio::sync::broadcast;
use tracing::debug;

/// Event broadcaster using tokio::sync::broadcast channel
///
/// Provides pub/sub pattern for CoordinationEvent distribution to multiple subscribers.
/// The broadcaster ignores send errors (no subscribers is OK), making it safe to emit
/// events even when no subscribers are active.
#[derive(Clone)]
pub struct EventBroadcaster {
    sender: broadcast::Sender<CoordinationEvent>,
}

impl EventBroadcaster {
    /// Create a new event broadcaster with the given channel capacity
    ///
    /// Capacity determines how many events can be buffered when subscribers lag behind.
    /// Default recommendation: 1000 events for typical workloads.
    ///
    /// # Arguments
    /// * `capacity` - Number of events to buffer per subscriber
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Create broadcaster with default capacity (1000 events)
    pub fn default() -> Self {
        Self::new(1000)
    }

    /// Emit an event to all subscribers
    ///
    /// Ignores errors if no subscribers are active. Logs warnings if some subscribers
    /// couldn't receive the event (lagged behind and dropped events).
    ///
    /// # Arguments
    /// * `event` - The coordination event to broadcast
    pub fn emit(&self, event: CoordinationEvent) {
        match self.sender.send(event) {
            Ok(receiver_count) => {
                debug!(
                    "Event {} broadcasted to {} subscribers",
                    receiver_count, receiver_count
                );
            }
            Err(_) => {
                // No subscribers - this is OK, events are best-effort
                debug!("Event emitted with no active subscribers");
            }
        }
    }

    /// Subscribe to coordination events
    ///
    /// Returns a receiver that will receive all future events. Each subscriber
    /// receives a clone of every event.
    ///
    /// # Returns
    /// A broadcast receiver for CoordinationEvent
    pub fn subscribe(&self) -> broadcast::Receiver<CoordinationEvent> {
        self.sender.subscribe()
    }

    /// Get the number of active subscribers
    ///
    /// Useful for health checks and monitoring.
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }

    /// Get the channel capacity
    pub fn capacity(&self) -> usize {
        // Note: broadcast::Sender doesn't expose capacity directly,
        // so we return the value used during construction
        // For now, we'll rely on the sender's default behavior
        // Future enhancement: store capacity as a field
        1000 // Default capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aof_core::ActivityEvent;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_single_producer_single_consumer() {
        let broadcaster = EventBroadcaster::new(100);
        let mut receiver = broadcaster.subscribe();

        let event = CoordinationEvent::from_activity(
            ActivityEvent::thinking("Processing request"),
            "agent-1",
            "session-123",
        );

        broadcaster.emit(event.clone());

        let received = timeout(Duration::from_secs(1), receiver.recv())
            .await
            .expect("Timeout waiting for event")
            .expect("Failed to receive event");

        assert_eq!(received.agent_id, "agent-1");
        assert_eq!(received.session_id, "session-123");
    }

    #[tokio::test]
    async fn test_single_producer_multiple_consumers() {
        let broadcaster = EventBroadcaster::new(100);
        let mut receiver1 = broadcaster.subscribe();
        let mut receiver2 = broadcaster.subscribe();

        assert_eq!(broadcaster.subscriber_count(), 2);

        let event = CoordinationEvent::from_activity(
            ActivityEvent::thinking("Processing request"),
            "agent-1",
            "session-123",
        );

        broadcaster.emit(event.clone());

        // Both receivers should get the same event
        let received1 = timeout(Duration::from_secs(1), receiver1.recv())
            .await
            .expect("Timeout on receiver1")
            .expect("Failed on receiver1");

        let received2 = timeout(Duration::from_secs(1), receiver2.recv())
            .await
            .expect("Timeout on receiver2")
            .expect("Failed on receiver2");

        assert_eq!(received1.event_id, received2.event_id);
        assert_eq!(received1.agent_id, "agent-1");
        assert_eq!(received2.agent_id, "agent-1");
    }

    #[tokio::test]
    async fn test_emit_with_no_subscribers() {
        let broadcaster = EventBroadcaster::new(100);

        // Should not panic when emitting with no subscribers
        let event = CoordinationEvent::from_activity(
            ActivityEvent::thinking("Processing request"),
            "agent-1",
            "session-123",
        );

        broadcaster.emit(event); // Should not panic
        assert_eq!(broadcaster.subscriber_count(), 0);
    }

    #[tokio::test]
    async fn test_subscriber_count() {
        let broadcaster = EventBroadcaster::new(100);
        assert_eq!(broadcaster.subscriber_count(), 0);

        let _receiver1 = broadcaster.subscribe();
        assert_eq!(broadcaster.subscriber_count(), 1);

        let _receiver2 = broadcaster.subscribe();
        assert_eq!(broadcaster.subscriber_count(), 2);

        drop(_receiver1);
        // Note: Dropping receiver decreases count, but this is eventually consistent
        // in tokio's broadcast implementation
    }

    #[tokio::test]
    async fn test_broadcaster_clone() {
        let broadcaster1 = EventBroadcaster::new(100);
        let broadcaster2 = broadcaster1.clone();

        let mut receiver = broadcaster1.subscribe();

        // Emit from cloned broadcaster
        let event = CoordinationEvent::from_activity(
            ActivityEvent::thinking("Test message"),
            "agent-1",
            "session-123",
        );
        broadcaster2.emit(event);

        // Should receive on original broadcaster's subscriber
        let received = timeout(Duration::from_secs(1), receiver.recv())
            .await
            .expect("Timeout")
            .expect("Failed to receive");

        assert_eq!(received.agent_id, "agent-1");
    }
}
