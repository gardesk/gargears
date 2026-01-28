//! Connection manager for coordinating IPC with all gardesk daemons

use crate::ipc::adapters::{
    GarbgAdapter, GarbgEvent, GarAdapter, GarbarAdapter, GarclipAdapter, GarfieldAdapter,
    GarlaunchAdapter, GarlockAdapter, GarnotifyAdapter, GarshotAdapter, GartermAdapter,
    GartrayAdapter,
};
use crate::ipc::discovery::{discover_daemons, DaemonStatus};
use crate::panels::Component;
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

/// Events from gardesk daemons
#[derive(Debug, Clone)]
pub enum IpcEvent {
    /// A daemon connected
    Connected(Component),
    /// A daemon disconnected
    Disconnected(Component),
    /// Status update from a daemon
    StatusUpdate {
        component: Component,
        data: Value,
    },
    /// Workspace changed (from gar)
    WorkspaceChanged {
        old: Option<usize>,
        new: usize,
    },
    /// Wallpaper changed (from garbg)
    WallpaperChanged {
        monitor: String,
        source: String,
    },
    /// Error from a daemon
    Error {
        component: Component,
        message: String,
    },
}

/// Manages connections to all gardesk daemons
pub struct ConnectionManager {
    // Individual adapters
    pub gar: GarAdapter,
    pub garbar: GarbarAdapter,
    pub garbg: GarbgAdapter,
    pub garterm: GartermAdapter,
    pub gartray: GartrayAdapter,
    pub garshot: GarshotAdapter,
    pub garlock: GarlockAdapter,
    pub garfield: GarfieldAdapter,
    pub garclip: GarclipAdapter,
    pub garlaunch: GarlaunchAdapter,
    pub garnotify: GarnotifyAdapter,

    // Connection state
    connection_state: HashMap<Component, bool>,

    // Pending events
    pending_events: Vec<IpcEvent>,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new() -> Self {
        Self {
            gar: GarAdapter::new(),
            garbar: GarbarAdapter::new(),
            garbg: GarbgAdapter::new(),
            garterm: GartermAdapter::new(),
            gartray: GartrayAdapter::new(),
            garshot: GarshotAdapter::new(),
            garlock: GarlockAdapter::new(),
            garfield: GarfieldAdapter::new(),
            garclip: GarclipAdapter::new(),
            garlaunch: GarlaunchAdapter::new(),
            garnotify: GarnotifyAdapter::new(),
            connection_state: HashMap::new(),
            pending_events: Vec::new(),
        }
    }

    /// Discover which daemons are running and attempt to connect
    pub fn connect_all(&mut self) -> Vec<DaemonStatus> {
        let statuses = discover_daemons();

        for status in &statuses {
            if status.running {
                let _ = self.connect(status.component);
            }
        }

        statuses
    }

    /// Connect to a specific daemon
    pub fn connect(&mut self, component: Component) -> Result<()> {
        let was_connected = self.is_connected(component);

        let result = match component {
            Component::Gar => self.gar.connect(),
            Component::Garbar => self.garbar.connect(),
            Component::Garbg => self.garbg.connect(),
            Component::Garterm => self.garterm.connect(),
            Component::Gartray => self.gartray.connect(),
            Component::Garshot => self.garshot.connect(),
            Component::Garlock => self.garlock.connect(),
            Component::Garfield => self.garfield.connect(),
            Component::Garclip => self.garclip.connect(),
            Component::Garlaunch => self.garlaunch.connect(),
            Component::Garnotify => self.garnotify.connect(),
        };

        match &result {
            Ok(()) => {
                self.connection_state.insert(component, true);
                if !was_connected {
                    self.pending_events.push(IpcEvent::Connected(component));
                }
            }
            Err(_) => {
                self.connection_state.insert(component, false);
                if was_connected {
                    self.pending_events.push(IpcEvent::Disconnected(component));
                }
            }
        }

        result
    }

    /// Disconnect from a specific daemon
    pub fn disconnect(&mut self, component: Component) {
        let was_connected = self.is_connected(component);

        match component {
            Component::Gar => self.gar.disconnect(),
            Component::Garbar => self.garbar.disconnect(),
            Component::Garbg => self.garbg.disconnect(),
            Component::Garterm => self.garterm.disconnect(),
            Component::Gartray => self.gartray.disconnect(),
            Component::Garshot => self.garshot.disconnect(),
            Component::Garlock => self.garlock.disconnect(),
            Component::Garfield => self.garfield.disconnect(),
            Component::Garclip => self.garclip.disconnect(),
            Component::Garlaunch => self.garlaunch.disconnect(),
            Component::Garnotify => self.garnotify.disconnect(),
        }

        self.connection_state.insert(component, false);
        if was_connected {
            self.pending_events.push(IpcEvent::Disconnected(component));
        }
    }

    /// Check if connected to a specific daemon
    pub fn is_connected(&self, component: Component) -> bool {
        match component {
            Component::Gar => self.gar.is_connected(),
            Component::Garbar => self.garbar.is_connected(),
            Component::Garbg => self.garbg.is_connected(),
            Component::Garterm => self.garterm.is_connected(),
            Component::Gartray => self.gartray.is_connected(),
            Component::Garshot => self.garshot.is_connected(),
            Component::Garlock => self.garlock.is_connected(),
            Component::Garfield => self.garfield.is_connected(),
            Component::Garclip => self.garclip.is_connected(),
            Component::Garlaunch => self.garlaunch.is_connected(),
            Component::Garnotify => self.garnotify.is_connected(),
        }
    }

    /// Get connection statuses for all components
    pub fn get_statuses(&self) -> Vec<DaemonStatus> {
        Component::all()
            .into_iter()
            .map(|component| DaemonStatus {
                component,
                running: self.is_connected(component),
                socket_path: crate::ipc::discovery::socket_path_for(component),
            })
            .collect()
    }

    /// Poll for events from all connected daemons
    pub fn poll_events(&mut self) -> Vec<IpcEvent> {
        let mut events = std::mem::take(&mut self.pending_events);

        // Poll gar events
        if self.gar.is_connected() {
            for event in self.gar.poll_events() {
                match event.event_type.as_str() {
                    "workspace" => {
                        if let Some(new) = event.data.get("current").and_then(|v| v.as_u64()) {
                            let old = event.data.get("previous").and_then(|v| v.as_u64());
                            events.push(IpcEvent::WorkspaceChanged {
                                old: old.map(|v| v as usize),
                                new: new as usize,
                            });
                        }
                    }
                    _ => {
                        events.push(IpcEvent::StatusUpdate {
                            component: Component::Gar,
                            data: event.data,
                        });
                    }
                }
            }
        }

        // Poll garbg events
        if self.garbg.is_connected() {
            for event in self.garbg.poll_events() {
                match event {
                    GarbgEvent::WallpaperChanged { monitor, source, .. } => {
                        events.push(IpcEvent::WallpaperChanged { monitor, source });
                    }
                    GarbgEvent::Error { message, .. } => {
                        events.push(IpcEvent::Error {
                            component: Component::Garbg,
                            message,
                        });
                    }
                    _ => {}
                }
            }
        }

        events
    }

    /// Try to reconnect to daemons that have become available
    pub fn refresh_connections(&mut self) {
        let statuses = discover_daemons();

        for status in statuses {
            let currently_connected = self.is_connected(status.component);

            if status.running && !currently_connected {
                // Daemon is now available, try to connect
                let _ = self.connect(status.component);
            } else if !status.running && currently_connected {
                // Daemon is no longer available
                self.disconnect(status.component);
            }
        }
    }

    /// Attempt reconnection with exponential backoff for disconnected daemons
    pub fn reconnect_with_backoff(&mut self, component: Component) -> Result<()> {
        match component {
            Component::Gar => self.gar.reconnect(),
            Component::Garbar => self.garbar.reconnect(),
            Component::Garbg => self.garbg.reconnect(),
            Component::Gartray => self.gartray.reconnect(),
            Component::Garshot => self.garshot.reconnect(),
            Component::Garlock => self.garlock.reconnect(),
            Component::Garfield => self.garfield.reconnect(),
            Component::Garclip => self.garclip.reconnect(),
            Component::Garlaunch => self.garlaunch.reconnect(),
            Component::Garnotify => self.garnotify.reconnect(),
            // garterm uses different socket scheme, just try fresh connect
            Component::Garterm => self.garterm.connect(),
        }
    }

    /// Check if reconnection should be attempted for a component
    pub fn should_reconnect(&self, component: Component) -> bool {
        match component {
            Component::Gar => self.gar.should_reconnect(),
            Component::Garbar => self.garbar.should_reconnect(),
            Component::Garbg => self.garbg.should_reconnect(),
            Component::Gartray => self.gartray.should_reconnect(),
            Component::Garshot => self.garshot.should_reconnect(),
            Component::Garlock => self.garlock.should_reconnect(),
            Component::Garfield => self.garfield.should_reconnect(),
            Component::Garclip => self.garclip.should_reconnect(),
            Component::Garlaunch => self.garlaunch.should_reconnect(),
            Component::Garnotify => self.garnotify.should_reconnect(),
            Component::Garterm => true, // Always allow garterm reconnect
        }
    }

    /// Reset backoff state for a component (e.g., after manual refresh)
    pub fn reset_backoff(&mut self, component: Component) {
        match component {
            Component::Gar => self.gar.reset_backoff(),
            Component::Garbar => self.garbar.reset_backoff(),
            Component::Garbg => self.garbg.reset_backoff(),
            Component::Gartray => self.gartray.reset_backoff(),
            Component::Garshot => self.garshot.reset_backoff(),
            Component::Garlock => self.garlock.reset_backoff(),
            Component::Garfield => self.garfield.reset_backoff(),
            Component::Garclip => self.garclip.reset_backoff(),
            Component::Garlaunch => self.garlaunch.reset_backoff(),
            Component::Garnotify => self.garnotify.reset_backoff(),
            Component::Garterm => {} // garterm doesn't track backoff
        }
    }

    /// Reset backoff for all components
    pub fn reset_all_backoff(&mut self) {
        for component in Component::all() {
            self.reset_backoff(component);
        }
    }

    /// Subscribe to events from daemons that support it
    pub fn subscribe_to_events(&mut self) {
        // Subscribe to gar workspace events
        if self.gar.is_connected() {
            let _ = self.gar.subscribe(&["workspace", "monitor", "window"]);
        }

        // Subscribe to garbg events
        if self.garbg.is_connected() {
            let _ = self.garbg.subscribe(&["wallpaper_changed", "slideshow_advanced"]);
        }
    }

    /// Take the gar adapter, replacing it with a disconnected one
    pub fn take_gar_adapter(&mut self) -> GarAdapter {
        std::mem::take(&mut self.gar)
    }

    /// Take the garbar adapter, replacing it with a disconnected one
    pub fn take_garbar_adapter(&mut self) -> GarbarAdapter {
        std::mem::take(&mut self.garbar)
    }

    /// Take the garbg adapter, replacing it with a disconnected one
    pub fn take_garbg_adapter(&mut self) -> GarbgAdapter {
        std::mem::take(&mut self.garbg)
    }

    /// Take the garterm adapter, replacing it with a disconnected one
    pub fn take_garterm_adapter(&mut self) -> GartermAdapter {
        std::mem::take(&mut self.garterm)
    }

    /// Take the gartray adapter, replacing it with a disconnected one
    pub fn take_gartray_adapter(&mut self) -> GartrayAdapter {
        std::mem::take(&mut self.gartray)
    }

    /// Take the garshot adapter, replacing it with a disconnected one
    pub fn take_garshot_adapter(&mut self) -> GarshotAdapter {
        std::mem::take(&mut self.garshot)
    }

    /// Take the garlock adapter, replacing it with a disconnected one
    pub fn take_garlock_adapter(&mut self) -> GarlockAdapter {
        std::mem::take(&mut self.garlock)
    }

    /// Take the garfield adapter, replacing it with a disconnected one
    pub fn take_garfield_adapter(&mut self) -> GarfieldAdapter {
        std::mem::take(&mut self.garfield)
    }

    /// Take the garclip adapter, replacing it with a disconnected one
    pub fn take_garclip_adapter(&mut self) -> GarclipAdapter {
        std::mem::take(&mut self.garclip)
    }

    /// Take the garlaunch adapter, replacing it with a disconnected one
    pub fn take_garlaunch_adapter(&mut self) -> GarlaunchAdapter {
        std::mem::take(&mut self.garlaunch)
    }

    /// Take the garnotify adapter, replacing it with a disconnected one
    pub fn take_garnotify_adapter(&mut self) -> GarnotifyAdapter {
        std::mem::take(&mut self.garnotify)
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}
