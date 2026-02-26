//! IPC adapter for garcard (authentication agent)

use crate::ipc::client::{IpcClient, StandardResponse};
use crate::ipc::discovery::socket_path_for;
use crate::panels::Component;
use anyhow::Result;
use serde::Serialize;
use serde_json::Value;
use std::path::PathBuf;

/// Commands for garcard daemon.
#[derive(Debug, Serialize)]
#[serde(tag = "command", rename_all = "snake_case")]
enum GarcardCommand {
    Ping,
    Status,
    Diagnose,
    Version,
    AuthSummary,
    TempList,
    TempRevoke { authorization_id: String },
    TempRevokeAll,
    Quit,
}

/// Adapter for garcard authentication agent
pub struct GarcardAdapter {
    client: IpcClient,
}

impl GarcardAdapter {
    pub fn new() -> Self {
        use std::time::Duration;
        Self {
            client: IpcClient::new().with_timeout(Duration::from_secs(5)),
        }
    }

    pub fn socket_path(&self) -> PathBuf {
        socket_path_for(Component::Garcard)
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

    fn send_command(&mut self, command: GarcardCommand) -> Result<Option<Value>> {
        let response: StandardResponse = self.client.send_receive(&command)?;
        response.into_result()
    }

    pub fn ping(&mut self) -> Result<Option<Value>> {
        self.send_command(GarcardCommand::Ping)
    }

    pub fn status(&mut self) -> Result<Option<Value>> {
        self.send_command(GarcardCommand::Status)
    }

    pub fn diagnose(&mut self) -> Result<Option<Value>> {
        self.send_command(GarcardCommand::Diagnose)
    }

    pub fn version(&mut self) -> Result<Option<Value>> {
        self.send_command(GarcardCommand::Version)
    }

    pub fn auth_summary(&mut self) -> Result<Option<Value>> {
        self.send_command(GarcardCommand::AuthSummary)
    }

    pub fn temp_list(&mut self) -> Result<Option<Value>> {
        self.send_command(GarcardCommand::TempList)
    }

    pub fn temp_revoke(&mut self, authorization_id: impl Into<String>) -> Result<Option<Value>> {
        self.send_command(GarcardCommand::TempRevoke {
            authorization_id: authorization_id.into(),
        })
    }

    pub fn temp_revoke_all(&mut self) -> Result<Option<Value>> {
        self.send_command(GarcardCommand::TempRevokeAll)
    }

    pub fn quit(&mut self) -> Result<Option<Value>> {
        self.send_command(GarcardCommand::Quit)
    }
}

impl Default for GarcardAdapter {
    fn default() -> Self {
        Self::new()
    }
}
