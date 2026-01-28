//! Daemon mode for gargears
//!
//! Provides IPC server functionality for controlling gargears from external tools.

use anyhow::{Context, Result};
use gargears_ipc::{Command, DaemonStatus, Response};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process::Command as ProcessCommand;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};

/// Commands sent from IPC server to main app
#[derive(Debug)]
pub enum DaemonCommand {
    Show,
    Hide,
    Toggle,
    Status,
    Quit,
}

/// IPC server for daemon mode
pub struct IpcServer {
    listener: UnixListener,
    socket_path: PathBuf,
    command_tx: Sender<DaemonCommand>,
}

impl IpcServer {
    /// Create a new IPC server
    pub fn new(command_tx: Sender<DaemonCommand>) -> Result<Self> {
        let socket_path = gargears_ipc::socket_path();

        // Remove existing socket if present
        if socket_path.exists() {
            std::fs::remove_file(&socket_path)
                .with_context(|| format!("Failed to remove existing socket: {:?}", socket_path))?;
        }

        let listener = UnixListener::bind(&socket_path)
            .with_context(|| format!("Failed to bind to socket: {:?}", socket_path))?;

        // Set non-blocking so we can check for shutdown
        listener.set_nonblocking(true)?;

        tracing::info!("IPC server listening on {:?}", socket_path);

        Ok(Self {
            listener,
            socket_path,
            command_tx,
        })
    }

    /// Run the server loop (should be called in a separate thread)
    pub fn run(&self, visible: std::sync::Arc<std::sync::atomic::AtomicBool>) {
        loop {
            match self.listener.accept() {
                Ok((stream, _)) => {
                    if let Err(e) = self.handle_client(stream, &visible) {
                        tracing::warn!("Error handling client: {}", e);
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No connection, sleep briefly
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                Err(e) => {
                    tracing::error!("Accept error: {}", e);
                    break;
                }
            }
        }
    }

    fn handle_client(
        &self,
        mut stream: UnixStream,
        visible: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<()> {
        stream.set_nonblocking(false)?;

        let mut reader = BufReader::new(stream.try_clone()?);
        let mut line = String::new();

        if reader.read_line(&mut line)? == 0 {
            return Ok(());
        }

        let command: Command = serde_json::from_str(line.trim())
            .with_context(|| format!("Failed to parse command: {}", line.trim()))?;

        let response = match command {
            Command::Show => {
                self.command_tx.send(DaemonCommand::Show)?;
                Response::ok()
            }
            Command::Hide => {
                self.command_tx.send(DaemonCommand::Hide)?;
                Response::ok()
            }
            Command::Toggle => {
                self.command_tx.send(DaemonCommand::Toggle)?;
                Response::ok()
            }
            Command::Status => {
                let status = DaemonStatus {
                    visible: visible.load(std::sync::atomic::Ordering::Relaxed),
                    running: true,
                };
                Response::ok_with_data(serde_json::to_value(status)?)
            }
            Command::Quit => {
                self.command_tx.send(DaemonCommand::Quit)?;
                Response::ok()
            }
        };

        let response_str = serde_json::to_string(&response)?;
        stream.write_all(response_str.as_bytes())?;
        stream.write_all(b"\n")?;
        stream.flush()?;

        Ok(())
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

/// Create command channel for daemon mode
pub fn create_command_channel() -> (Sender<DaemonCommand>, Receiver<DaemonCommand>) {
    mpsc::channel()
}

/// Check for pending daemon commands (non-blocking)
pub fn check_command(rx: &Receiver<DaemonCommand>) -> Option<DaemonCommand> {
    match rx.try_recv() {
        Ok(cmd) => Some(cmd),
        Err(TryRecvError::Empty) => None,
        Err(TryRecvError::Disconnected) => Some(DaemonCommand::Quit),
    }
}

/// Send a desktop notification
pub fn notify(summary: &str, body: &str) {
    let _ = ProcessCommand::new("notify-send")
        .arg("--app-name=gargears")
        .arg("--icon=preferences-system")
        .arg(summary)
        .arg(body)
        .spawn();
}

/// Send startup notification
pub fn notify_startup() {
    notify(
        "gargears daemon started",
        "Use 'gargearsctl show' or keybind to open settings",
    );
}
