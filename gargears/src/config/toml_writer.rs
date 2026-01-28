//! TOML configuration writer for gardesk components
//!
//! Handles writing configuration for: garbg, garlock, garshot, garclip

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Get XDG config directory
fn config_dir() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            PathBuf::from(home).join(".config")
        })
}

/// garbg configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GarbgConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<Vec<GarbgSource>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slideshow: Option<GarbgSlideshow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animation: Option<GarbgAnimation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarbgSource {
    #[serde(rename = "type")]
    pub source_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recursive: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarbgSlideshow {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval_secs: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shuffle: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarbgAnimation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

impl GarbgConfig {
    pub fn config_path() -> PathBuf {
        config_dir().join("garbg").join("config.toml")
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read {}", path.display()))?;
            toml::from_str(&content)
                .with_context(|| format!("Failed to parse {}", path.display()))
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create {}", parent.display()))?;
        }
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize garbg config")?;
        fs::write(&path, content)
            .with_context(|| format!("Failed to write {}", path.display()))
    }
}

/// garlock configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GarlockConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idle_timeout_secs: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blur: Option<GarlockBlur>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clock: Option<GarlockClock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarlockBlur {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radius: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarlockClock {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<u32>,
}

impl GarlockConfig {
    pub fn config_path() -> PathBuf {
        config_dir().join("garlock").join("config.toml")
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read {}", path.display()))?;
            toml::from_str(&content)
                .with_context(|| format!("Failed to parse {}", path.display()))
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create {}", parent.display()))?;
        }
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize garlock config")?;
        fs::write(&path, content)
            .with_context(|| format!("Failed to write {}", path.display()))
    }
}

/// garshot configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GarshotConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copy_to_clipboard: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_notification: Option<bool>,
}

impl GarshotConfig {
    pub fn config_path() -> PathBuf {
        config_dir().join("garshot").join("config.toml")
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read {}", path.display()))?;
            toml::from_str(&content)
                .with_context(|| format!("Failed to parse {}", path.display()))
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create {}", parent.display()))?;
        }
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize garshot config")?;
        fs::write(&path, content)
            .with_context(|| format!("Failed to write {}", path.display()))
    }
}

/// garclip configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GarclipConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_history: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persist_history: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_primary: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deduplicate: Option<bool>,
}

impl GarclipConfig {
    pub fn config_path() -> PathBuf {
        config_dir().join("garclip").join("config.toml")
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read {}", path.display()))?;
            toml::from_str(&content)
                .with_context(|| format!("Failed to parse {}", path.display()))
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create {}", parent.display()))?;
        }
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize garclip config")?;
        fs::write(&path, content)
            .with_context(|| format!("Failed to write {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_garbg_serialize() {
        let config = GarbgConfig {
            fit: Some("fill".to_string()),
            color: Some("#282a36".to_string()),
            slideshow: Some(GarbgSlideshow {
                enabled: true,
                interval_secs: Some(300),
                shuffle: Some(true),
            }),
            ..Default::default()
        };
        let toml = toml::to_string_pretty(&config).unwrap();
        assert!(toml.contains("fit = \"fill\""));
        assert!(toml.contains("[slideshow]"));
    }

    #[test]
    fn test_garlock_serialize() {
        let config = GarlockConfig {
            idle_timeout_secs: Some(300),
            blur: Some(GarlockBlur {
                enabled: true,
                radius: Some(10),
            }),
            ..Default::default()
        };
        let toml = toml::to_string_pretty(&config).unwrap();
        assert!(toml.contains("idle_timeout_secs = 300"));
        assert!(toml.contains("[blur]"));
    }
}
