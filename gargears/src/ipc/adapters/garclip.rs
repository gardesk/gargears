//! IPC adapter for garclip (clipboard manager)

use crate::ipc::client::{IpcClient, StandardResponse};
use crate::ipc::discovery::socket_path_for;
use crate::panels::Component;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

/// Commands for garclip
#[derive(Debug, Serialize)]
#[serde(tag = "command", rename_all = "snake_case")]
enum GarclipCommand {
    Status,
    History { limit: usize },
    Clear,
    ClearHistory { keep_pinned: bool },
    Reload,
}

/// Adapter for garclip clipboard manager
pub struct GarclipAdapter {
    client: IpcClient,
}

impl GarclipAdapter {
    pub fn new() -> Self {
        use std::time::Duration;
        Self {
            client: IpcClient::new().with_timeout(Duration::from_secs(5)),
        }
    }

    pub fn socket_path(&self) -> PathBuf {
        socket_path_for(Component::Garclip)
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

    fn send_command(&mut self, command: GarclipCommand) -> Result<Option<Value>> {
        let response: StandardResponse = self.client.send_receive(&command)?;
        response.into_result()
    }

    pub fn status(&mut self) -> Result<GarclipStatus> {
        let data = self.send_command(GarclipCommand::Status)?;
        if let Some(data) = data {
            Ok(serde_json::from_value(data)?)
        } else {
            Ok(GarclipStatus::default())
        }
    }

    pub fn history(&mut self, limit: usize) -> Result<Option<Value>> {
        self.send_command(GarclipCommand::History { limit })
    }

    pub fn clear(&mut self) -> Result<()> {
        self.send_command(GarclipCommand::Clear)?;
        Ok(())
    }

    pub fn clear_history(&mut self, keep_pinned: bool) -> Result<()> {
        self.send_command(GarclipCommand::ClearHistory { keep_pinned })?;
        Ok(())
    }
}

impl Default for GarclipAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GarclipStatus {
    #[serde(default)]
    pub history_count: usize,
    #[serde(default)]
    pub pinned_count: usize,
    #[serde(default)]
    pub owns_clipboard: bool,
    #[serde(default)]
    pub watching_primary: bool,
}
