//! IPC adapter for gar (window manager)

use crate::ipc::client::{IpcClient, StandardResponse};
use crate::ipc::discovery::socket_path_for;
use crate::panels::Component;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;

/// Request format for gar IPC
#[derive(Debug, Serialize)]
struct GarRequest {
    command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    args: Option<Value>,
}

/// Adapter for gar window manager
pub struct GarAdapter {
    client: IpcClient,
    subscribed: bool,
}

impl GarAdapter {
    pub fn new() -> Self {
        use std::time::Duration;
        Self {
            client: IpcClient::new().with_timeout(Duration::from_secs(5)),
            subscribed: false,
        }
    }

    pub fn socket_path(&self) -> PathBuf {
        socket_path_for(Component::Gar)
    }

    pub fn connect(&mut self) -> Result<()> {
        self.client.connect(&self.socket_path())
    }

    /// Attempt reconnection with exponential backoff
    pub fn reconnect(&mut self) -> Result<()> {
        self.subscribed = false;
        self.client.reconnect()
    }

    /// Check if we should attempt reconnection (backoff elapsed)
    pub fn should_reconnect(&self) -> bool {
        self.client.should_attempt_reconnect()
    }

    /// Reset backoff state (e.g., on manual refresh)
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

    /// Send a command to gar
    fn send_command(&mut self, command: &str, args: Option<Value>) -> Result<Option<Value>> {
        let request = GarRequest {
            command: command.to_string(),
            args,
        };

        let response: StandardResponse = self.client.send_receive(&request)?;
        response.into_result()
    }

    /// Get gar status
    pub fn status(&mut self) -> Result<GarStatus> {
        // Query workspaces
        let workspaces = self.send_command("workspaces", None)?;

        // Query focused window
        let focused = self.send_command("focused", None)?;

        // Query config values
        let config = self.query_config()?;

        Ok(GarStatus {
            workspaces: workspaces.unwrap_or(Value::Array(vec![])),
            focused_window: focused,
            config,
        })
    }

    /// Query configuration values
    pub fn query_config(&mut self) -> Result<GarConfig> {
        // These are the main configurable values
        let border_width = self.get_config_value("border_width")?;
        let gap_inner = self.get_config_value("gap_inner")?;
        let gap_outer = self.get_config_value("gap_outer")?;
        let border_color_focused = self.get_config_value("border_color_focused")?;
        let border_color_unfocused = self.get_config_value("border_color_unfocused")?;
        let focus_follows_mouse = self.get_config_value("focus_follows_mouse")?;
        let mouse_follows_focus = self.get_config_value("mouse_follows_focus")?;
        let titlebar_enabled = self.get_config_value("titlebar_enabled")?;
        let titlebar_height = self.get_config_value("titlebar_height")?;

        Ok(GarConfig {
            border_width: border_width.and_then(|v| v.as_u64()).map(|v| v as u32),
            gap_inner: gap_inner.and_then(|v| v.as_u64()).map(|v| v as u32),
            gap_outer: gap_outer.and_then(|v| v.as_u64()).map(|v| v as u32),
            border_color_focused: border_color_focused.and_then(|v| v.as_str().map(String::from)),
            border_color_unfocused: border_color_unfocused
                .and_then(|v| v.as_str().map(String::from)),
            focus_follows_mouse: focus_follows_mouse.and_then(|v| v.as_bool()),
            mouse_follows_focus: mouse_follows_focus.and_then(|v| v.as_bool()),
            titlebar_enabled: titlebar_enabled.and_then(|v| v.as_bool()),
            titlebar_height: titlebar_height.and_then(|v| v.as_u64()).map(|v| v as u32),
        })
    }

    /// Get a single config value
    fn get_config_value(&mut self, key: &str) -> Result<Option<Value>> {
        self.send_command("get", Some(json!({ "key": key })))
    }

    /// Set a config value
    pub fn set_config_value(&mut self, key: &str, value: Value) -> Result<()> {
        self.send_command("set", Some(json!({ "key": key, "value": value })))?;
        Ok(())
    }

    /// Query keybinds
    pub fn query_keybinds(&mut self) -> Result<Vec<GarKeybind>> {
        let result = self.send_command("keybinds", None)?;
        if let Some(data) = result {
            Ok(serde_json::from_value(data).unwrap_or_default())
        } else {
            Ok(vec![])
        }
    }

    /// Query window rules
    pub fn query_rules(&mut self) -> Result<Vec<GarRule>> {
        let result = self.send_command("rules", None)?;
        if let Some(data) = result {
            Ok(serde_json::from_value(data).unwrap_or_default())
        } else {
            Ok(vec![])
        }
    }

    /// Subscribe to events
    pub fn subscribe(&mut self, events: &[&str]) -> Result<()> {
        let events: Vec<String> = events.iter().map(|s| s.to_string()).collect();
        self.send_command("subscribe", Some(json!({ "events": events })))?;
        self.subscribed = true;
        Ok(())
    }

    /// Poll for events (non-blocking)
    pub fn poll_events(&mut self) -> Vec<GarEvent> {
        let mut events = Vec::new();

        while let Some(line) = self.client.try_read_line() {
            if let Ok(event) = serde_json::from_str::<GarEventRaw>(&line) {
                events.push(GarEvent {
                    event_type: event.event,
                    data: event.data,
                });
            }
        }

        events
    }
}

impl Default for GarAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Gar status information
#[derive(Debug, Clone)]
pub struct GarStatus {
    pub workspaces: Value,
    pub focused_window: Option<Value>,
    pub config: GarConfig,
}

/// Gar configuration values
#[derive(Debug, Clone, Default)]
pub struct GarConfig {
    pub border_width: Option<u32>,
    pub gap_inner: Option<u32>,
    pub gap_outer: Option<u32>,
    pub border_color_focused: Option<String>,
    pub border_color_unfocused: Option<String>,
    pub focus_follows_mouse: Option<bool>,
    pub mouse_follows_focus: Option<bool>,
    pub titlebar_enabled: Option<bool>,
    pub titlebar_height: Option<u32>,
}

/// Raw event from gar
#[derive(Debug, Deserialize)]
struct GarEventRaw {
    event: String,
    data: Value,
}

/// Parsed gar event
#[derive(Debug, Clone)]
pub struct GarEvent {
    pub event_type: String,
    pub data: Value,
}

/// A keybind from gar
#[derive(Debug, Clone, Default, Deserialize)]
pub struct GarKeybind {
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub modifiers: Vec<String>,
    #[serde(default)]
    pub action: String,
}

impl GarKeybind {
    /// Format as display string (e.g., "mod+Return → exec kitty")
    pub fn display(&self) -> String {
        let mods = self.modifiers.join("+");
        if mods.is_empty() {
            format!("{} → {}", self.key, self.action)
        } else {
            format!("{}+{} → {}", mods, self.key, self.action)
        }
    }
}

/// A window rule from gar
#[derive(Debug, Clone, Default, Deserialize)]
pub struct GarRule {
    #[serde(default)]
    pub class: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub workspace: Option<u32>,
    #[serde(default)]
    pub floating: Option<bool>,
}

impl GarRule {
    /// Format as display string
    pub fn display(&self) -> String {
        let mut matcher = Vec::new();
        if let Some(ref c) = self.class {
            matcher.push(format!("class={}", c));
        }
        if let Some(ref t) = self.title {
            matcher.push(format!("title={}", t));
        }
        let match_str = if matcher.is_empty() {
            "*".to_string()
        } else {
            matcher.join(", ")
        };

        let mut actions = Vec::new();
        if let Some(ws) = self.workspace {
            actions.push(format!("ws={}", ws));
        }
        if let Some(true) = self.floating {
            actions.push("floating".to_string());
        }
        let action_str = if actions.is_empty() {
            "none".to_string()
        } else {
            actions.join(", ")
        };

        format!("{} → {}", match_str, action_str)
    }
}
