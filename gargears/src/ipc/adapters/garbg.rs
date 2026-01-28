//! IPC adapter for garbg (wallpaper daemon)

use crate::ipc::client::{IpcClient, StandardResponse};
use crate::ipc::discovery::socket_path_for;
use crate::panels::Component;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

/// Commands for garbg (uses serde tag)
#[derive(Debug, Serialize)]
#[serde(tag = "command", rename_all = "snake_case")]
enum GarbgCommand {
    Status,
    Next {
        #[serde(skip_serializing_if = "Option::is_none")]
        monitor: Option<String>,
    },
    Prev {
        #[serde(skip_serializing_if = "Option::is_none")]
        monitor: Option<String>,
    },
    Pause,
    Resume,
    Toggle,
    QueryMonitors,
    QueryCurrent,
    Subscribe {
        events: Vec<String>,
    },
}

/// Adapter for garbg wallpaper daemon
pub struct GarbgAdapter {
    client: IpcClient,
    subscribed: bool,
}

impl GarbgAdapter {
    pub fn new() -> Self {
        use std::time::Duration;
        Self {
            client: IpcClient::new().with_timeout(Duration::from_secs(5)),
            subscribed: false,
        }
    }

    pub fn socket_path(&self) -> PathBuf {
        socket_path_for(Component::Garbg)
    }

    pub fn connect(&mut self) -> Result<()> {
        self.client.connect(&self.socket_path())
    }

    /// Attempt reconnection with exponential backoff
    pub fn reconnect(&mut self) -> Result<()> {
        self.subscribed = false;
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
        self.subscribed = false;
    }

    pub fn is_connected(&self) -> bool {
        self.client.is_connected()
    }

    /// Send a command
    fn send_command(&mut self, command: GarbgCommand) -> Result<Option<Value>> {
        let response: StandardResponse = self.client.send_receive(&command)?;
        response.into_result()
    }

    /// Get status
    pub fn status(&mut self) -> Result<GarbgStatus> {
        let data = self.send_command(GarbgCommand::Status)?;

        if let Some(data) = data {
            Ok(serde_json::from_value(data)?)
        } else {
            Ok(GarbgStatus::default())
        }
    }

    /// Query current wallpaper info
    pub fn query_current(&mut self) -> Result<Option<Value>> {
        self.send_command(GarbgCommand::QueryCurrent)
    }

    /// Query monitors
    pub fn query_monitors(&mut self) -> Result<Option<Value>> {
        self.send_command(GarbgCommand::QueryMonitors)
    }

    /// Next wallpaper
    pub fn next(&mut self, monitor: Option<String>) -> Result<()> {
        self.send_command(GarbgCommand::Next { monitor })?;
        Ok(())
    }

    /// Previous wallpaper
    pub fn prev(&mut self, monitor: Option<String>) -> Result<()> {
        self.send_command(GarbgCommand::Prev { monitor })?;
        Ok(())
    }

    /// Pause slideshow/animation
    pub fn pause(&mut self) -> Result<()> {
        self.send_command(GarbgCommand::Pause)?;
        Ok(())
    }

    /// Resume slideshow/animation
    pub fn resume(&mut self) -> Result<()> {
        self.send_command(GarbgCommand::Resume)?;
        Ok(())
    }

    /// Toggle pause state
    pub fn toggle(&mut self) -> Result<()> {
        self.send_command(GarbgCommand::Toggle)?;
        Ok(())
    }

    /// Subscribe to events
    pub fn subscribe(&mut self, events: &[&str]) -> Result<()> {
        let events: Vec<String> = events.iter().map(|s| s.to_string()).collect();
        self.send_command(GarbgCommand::Subscribe { events })?;
        self.subscribed = true;
        Ok(())
    }

    /// Poll for events
    pub fn poll_events(&mut self) -> Vec<GarbgEvent> {
        let mut events = Vec::new();

        while let Some(line) = self.client.try_read_line() {
            if let Ok(event) = serde_json::from_str::<GarbgEvent>(&line) {
                events.push(event);
            }
        }

        events
    }
}

impl Default for GarbgAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Garbg status
#[derive(Debug, Clone, Default, Deserialize)]
pub struct GarbgStatus {
    #[serde(default)]
    pub paused: bool,
    #[serde(default)]
    pub current_source: Option<String>,
    #[serde(default)]
    pub current_wallpaper: Option<String>,
    #[serde(default)]
    pub interval_secs: Option<u64>,
    #[serde(default)]
    pub playlist_index: Option<usize>,
    #[serde(default)]
    pub playlist_total: Option<usize>,
}

/// Events from garbg
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum GarbgEvent {
    WallpaperChanged {
        monitor: String,
        source: String,
        #[serde(default)]
        workspace: Option<usize>,
    },
    SourceUpdated {
        source: String,
        count: usize,
    },
    AnimationState {
        playing: bool,
    },
    SlideshowAdvanced {
        current: usize,
        total: usize,
        source: String,
    },
    Error {
        message: String,
        #[serde(default)]
        context: Option<String>,
    },
}
