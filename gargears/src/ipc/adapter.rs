//! IPC adapter trait for communicating with gardesk daemons

use crate::panels::Component;
use anyhow::Result;
use std::path::PathBuf;

/// Status of a component
#[derive(Debug, Clone)]
pub struct ComponentStatus {
    /// Whether the daemon is running
    pub running: bool,
    /// Current configuration values (component-specific)
    pub config: serde_json::Value,
}

/// Trait for IPC adapters to gardesk components
pub trait IpcAdapter: Send {
    /// Get the component this adapter is for
    fn component(&self) -> Component;

    /// Get the socket path for this component
    fn socket_path(&self) -> PathBuf;

    /// Connect to the daemon
    fn connect(&mut self) -> Result<()>;

    /// Disconnect from the daemon
    fn disconnect(&mut self);

    /// Check if connected
    fn is_connected(&self) -> bool;

    /// Query the daemon's current status/config
    fn status(&mut self) -> Result<ComponentStatus>;

    /// Send a command to the daemon
    fn send_command(&mut self, command: &str, args: serde_json::Value) -> Result<serde_json::Value>;

    /// Subscribe to events from the daemon (if supported)
    fn subscribe(&mut self, _events: &[&str]) -> Result<()> {
        // Default: no-op for daemons that don't support subscriptions
        Ok(())
    }

    /// Check for pending events (non-blocking)
    fn poll_events(&mut self) -> Vec<serde_json::Value> {
        // Default: no events
        Vec::new()
    }
}
