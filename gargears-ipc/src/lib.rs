//! IPC protocol types for gargears
//!
//! This crate defines the IPC protocol used by gargears daemon and gargearsctl.

use serde::{Deserialize, Serialize};

/// Commands that can be sent to the gargears daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum Command {
    /// Show the gargears window
    Show,
    /// Hide the gargears window
    Hide,
    /// Toggle window visibility
    Toggle,
    /// Get daemon status
    Status,
    /// Quit the daemon
    Quit,
}

/// Response from the gargears daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Response {
    pub fn ok() -> Self {
        Self {
            success: true,
            data: None,
            error: None,
        }
    }

    pub fn ok_with_data(data: serde_json::Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
        }
    }
}

/// Status information returned by the daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub visible: bool,
    pub running: bool,
}

/// Get the default socket path for gargears
pub fn socket_path() -> std::path::PathBuf {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        std::path::PathBuf::from(runtime_dir).join("gargears.sock")
    } else {
        std::path::PathBuf::from("/tmp/gargears.sock")
    }
}
