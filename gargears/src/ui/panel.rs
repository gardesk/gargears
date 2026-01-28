//! Panel trait for configuration panels

use anyhow::Result;
use gartk_core::{InputEvent, Rect, Theme};
use gartk_render::Renderer;

/// Actions that a panel can request
#[derive(Debug, Clone)]
pub enum PanelAction {
    /// No action
    None,
    /// Apply changes via IPC
    Apply,
    /// Reset to original values
    Reset,
    /// Request a redraw
    Redraw,
    /// Save changes to config file
    Save,
    /// Instant apply - apply the most recently changed field immediately
    InstantApply,
}

/// Trait for configuration panels
pub trait Panel {
    /// Get the panel name (for display)
    fn name(&self) -> &str;

    /// Get the component description
    fn description(&self) -> &str;

    /// Check if the panel has unsaved changes
    fn is_dirty(&self) -> bool;

    /// Render the panel
    fn render(&mut self, renderer: &mut Renderer, bounds: Rect, theme: &Theme) -> Result<()>;

    /// Handle an input event
    fn handle_event(&mut self, event: &InputEvent) -> PanelAction;

    /// Handle mouse move (for hover states). Returns true if redraw needed.
    fn on_mouse_move(&mut self, x: i32, y: i32) -> bool;

    /// Clear focus from any focused widget in this panel
    fn blur_focused(&mut self) {}

    /// Reset to original/saved values
    fn reset(&mut self);

    /// Apply changes (send IPC commands)
    fn apply(&mut self) -> Result<()>;

    /// Update with fresh status from daemon
    fn update_status(&mut self, status: serde_json::Value);

    /// Check if this panel has a config file that can be saved
    fn has_config_file(&self) -> bool {
        false
    }

    /// Save current settings to config file
    fn save_to_config(&mut self) -> Result<()> {
        Ok(())
    }

    /// Set instant-apply mode
    fn set_instant_apply(&mut self, _enabled: bool) {
        // Default: do nothing (panel doesn't support instant apply)
    }

    /// Check if instant-apply is enabled
    fn instant_apply_enabled(&self) -> bool {
        false
    }

    /// Apply the most recently changed field (for instant-apply mode)
    fn apply_instant(&mut self) -> Result<()> {
        // Default: fall back to full apply
        self.apply()
    }
}
