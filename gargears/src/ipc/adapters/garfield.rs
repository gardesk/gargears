//! IPC adapter for garfield (file explorer)

use crate::ipc::client::{IpcClient, StandardResponse};
use crate::ipc::discovery::socket_path_for;
use crate::panels::Component;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

/// Commands for garfield
#[derive(Debug, Serialize)]
#[serde(tag = "command", rename_all = "snake_case")]
enum GarfieldCommand {
    Status,
    CurrentDir,
    Open { path: String },
}

/// Adapter for garfield file explorer
pub struct GarfieldAdapter {
    client: IpcClient,
}

impl GarfieldAdapter {
    pub fn new() -> Self {
        use std::time::Duration;
        Self {
            client: IpcClient::new().with_timeout(Duration::from_secs(5)),
        }
    }

    pub fn socket_path(&self) -> PathBuf {
        socket_path_for(Component::Garfield)
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

    fn send_command(&mut self, command: GarfieldCommand) -> Result<Option<Value>> {
        let response: StandardResponse = self.client.send_receive(&command)?;
        response.into_result()
    }

    pub fn status(&mut self) -> Result<GarfieldStatus> {
        let data = self.send_command(GarfieldCommand::Status)?;
        if let Some(data) = data {
            Ok(serde_json::from_value(data)?)
        } else {
            Ok(GarfieldStatus::default())
        }
    }

    pub fn current_dir(&mut self) -> Result<Option<String>> {
        let data = self.send_command(GarfieldCommand::CurrentDir)?;
        Ok(data.and_then(|v| v.as_str().map(String::from)))
    }

    pub fn open(&mut self, path: &str) -> Result<()> {
        self.send_command(GarfieldCommand::Open {
            path: path.to_string(),
        })?;
        Ok(())
    }
}

impl Default for GarfieldAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GarfieldStatus {
    #[serde(default)]
    pub running: bool,
    #[serde(default)]
    pub current_dir: Option<String>,
}
