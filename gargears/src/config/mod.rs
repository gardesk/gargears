//! Configuration file reading and writing
//!
//! This module handles reading and writing configuration files for gardesk components.
//! - Lua config: gar, garbar, garterm (~/.config/gar/init.lua)
//! - TOML config: garbg, garlock, garshot, garclip

mod lua_writer;
mod toml_writer;

pub use lua_writer::{
    parse_gar_sets, GarSettings, GarbarSettings, GartermSettings, LuaConfigWriter, LuaValue,
};
pub use toml_writer::{
    GarbgConfig, GarbgSlideshow, GarbgSource, GarclipConfig, GarlockConfig, GarshotConfig,
};

use anyhow::Result;
use std::process::Command;

/// Trigger config reload for a component
pub fn reload_component(component: &str) -> Result<()> {
    match component {
        "gar" => {
            // Send SIGHUP to gar or use IPC reload command
            let _ = Command::new("garctl").arg("reload").spawn();
        }
        "garbar" => {
            // garbar reloads on config change via inotify, or SIGHUP
            let _ = Command::new("pkill")
                .args(["-HUP", "garbar"])
                .spawn();
        }
        "garbg" => {
            let _ = Command::new("garbgctl").arg("reload").spawn();
        }
        "garlock" => {
            // garlock typically reloads on next lock
        }
        "garshot" => {
            // garshot reads config on each invocation
        }
        "garclip" => {
            let _ = Command::new("pkill")
                .args(["-HUP", "garclip"])
                .spawn();
        }
        "garterm" => {
            // garterm reads config on new window
        }
        _ => {}
    }
    Ok(())
}

/// Check if a config file exists
pub fn config_exists(component: &str) -> bool {
    match component {
        "gar" | "garbar" | "garterm" => lua_writer::lua_config_path().exists(),
        "garbg" => toml_writer::GarbgConfig::config_path().exists(),
        "garlock" => toml_writer::GarlockConfig::config_path().exists(),
        "garshot" => toml_writer::GarshotConfig::config_path().exists(),
        "garclip" => toml_writer::GarclipConfig::config_path().exists(),
        _ => false,
    }
}

/// Get the config path for a component
pub fn config_path(component: &str) -> Option<std::path::PathBuf> {
    match component {
        "gar" | "garbar" | "garterm" => Some(lua_writer::lua_config_path()),
        "garbg" => Some(toml_writer::GarbgConfig::config_path()),
        "garlock" => Some(toml_writer::GarlockConfig::config_path()),
        "garshot" => Some(toml_writer::GarshotConfig::config_path()),
        "garclip" => Some(toml_writer::GarclipConfig::config_path()),
        _ => None,
    }
}
