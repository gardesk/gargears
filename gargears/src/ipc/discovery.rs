//! Daemon discovery - check which gardesk daemons are running

use crate::panels::Component;
use std::path::PathBuf;

/// Status of a daemon
#[derive(Debug, Clone)]
pub struct DaemonStatus {
    pub component: Component,
    pub running: bool,
    pub socket_path: PathBuf,
}

/// Get the socket path for a component
pub fn socket_path_for(component: Component) -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| "/tmp".to_string());

    let socket_name = match component {
        Component::Gar => "gar.sock",
        Component::Garbar => "garbar.sock",
        Component::Garbg => "garbg.sock",
        Component::Garterm => "garterm/focused", // Special: points to focused instance
        Component::Gartray => "gartray.sock",
        Component::Garshot => "garshot.sock",
        Component::Garlock => "garlock.sock",
        Component::Garfield => "garfield.sock",
        Component::Garclip => "garclip.sock",
        Component::Garlaunch => "garlaunch.sock",
        Component::Garnotify => "garnotify.sock",
    };

    PathBuf::from(runtime_dir).join(socket_name)
}

/// Discover which daemons are running
pub fn discover_daemons() -> Vec<DaemonStatus> {
    Component::all()
        .into_iter()
        .map(|component| {
            let socket_path = socket_path_for(component);
            let running = socket_path.exists();

            DaemonStatus {
                component,
                running,
                socket_path,
            }
        })
        .collect()
}

/// Check if a specific daemon is running
pub fn is_daemon_running(component: Component) -> bool {
    socket_path_for(component).exists()
}
