//! Graceful shutdown handler for production deployments
//!
//! This module provides infrastructure for cleanly shutting down the AOF daemon:
//! 1. Listen for SIGTERM/SIGINT signals
//! 2. Stop accepting new connections
//! 3. Drain active WebSocket connections
//! 4. Save all session state
//! 5. Flush pending log entries
//! 6. Close metrics registry
//! 7. Exit within timeout (default: 30 seconds)

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{info, warn};

/// Graceful shutdown coordinator
pub struct GracefulShutdown {
    /// Broadcast channel to signal shutdown to all components
    shutdown_tx: broadcast::Sender<()>,
    /// Maximum time to wait for graceful shutdown before forcing exit
    timeout: Duration,
}

impl GracefulShutdown {
    /// Create a new graceful shutdown handler
    ///
    /// # Arguments
    /// * `timeout` - Maximum time to wait for graceful shutdown (e.g., Duration::from_secs(30))
    pub fn new(timeout: Duration) -> Self {
        let (shutdown_tx, _) = broadcast::channel(16);
        Self {
            shutdown_tx,
            timeout,
        }
    }

    /// Get a receiver that signals when shutdown starts
    ///
    /// Components should subscribe to this channel and begin their own shutdown
    /// procedures when they receive the signal.
    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }

    /// Wait for SIGTERM or SIGINT signal
    ///
    /// This is typically used with `axum::serve(...).with_graceful_shutdown(shutdown.wait_for_signal())`
    pub async fn wait_for_signal(&self) {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};

            let mut sigterm = signal(SignalKind::terminate())
                .expect("Failed to install SIGTERM handler");
            let mut sigint = signal(SignalKind::interrupt())
                .expect("Failed to install SIGINT handler");

            tokio::select! {
                _ = sigterm.recv() => {
                    info!("Received SIGTERM, starting graceful shutdown");
                }
                _ = sigint.recv() => {
                    info!("Received SIGINT (Ctrl+C), starting graceful shutdown");
                }
            }
        }

        #[cfg(not(unix))]
        {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");
            info!("Received Ctrl+C, starting graceful shutdown");
        }

        // Broadcast shutdown signal to all subscribers
        let subscriber_count = self.shutdown_tx.send(()).unwrap_or(0);
        info!(subscribers = subscriber_count, "Broadcasting shutdown signal to components");
    }

    /// Execute graceful shutdown sequence with timeout
    ///
    /// This method orchestrates the shutdown of all AOF components:
    /// 1. WebSocket connections are drained (close frames sent)
    /// 2. Session state is persisted
    /// 3. Pending log entries are flushed
    /// 4. Metrics are finalized
    ///
    /// If shutdown doesn't complete within timeout, the process exits forcefully.
    pub async fn execute<S>(&self, state: Arc<S>) -> crate::AofResult<()>
    where
        S: ShutdownHandler,
    {
        info!(timeout_secs = self.timeout.as_secs(), "Starting graceful shutdown sequence");

        // Run shutdown with timeout
        match tokio::time::timeout(self.timeout, state.shutdown()).await {
            Ok(Ok(())) => {
                info!("Graceful shutdown completed successfully");
                Ok(())
            }
            Ok(Err(e)) => {
                warn!(error = %e, "Graceful shutdown encountered errors");
                Err(e)
            }
            Err(_) => {
                warn!(timeout_secs = self.timeout.as_secs(), "Graceful shutdown timed out, forcing exit");
                Err(crate::AofError::internal(format!(
                    "Shutdown timeout after {} seconds",
                    self.timeout.as_secs()
                )))
            }
        }
    }
}

/// Trait for components that need to shut down gracefully
///
/// Implement this trait for your application state to customize shutdown behavior.
#[async_trait::async_trait]
pub trait ShutdownHandler: Send + Sync {
    /// Perform graceful shutdown
    ///
    /// This method should:
    /// - Close active connections
    /// - Save state to disk
    /// - Release resources
    /// - Flush buffers
    async fn shutdown(&self) -> crate::AofResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestShutdownHandler {
        shutdown_called: Arc<tokio::sync::Mutex<bool>>,
    }

    #[async_trait::async_trait]
    impl ShutdownHandler for TestShutdownHandler {
        async fn shutdown(&self) -> crate::AofResult<()> {
            let mut called = self.shutdown_called.lock().await;
            *called = true;
            info!("Test shutdown handler executed");
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_shutdown_creation() {
        let shutdown = GracefulShutdown::new(Duration::from_secs(30));
        assert!(shutdown.timeout == Duration::from_secs(30));
    }

    #[tokio::test]
    async fn test_shutdown_subscribe() {
        let shutdown = GracefulShutdown::new(Duration::from_secs(30));
        let mut rx1 = shutdown.subscribe();
        let mut rx2 = shutdown.subscribe();

        // Send shutdown signal
        shutdown.shutdown_tx.send(()).unwrap();

        // Both receivers should get the signal
        assert!(rx1.try_recv().is_ok());
        assert!(rx2.try_recv().is_ok());
    }

    #[tokio::test]
    async fn test_shutdown_execute_success() {
        let shutdown = GracefulShutdown::new(Duration::from_secs(5));
        let shutdown_called = Arc::new(tokio::sync::Mutex::new(false));
        let handler = Arc::new(TestShutdownHandler {
            shutdown_called: Arc::clone(&shutdown_called),
        });

        let result = shutdown.execute(handler).await;
        assert!(result.is_ok());

        // Verify shutdown was called
        let called = shutdown_called.lock().await;
        assert!(*called);
    }

    #[tokio::test]
    async fn test_shutdown_execute_timeout() {
        struct SlowShutdownHandler;

        #[async_trait::async_trait]
        impl ShutdownHandler for SlowShutdownHandler {
            async fn shutdown(&self) -> crate::AofResult<()> {
                // Simulate slow shutdown that exceeds timeout
                tokio::time::sleep(Duration::from_secs(10)).await;
                Ok(())
            }
        }

        let shutdown = GracefulShutdown::new(Duration::from_secs(1));
        let handler = Arc::new(SlowShutdownHandler);

        let result = shutdown.execute(handler).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timeout"));
    }
}
