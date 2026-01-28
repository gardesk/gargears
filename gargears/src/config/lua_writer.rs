//! Lua configuration writer for gardesk components
//!
//! Handles writing configuration for: gar, garbar, garterm
//! These share ~/.config/gar/init.lua

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Get the path to the shared Lua config
pub fn lua_config_path() -> PathBuf {
    let config_dir = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            PathBuf::from(home).join(".config")
        });
    config_dir.join("gar").join("init.lua")
}

/// Represents a Lua value that can be serialized
#[derive(Debug, Clone)]
pub enum LuaValue {
    Number(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Color(String),
}

impl LuaValue {
    pub fn to_lua(&self) -> String {
        match self {
            LuaValue::Number(n) => n.to_string(),
            LuaValue::Float(f) => format!("{:.2}", f),
            LuaValue::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
            LuaValue::Bool(b) => if *b { "true" } else { "false" }.to_string(),
            LuaValue::Color(c) => format!("\"{}\"", c),
        }
    }
}

/// gar window manager settings
#[derive(Debug, Clone, Default)]
pub struct GarSettings {
    pub border_width: Option<u32>,
    pub gap_inner: Option<u32>,
    pub gap_outer: Option<u32>,
    pub border_color_focused: Option<String>,
    pub border_color_unfocused: Option<String>,
    pub border_color_urgent: Option<String>,
    pub focus_follows_mouse: Option<bool>,
}

impl GarSettings {
    pub fn to_lua_statements(&self) -> Vec<(String, LuaValue)> {
        let mut statements = Vec::new();

        if let Some(v) = self.border_width {
            statements.push(("border_width".to_string(), LuaValue::Number(v as i64)));
        }
        if let Some(v) = self.gap_inner {
            statements.push(("gap_inner".to_string(), LuaValue::Number(v as i64)));
        }
        if let Some(v) = self.gap_outer {
            statements.push(("gap_outer".to_string(), LuaValue::Number(v as i64)));
        }
        if let Some(ref v) = self.border_color_focused {
            statements.push(("border_color_focused".to_string(), LuaValue::Color(v.clone())));
        }
        if let Some(ref v) = self.border_color_unfocused {
            statements.push(("border_color_unfocused".to_string(), LuaValue::Color(v.clone())));
        }
        if let Some(ref v) = self.border_color_urgent {
            statements.push(("border_color_urgent".to_string(), LuaValue::Color(v.clone())));
        }
        if let Some(v) = self.focus_follows_mouse {
            statements.push(("focus_follows_mouse".to_string(), LuaValue::Bool(v)));
        }

        statements
    }
}

/// garbar status bar settings
#[derive(Debug, Clone, Default)]
pub struct GarbarSettings {
    pub height: Option<u32>,
    pub position: Option<String>,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub background_color: Option<String>,
    pub foreground_color: Option<String>,
    pub modules_left: Option<Vec<String>>,
    pub modules_center: Option<Vec<String>>,
    pub modules_right: Option<Vec<String>>,
}

impl GarbarSettings {
    pub fn to_lua_table(&self) -> String {
        let mut lines = Vec::new();
        lines.push("gar.bar = {".to_string());

        if let Some(v) = self.height {
            lines.push(format!("    height = {},", v));
        }
        if let Some(ref v) = self.position {
            lines.push(format!("    position = \"{}\",", v));
        }
        if let Some(ref v) = self.font_family {
            lines.push(format!("    font_family = \"{}\",", v));
        }
        if let Some(v) = self.font_size {
            lines.push(format!("    font_size = {:.1},", v));
        }
        if let Some(ref v) = self.background_color {
            lines.push(format!("    background = \"{}\",", v));
        }
        if let Some(ref v) = self.foreground_color {
            lines.push(format!("    foreground = \"{}\",", v));
        }
        if let Some(ref v) = self.modules_left {
            let mods: Vec<String> = v.iter().map(|m| format!("\"{}\"", m)).collect();
            lines.push(format!("    modules_left = {{ {} }},", mods.join(", ")));
        }
        if let Some(ref v) = self.modules_center {
            let mods: Vec<String> = v.iter().map(|m| format!("\"{}\"", m)).collect();
            lines.push(format!("    modules_center = {{ {} }},", mods.join(", ")));
        }
        if let Some(ref v) = self.modules_right {
            let mods: Vec<String> = v.iter().map(|m| format!("\"{}\"", m)).collect();
            lines.push(format!("    modules_right = {{ {} }},", mods.join(", ")));
        }

        lines.push("}".to_string());
        lines.join("\n")
    }
}

/// garterm terminal settings
#[derive(Debug, Clone, Default)]
pub struct GartermSettings {
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub scrollback_lines: Option<u32>,
    pub cursor_style: Option<String>,
    pub cursor_blink: Option<bool>,
}

impl GartermSettings {
    pub fn to_lua_table(&self) -> String {
        let mut lines = Vec::new();
        lines.push("gar.terminal = {".to_string());

        if let Some(ref v) = self.font_family {
            lines.push(format!("    font_family = \"{}\",", v));
        }
        if let Some(v) = self.font_size {
            lines.push(format!("    font_size = {:.1},", v));
        }
        if let Some(v) = self.scrollback_lines {
            lines.push(format!("    scrollback = {},", v));
        }
        if let Some(ref v) = self.cursor_style {
            lines.push(format!("    cursor_style = \"{}\",", v));
        }
        if let Some(v) = self.cursor_blink {
            lines.push(format!("    cursor_blink = {},", v));
        }

        lines.push("}".to_string());
        lines.join("\n")
    }
}

/// Lua config writer that preserves existing structure
pub struct LuaConfigWriter {
    content: String,
    path: PathBuf,
}

impl LuaConfigWriter {
    /// Load existing config or create empty
    pub fn load() -> Result<Self> {
        let path = lua_config_path();
        let content = if path.exists() {
            fs::read_to_string(&path)
                .with_context(|| format!("Failed to read {}", path.display()))?
        } else {
            String::new()
        };
        Ok(Self { content, path })
    }

    /// Update a gar.set() call, or add it if not present
    pub fn set_value(&mut self, key: &str, value: LuaValue) {
        let pattern = format!(r#"gar\.set\(\s*["']{}["']\s*,\s*[^)]+\)"#, regex::escape(key));
        let replacement = format!("gar.set(\"{}\", {})", key, value.to_lua());

        if let Ok(re) = regex::Regex::new(&pattern) {
            if re.is_match(&self.content) {
                self.content = re.replace(&self.content, replacement.as_str()).to_string();
            } else {
                // Add new setting after last gar.set() or at start
                if let Some(pos) = self.find_last_gar_set() {
                    // Find end of line
                    let end = self.content[pos..].find('\n').map(|p| pos + p + 1).unwrap_or(self.content.len());
                    self.content.insert_str(end, &format!("{}\n", replacement));
                } else {
                    // No existing gar.set(), add at beginning
                    self.content = format!("{}\n{}", replacement, self.content);
                }
            }
        }
    }

    /// Update multiple values
    pub fn set_values(&mut self, settings: &[(String, LuaValue)]) {
        for (key, value) in settings {
            self.set_value(key, value.clone());
        }
    }

    /// Update gar.bar table
    pub fn set_bar_table(&mut self, settings: &GarbarSettings) {
        let table = settings.to_lua_table();

        // Try to find and replace existing gar.bar block
        if let Ok(re) = regex::Regex::new(r"gar\.bar\s*=\s*\{[^}]*\}") {
            if re.is_match(&self.content) {
                self.content = re.replace(&self.content, table.as_str()).to_string();
                return;
            }
        }

        // No existing block, add after gar.set() calls or at end
        self.content.push_str("\n\n");
        self.content.push_str(&table);
        self.content.push('\n');
    }

    /// Update gar.terminal table
    pub fn set_terminal_table(&mut self, settings: &GartermSettings) {
        let table = settings.to_lua_table();

        // Try to find and replace existing gar.terminal block
        if let Ok(re) = regex::Regex::new(r"gar\.terminal\s*=\s*\{[^}]*\}") {
            if re.is_match(&self.content) {
                self.content = re.replace(&self.content, table.as_str()).to_string();
                return;
            }
        }

        // No existing block, add after gar.bar or at end
        self.content.push_str("\n\n");
        self.content.push_str(&table);
        self.content.push('\n');
    }

    /// Save the config file
    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create {}", parent.display()))?;
        }

        // Create backup
        if self.path.exists() {
            let backup = self.path.with_extension("lua.bak");
            let _ = fs::copy(&self.path, &backup);
        }

        fs::write(&self.path, &self.content)
            .with_context(|| format!("Failed to write {}", self.path.display()))
    }

    /// Find position of last gar.set() call
    fn find_last_gar_set(&self) -> Option<usize> {
        let pattern = r"gar\.set\(";
        let mut last_pos = None;

        if let Ok(re) = regex::Regex::new(pattern) {
            for m in re.find_iter(&self.content) {
                last_pos = Some(m.start());
            }
        }

        last_pos
    }

    /// Get current content (for preview)
    pub fn content(&self) -> &str {
        &self.content
    }
}

/// Helper to extract current gar.set() values from config
pub fn parse_gar_sets(content: &str) -> HashMap<String, String> {
    let mut values = HashMap::new();

    // Match gar.set("key", value) patterns
    if let Ok(re) = regex::Regex::new(r#"gar\.set\(\s*["']([^"']+)["']\s*,\s*([^)]+)\)"#) {
        for cap in re.captures_iter(content) {
            if let (Some(key), Some(val)) = (cap.get(1), cap.get(2)) {
                values.insert(key.as_str().to_string(), val.as_str().trim().to_string());
            }
        }
    }

    values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_value_serialization() {
        assert_eq!(LuaValue::Number(42).to_lua(), "42");
        assert_eq!(LuaValue::Float(3.14).to_lua(), "3.14");
        assert_eq!(LuaValue::String("hello".into()).to_lua(), "\"hello\"");
        assert_eq!(LuaValue::Bool(true).to_lua(), "true");
        assert_eq!(LuaValue::Color("#ff0000".into()).to_lua(), "\"#ff0000\"");
    }

    #[test]
    fn test_gar_settings() {
        let settings = GarSettings {
            border_width: Some(2),
            gap_inner: Some(10),
            ..Default::default()
        };
        let stmts = settings.to_lua_statements();
        assert_eq!(stmts.len(), 2);
    }

    #[test]
    fn test_garbar_table() {
        let settings = GarbarSettings {
            height: Some(28),
            position: Some("top".into()),
            modules_left: Some(vec!["workspaces".into()]),
            ..Default::default()
        };
        let table = settings.to_lua_table();
        assert!(table.contains("height = 28"));
        assert!(table.contains("position = \"top\""));
        assert!(table.contains("modules_left = { \"workspaces\" }"));
    }

    #[test]
    fn test_parse_gar_sets() {
        let content = r#"
gar.set("border_width", 2)
gar.set("gap_inner", 10)
gar.set('focus_follows_mouse', true)
"#;
        let values = parse_gar_sets(content);
        assert_eq!(values.get("border_width"), Some(&"2".to_string()));
        assert_eq!(values.get("gap_inner"), Some(&"10".to_string()));
        assert_eq!(values.get("focus_follows_mouse"), Some(&"true".to_string()));
    }
}
