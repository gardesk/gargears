//! Base IPC client functionality

use anyhow::{Context, Result};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

/// Default read/write timeout
const DEFAULT_TIMEOUT_SECS: u64 = 5;

/// Maximum reconnection attempts
const MAX_RECONNECT_ATTEMPTS: u32 = 5;

/// Initial backoff delay
const INITIAL_BACKOFF_MS: u64 = 100;

/// Maximum backoff delay
const MAX_BACKOFF_MS: u64 = 5000;

/// Base IPC client for Unix socket communication
pub struct IpcClient {
    stream: Option<UnixStream>,
    reader: Option<BufReader<UnixStream>>,
    socket_path: Option<PathBuf>,
    timeout: Duration,
    last_connect_attempt: Option<Instant>,
    backoff_ms: u64,
    connect_attempts: u32,
}

impl IpcClient {
    /// Create a new disconnected client
    pub fn new() -> Self {
        Self {
            stream: None,
            reader: None,
            socket_path: None,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            last_connect_attempt: None,
            backoff_ms: INITIAL_BACKOFF_MS,
            connect_attempts: 0,
        }
    }

    /// Set read/write timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Connect to a Unix socket
    pub fn connect(&mut self, path: &Path) -> Result<()> {
        self.socket_path = Some(path.to_path_buf());
        self.connect_internal(path)
    }

    /// Internal connect implementation
    fn connect_internal(&mut self, path: &Path) -> Result<()> {
        let stream = UnixStream::connect(path)
            .with_context(|| format!("Failed to connect to {:?}", path))?;

        // Set timeouts
        stream.set_read_timeout(Some(self.timeout))?;
        stream.set_write_timeout(Some(self.timeout))?;

        // Clone for reader
        let reader_stream = stream.try_clone()?;

        self.stream = Some(stream);
        self.reader = Some(BufReader::new(reader_stream));

        // Reset backoff on successful connection
        self.backoff_ms = INITIAL_BACKOFF_MS;
        self.connect_attempts = 0;
        self.last_connect_attempt = Some(Instant::now());

        Ok(())
    }

    /// Attempt to reconnect with exponential backoff
    pub fn reconnect(&mut self) -> Result<()> {
        let path = self.socket_path.clone().context("No socket path stored")?;

        // Check if we should wait before retrying
        if let Some(last_attempt) = self.last_connect_attempt {
            let elapsed = last_attempt.elapsed().as_millis() as u64;
            if elapsed < self.backoff_ms {
                let wait_time = self.backoff_ms - elapsed;
                thread::sleep(Duration::from_millis(wait_time));
            }
        }

        self.connect_attempts += 1;
        self.last_connect_attempt = Some(Instant::now());

        let result = self.connect_internal(&path);

        if result.is_err() {
            // Exponential backoff with cap
            self.backoff_ms = (self.backoff_ms * 2).min(MAX_BACKOFF_MS);
        }

        result
    }

    /// Attempt to reconnect with retry limit
    pub fn reconnect_with_retry(&mut self) -> Result<()> {
        for _ in 0..MAX_RECONNECT_ATTEMPTS {
            match self.reconnect() {
                Ok(()) => return Ok(()),
                Err(_) if self.connect_attempts < MAX_RECONNECT_ATTEMPTS => continue,
                Err(e) => return Err(e),
            }
        }
        Err(anyhow::anyhow!("Max reconnection attempts exceeded"))
    }

    /// Check if we should attempt reconnection (backoff elapsed)
    pub fn should_attempt_reconnect(&self) -> bool {
        if self.connect_attempts >= MAX_RECONNECT_ATTEMPTS {
            return false;
        }
        match self.last_connect_attempt {
            Some(last) => last.elapsed().as_millis() as u64 >= self.backoff_ms,
            None => true,
        }
    }

    /// Reset reconnection state (call when manual refresh requested)
    pub fn reset_backoff(&mut self) {
        self.backoff_ms = INITIAL_BACKOFF_MS;
        self.connect_attempts = 0;
        self.last_connect_attempt = None;
    }

    /// Disconnect from the socket
    pub fn disconnect(&mut self) {
        self.stream = None;
        self.reader = None;
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    /// Send a JSON message and receive a response
    pub fn send_receive<T: serde::Serialize, R: serde::de::DeserializeOwned>(
        &mut self,
        message: &T,
    ) -> Result<R> {
        let stream = self
            .stream
            .as_mut()
            .context("Not connected")?;

        // Serialize and send
        let json = serde_json::to_string(message)?;
        writeln!(stream, "{}", json)?;
        stream.flush()?;

        // Read response
        let reader = self.reader.as_mut().context("No reader")?;
        let mut line = String::new();
        reader.read_line(&mut line)?;

        // Parse response
        let response: R = serde_json::from_str(&line)
            .with_context(|| format!("Failed to parse response: {}", line.trim()))?;

        Ok(response)
    }

    /// Send a message without expecting a response
    pub fn send<T: serde::Serialize>(&mut self, message: &T) -> Result<()> {
        let stream = self.stream.as_mut().context("Not connected")?;

        let json = serde_json::to_string(message)?;
        writeln!(stream, "{}", json)?;
        stream.flush()?;

        Ok(())
    }

    /// Try to read a line (non-blocking style with short timeout)
    pub fn try_read_line(&mut self) -> Option<String> {
        let reader = self.reader.as_mut()?;

        // Set a very short timeout for polling
        if let Some(stream) = &self.stream {
            let _ = stream.set_read_timeout(Some(Duration::from_millis(10)));
        }

        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => None, // EOF
            Ok(_) => Some(line),
            Err(_) => None, // Timeout or error
        }
    }
}

impl Default for IpcClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Standard response format used by most gardesk daemons
#[derive(Debug, Clone, serde::Deserialize)]
pub struct StandardResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

impl StandardResponse {
    /// Get data or return error
    pub fn into_result(self) -> Result<Option<serde_json::Value>> {
        if self.success {
            Ok(self.data)
        } else {
            Err(anyhow::anyhow!(
                self.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }
}
