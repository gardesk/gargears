//! IPC communication with gardesk daemons

pub mod adapter;
pub mod adapters;
pub mod client;
pub mod discovery;
pub mod manager;

pub use adapter::IpcAdapter;
pub use manager::{ConnectionManager, IpcEvent};
