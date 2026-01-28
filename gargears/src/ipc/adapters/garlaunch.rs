//! IPC adapter for garlaunch (application launcher)

use crate::ipc::client::{IpcClient, StandardResponse};
use crate::ipc::discovery::socket_path_for;
use crate::panels::Component;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

/// Commands for garlaunch
#[derive(Debug, Serialize)]
struct GarlaunchRequest {
    command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
}

/// Adapter for garlaunch application launcher
pub struct GarlaunchAdapter {
    client: IpcClient,
}

impl GarlaunchAdapter {
    pub fn new() -> Self {
        use std::time::Duration;
        Self {
            client: IpcClient::new().with_timeout(Duration::from_secs(5)),
        }
    }

    pub fn socket_path(&self) -> PathBuf {
        socket_path_for(Component::Garlaunch)
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

    fn send_command(&mut self, command: &str, mode: Option<&str>) -> Result<Option<Value>> {
        let request = GarlaunchRequest {
            command: command.to_string(),
            mode: mode.map(String::from),
            source: None,
        };
        let response: StandardResponse = self.client.send_receive(&request)?;
        response.into_result()
    }

    pub fn status(&mut self) -> Result<GarlaunchStatus> {
        let data = self.send_command("status", None)?;
        if let Some(data) = data {
            Ok(serde_json::from_value(data)?)
        } else {
            Ok(GarlaunchStatus::default())
        }
    }

    pub fn show(&mut self, mode: Option<&str>) -> Result<()> {
        self.send_command("show", mode)?;
        Ok(())
    }

    pub fn toggle(&mut self, mode: Option<&str>) -> Result<()> {
        self.send_command("toggle", mode)?;
        Ok(())
    }
}

impl Default for GarlaunchAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GarlaunchStatus {
    #[serde(default)]
    pub daemon_running: bool,
}
