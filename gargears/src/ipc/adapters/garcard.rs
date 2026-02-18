//! IPC adapter for garcard (authentication agent)

use crate::ipc::client::IpcClient;
use crate::ipc::discovery::socket_path_for;
use crate::panels::Component;
use anyhow::Result;
use std::path::PathBuf;

/// Adapter for garcard authentication agent
pub struct GarcardAdapter {
    client: IpcClient,
}

impl GarcardAdapter {
    pub fn new() -> Self {
        use std::time::Duration;
        Self {
            client: IpcClient::new().with_timeout(Duration::from_secs(5)),
        }
    }

    pub fn socket_path(&self) -> PathBuf {
        socket_path_for(Component::Garcard)
    }

    pub fn connect(&mut self) -> Result<()> {
        self.client.connect(&self.socket_path())
    }

    /// Attempt reconnection with exponential backoff
    pub fn reconnect(&mut self) -> Result<()> {
        self.client.reconnect()
    }

    /// Check if we should attempt reconnection
    pub fn should_reconnect(&self) -> bool {
        self.client.should_attempt_reconnect()
    }

    /// Reset backoff state
    pub fn reset_backoff(&mut self) {
        self.client.reset_backoff();
    }

    pub fn disconnect(&mut self) {
        self.client.disconnect();
    }

    pub fn is_connected(&self) -> bool {
        self.client.is_connected()
    }
}

impl Default for GarcardAdapter {
    fn default() -> Self {
        Self::new()
    }
}
