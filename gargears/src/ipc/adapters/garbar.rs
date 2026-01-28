//! IPC adapter for garbar (status bar)

use crate::ipc::client::{IpcClient, StandardResponse};
use crate::ipc::discovery::socket_path_for;
use crate::panels::Component;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

/// Commands for garbar
#[derive(Debug, Serialize)]
#[serde(tag = "command", rename_all = "snake_case")]
enum GarbarCommand {
    Show,
    Hide,
    Toggle,
    Reload,
    Quit,
    Status,
    UpdateModule { module: String },
}

/// Adapter for garbar status bar
pub struct GarbarAdapter {
    client: IpcClient,
}

impl GarbarAdapter {
    pub fn new() -> Self {
        use std::time::Duration;
        Self {
            client: IpcClient::new().with_timeout(Duration::from_secs(5)),
        }
    }

    pub fn socket_path(&self) -> PathBuf {
        socket_path_for(Component::Garbar)
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

    /// Send a command
    fn send_command(&mut self, command: GarbarCommand) -> Result<Option<Value>> {
        let response: StandardResponse = self.client.send_receive(&command)?;
        response.into_result()
    }

    /// Get status
    pub fn status(&mut self) -> Result<GarbarStatus> {
        let data = self.send_command(GarbarCommand::Status)?;

        if let Some(data) = data {
            Ok(serde_json::from_value(data)?)
        } else {
            Ok(GarbarStatus::default())
        }
    }

    /// Show the bar
    pub fn show(&mut self) -> Result<()> {
        self.send_command(GarbarCommand::Show)?;
        Ok(())
    }

    /// Hide the bar
    pub fn hide(&mut self) -> Result<()> {
        self.send_command(GarbarCommand::Hide)?;
        Ok(())
    }

    /// Toggle visibility
    pub fn toggle(&mut self) -> Result<()> {
        self.send_command(GarbarCommand::Toggle)?;
        Ok(())
    }

    /// Reload configuration
    pub fn reload(&mut self) -> Result<()> {
        self.send_command(GarbarCommand::Reload)?;
        Ok(())
    }

    /// Update a specific module
    pub fn update_module(&mut self, module: &str) -> Result<()> {
        self.send_command(GarbarCommand::UpdateModule {
            module: module.to_string(),
        })?;
        Ok(())
    }
}

impl Default for GarbarAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Garbar status
#[derive(Debug, Clone, Default, Deserialize)]
pub struct GarbarStatus {
    #[serde(default)]
    pub visible: bool,
    #[serde(default)]
    pub width: u16,
    #[serde(default)]
    pub height: u16,
    #[serde(default)]
    pub position: Option<String>,
    #[serde(default)]
    pub modules_left: Vec<String>,
    #[serde(default)]
    pub modules_center: Vec<String>,
    #[serde(default)]
    pub modules_right: Vec<String>,
}
