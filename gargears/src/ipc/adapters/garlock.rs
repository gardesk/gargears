//! IPC adapter for garlock (screen locker)

use crate::ipc::client::{IpcClient, StandardResponse};
use crate::ipc::discovery::socket_path_for;
use crate::panels::Component;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

/// Commands for garlock
#[derive(Debug, Serialize)]
#[serde(tag = "command", rename_all = "snake_case")]
enum GarlockCommand {
    Status,
    Lock,
}

/// Adapter for garlock screen locker
pub struct GarlockAdapter {
    client: IpcClient,
}

impl GarlockAdapter {
    pub fn new() -> Self {
        use std::time::Duration;
        Self {
            client: IpcClient::new().with_timeout(Duration::from_secs(5)),
        }
    }

    pub fn socket_path(&self) -> PathBuf {
        socket_path_for(Component::Garlock)
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

    fn send_command(&mut self, command: GarlockCommand) -> Result<Option<Value>> {
        let response: StandardResponse = self.client.send_receive(&command)?;
        response.into_result()
    }

    pub fn status(&mut self) -> Result<GarlockStatus> {
        let data = self.send_command(GarlockCommand::Status)?;
        if let Some(data) = data {
            Ok(serde_json::from_value(data)?)
        } else {
            Ok(GarlockStatus::default())
        }
    }

    pub fn lock(&mut self) -> Result<()> {
        self.send_command(GarlockCommand::Lock)?;
        Ok(())
    }
}

impl Default for GarlockAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GarlockStatus {
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub idle_timeout_secs: Option<u64>,
}
