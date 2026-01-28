//! IPC adapter for garnotify (notification daemon)

use crate::ipc::client::{IpcClient, StandardResponse};
use crate::ipc::discovery::socket_path_for;
use crate::panels::Component;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

/// Commands for garnotify
#[derive(Debug, Serialize)]
#[serde(tag = "command", rename_all = "snake_case")]
enum GarnotifyCommand {
    Status,
    Clear,
    ClearAll,
    Pause,
    Resume,
    Toggle,
}

/// Adapter for garnotify notification daemon
pub struct GarnotifyAdapter {
    client: IpcClient,
}

impl GarnotifyAdapter {
    pub fn new() -> Self {
        use std::time::Duration;
        Self {
            client: IpcClient::new().with_timeout(Duration::from_secs(5)),
        }
    }

    pub fn socket_path(&self) -> PathBuf {
        socket_path_for(Component::Garnotify)
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

    fn send_command(&mut self, command: GarnotifyCommand) -> Result<Option<Value>> {
        let response: StandardResponse = self.client.send_receive(&command)?;
        response.into_result()
    }

    pub fn status(&mut self) -> Result<GarnotifyStatus> {
        let data = self.send_command(GarnotifyCommand::Status)?;
        if let Some(data) = data {
            Ok(serde_json::from_value(data)?)
        } else {
            Ok(GarnotifyStatus::default())
        }
    }

    pub fn clear(&mut self) -> Result<()> {
        self.send_command(GarnotifyCommand::Clear)?;
        Ok(())
    }

    pub fn clear_all(&mut self) -> Result<()> {
        self.send_command(GarnotifyCommand::ClearAll)?;
        Ok(())
    }

    pub fn pause(&mut self) -> Result<()> {
        self.send_command(GarnotifyCommand::Pause)?;
        Ok(())
    }

    pub fn resume(&mut self) -> Result<()> {
        self.send_command(GarnotifyCommand::Resume)?;
        Ok(())
    }

    pub fn toggle(&mut self) -> Result<()> {
        self.send_command(GarnotifyCommand::Toggle)?;
        Ok(())
    }
}

impl Default for GarnotifyAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GarnotifyStatus {
    #[serde(default)]
    pub paused: bool,
    #[serde(default)]
    pub pending_count: usize,
    #[serde(default)]
    pub history_count: usize,
}
