//! IPC adapter for garterm (terminal emulator)

use crate::ipc::client::{IpcClient, StandardResponse};
use crate::panels::Component;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

/// Commands for garterm
#[derive(Debug, Serialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
enum GartermCommand {
    Status,
    NewWindow,
    NewTab,
}

/// Adapter for garterm terminal emulator
///
/// Note: garterm uses per-instance sockets (garterm-{PID}.sock)
/// We connect to the focused instance via the "focused" file
pub struct GartermAdapter {
    client: IpcClient,
    focused_pid: Option<u32>,
}

impl GartermAdapter {
    pub fn new() -> Self {
        use std::time::Duration;
        Self {
            client: IpcClient::new().with_timeout(Duration::from_secs(5)),
            focused_pid: None,
        }
    }

    /// Get the directory containing garterm sockets
    fn socket_dir() -> PathBuf {
        let runtime_dir =
            std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(runtime_dir).join("garterm")
    }

    /// Get the focused PID file path
    fn focused_file() -> PathBuf {
        Self::socket_dir().join("focused")
    }

    /// Get socket path for a specific PID
    fn socket_for_pid(pid: u32) -> PathBuf {
        Self::socket_dir().join(format!("garterm-{}.sock", pid))
    }

    pub fn socket_path(&self) -> PathBuf {
        // Return the focused file path for discovery purposes
        Self::focused_file()
    }

    /// Read the focused PID
    fn read_focused_pid() -> Option<u32> {
        std::fs::read_to_string(Self::focused_file())
            .ok()
            .and_then(|s| s.trim().parse().ok())
    }

    pub fn connect(&mut self) -> Result<()> {
        // Read the focused PID and connect to that instance
        let pid = Self::read_focused_pid()
            .ok_or_else(|| anyhow::anyhow!("No focused garterm instance"))?;

        let socket_path = Self::socket_for_pid(pid);
        self.client.connect(&socket_path)?;
        self.focused_pid = Some(pid);

        Ok(())
    }

    pub fn disconnect(&mut self) {
        self.client.disconnect();
        self.focused_pid = None;
    }

    pub fn is_connected(&self) -> bool {
        self.client.is_connected()
    }

    /// List all running garterm instances
    pub fn list_instances() -> Vec<u32> {
        let socket_dir = Self::socket_dir();
        if !socket_dir.exists() {
            return Vec::new();
        }

        std::fs::read_dir(socket_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| {
                        let name = e.file_name().to_string_lossy().to_string();
                        if name.starts_with("garterm-") && name.ends_with(".sock") {
                            name.strip_prefix("garterm-")
                                .and_then(|s| s.strip_suffix(".sock"))
                                .and_then(|s| s.parse().ok())
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn send_command(&mut self, command: GartermCommand) -> Result<Option<Value>> {
        let response: StandardResponse = self.client.send_receive(&command)?;
        response.into_result()
    }

    pub fn status(&mut self) -> Result<GartermStatus> {
        let data = self.send_command(GartermCommand::Status)?;
        if let Some(data) = data {
            Ok(serde_json::from_value(data)?)
        } else {
            Ok(GartermStatus {
                pid: self.focused_pid,
                ..Default::default()
            })
        }
    }

    pub fn new_window(&mut self) -> Result<()> {
        self.send_command(GartermCommand::NewWindow)?;
        Ok(())
    }

    pub fn new_tab(&mut self) -> Result<()> {
        self.send_command(GartermCommand::NewTab)?;
        Ok(())
    }
}

impl Default for GartermAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GartermStatus {
    #[serde(default)]
    pub pid: Option<u32>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub tabs: Option<usize>,
}
