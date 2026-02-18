//! Configuration panels for each gardesk component

mod gar;
mod garbar;
mod garbg;
mod garclip;
mod garfield;
mod garlaunch;
mod garlock;
mod garnotify;
mod garshot;
mod garterm;
mod gartray;
mod placeholder;

pub use gar::GarPanel;
pub use garbar::GarbarPanel;
pub use garbg::GarbgPanel;
pub use garclip::GarclipPanel;
pub use garfield::GarfieldPanel;
pub use garlaunch::GarlaunchPanel;
pub use garlock::GarlockPanel;
pub use garnotify::GarnotifyPanel;
pub use garshot::GarshotPanel;
pub use garterm::GartermPanel;
pub use gartray::GartrayPanel;
pub use placeholder::PlaceholderPanel;

use serde::{Deserialize, Serialize};

/// All gardesk components that can be configured
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Component {
    Gar,
    Garbar,
    Garbg,
    Garterm,
    Gartray,
    Garshot,
    Garlock,
    Garfield,
    Garclip,
    Garlaunch,
    Garnotify,
    Garcard,
}

impl Component {
    /// Get all components in display order
    pub fn all() -> Vec<Component> {
        vec![
            Component::Gar,
            Component::Garbar,
            Component::Garbg,
            Component::Garterm,
            Component::Gartray,
            Component::Garshot,
            Component::Garlock,
            Component::Garfield,
            Component::Garclip,
            Component::Garlaunch,
            Component::Garnotify,
            Component::Garcard,
        ]
    }

    /// Get display name for a component
    pub fn display_name(&self) -> &'static str {
        match self {
            Component::Gar => "gar",
            Component::Garbar => "garbar",
            Component::Garbg => "garbg",
            Component::Garterm => "garterm",
            Component::Gartray => "gartray",
            Component::Garshot => "garshot",
            Component::Garlock => "garlock",
            Component::Garfield => "garfield",
            Component::Garclip => "garclip",
            Component::Garlaunch => "garlaunch",
            Component::Garnotify => "garnotify",
            Component::Garcard => "garcard",
        }
    }

    /// Get description for a component
    pub fn description(&self) -> &'static str {
        match self {
            Component::Gar => "Tiling window manager",
            Component::Garbar => "Status bar",
            Component::Garbg => "Wallpaper daemon",
            Component::Garterm => "Terminal emulator",
            Component::Gartray => "System tray & quick settings",
            Component::Garshot => "Screenshot tool",
            Component::Garlock => "Screen locker",
            Component::Garfield => "File explorer",
            Component::Garclip => "Clipboard manager",
            Component::Garlaunch => "Application launcher",
            Component::Garnotify => "Notification daemon",
            Component::Garcard => "Authentication agent",
        }
    }

    /// Parse a component from a name
    pub fn from_name(name: &str) -> Option<Component> {
        match name.to_lowercase().as_str() {
            "gar" => Some(Component::Gar),
            "garbar" => Some(Component::Garbar),
            "garbg" => Some(Component::Garbg),
            "garterm" => Some(Component::Garterm),
            "gartray" => Some(Component::Gartray),
            "garshot" => Some(Component::Garshot),
            "garlock" => Some(Component::Garlock),
            "garfield" => Some(Component::Garfield),
            "garclip" => Some(Component::Garclip),
            "garlaunch" => Some(Component::Garlaunch),
            "garnotify" => Some(Component::Garnotify),
            "garcard" => Some(Component::Garcard),
            _ => None,
        }
    }
}
