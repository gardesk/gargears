//! IPC adapter for gartray (system tray & quick settings)

use crate::ipc::client::{IpcClient, StandardResponse};
use crate::ipc::discovery::socket_path_for;
use crate::panels::Component;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

/// Commands for gartray
#[derive(Debug, Serialize)]
#[serde(tag = "command", rename_all = "snake_case")]
enum GartrayCommand {
    Status,
    Show,
    Hide,
    Toggle,
}

/// Adapter for gartray system tray
pub struct GartrayAdapter {
    client: IpcClient,
}

impl GartrayAdapter {
    pub fn new() -> Self {
        use std::time::Duration;
        Self {
            client: IpcClient::new().with_timeout(Duration::from_secs(5)),
        }
    }

    pub fn socket_path(&self) -> PathBuf {
        socket_path_for(Component::Gartray)
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

    fn send_command(&mut self, command: GartrayCommand) -> Result<Option<Value>> {
        let response: StandardResponse = self.client.send_receive(&command)?;
        response.into_result()
    }

    pub fn status(&mut self) -> Result<GartrayStatus> {
        let data = self.send_command(GartrayCommand::Status)?;
        if let Some(data) = data {
            Ok(serde_json::from_value(data)?)
        } else {
            Ok(GartrayStatus::default())
        }
    }

    pub fn show(&mut self) -> Result<()> {
        self.send_command(GartrayCommand::Show)?;
        Ok(())
    }

    pub fn hide(&mut self) -> Result<()> {
        self.send_command(GartrayCommand::Hide)?;
        Ok(())
    }

    pub fn toggle(&mut self) -> Result<()> {
        self.send_command(GartrayCommand::Toggle)?;
        Ok(())
    }
}

impl Default for GartrayAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GartrayStatus {
    #[serde(default)]
    pub visible: bool,
    #[serde(default)]
    pub tray_icons: usize,
}
