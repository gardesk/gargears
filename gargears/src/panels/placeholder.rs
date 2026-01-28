//! Placeholder panel for components without full implementations

use crate::ui::{Panel, PanelAction};
use anyhow::Result;
use gartk_core::{InputEvent, Rect, Theme};
use gartk_render::{Renderer, TextStyle};

/// Placeholder panel for components not yet implemented
pub struct PlaceholderPanel {
    name: String,
    description: String,
    connected: bool,
}

impl PlaceholderPanel {
    pub fn new(name: impl Into<String>, description: impl Into<String>, connected: bool) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            connected,
        }
    }
}

impl Panel for PlaceholderPanel {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn is_dirty(&self) -> bool {
        false
    }

    fn render(&mut self, renderer: &mut Renderer, bounds: Rect, theme: &Theme) -> Result<()> {
        let padding = theme.padding as i32;

        // Title
        let title_style = TextStyle::new()
            .font_family(&theme.font_family)
            .font_size(theme.font_size + 4.0)
            .color(theme.foreground);

        renderer.text(
            &self.name,
            (bounds.x + padding) as f64,
            (bounds.y + padding) as f64,
            &title_style,
        )?;

        // Description
        let desc_style = TextStyle::new()
            .font_family(&theme.font_family)
            .font_size(theme.font_size)
            .color(theme.item_description);

        renderer.text(
            &self.description,
            (bounds.x + padding) as f64,
            (bounds.y + padding + theme.font_size as i32 + 12) as f64,
            &desc_style,
        )?;

        // Connection status
        let status_color = if self.connected {
            gartk_core::Color::from_u8(0x50, 0xfa, 0x7b, 0xff) // green
        } else {
            gartk_core::Color::from_u8(0xff, 0x55, 0x55, 0xff) // red
        };

        let status_style = TextStyle::new()
            .font_family(&theme.font_family)
            .font_size(theme.font_size)
            .color(status_color);

        let status_text = if self.connected {
            "● Connected"
        } else {
            "○ Not running"
        };

        renderer.text(
            status_text,
            (bounds.x + padding) as f64,
            (bounds.y + padding + (theme.font_size as i32 + 12) * 2) as f64,
            &status_style,
        )?;

        // Coming soon message
        let coming_style = TextStyle::new()
            .font_family(&theme.font_family)
            .font_size(theme.font_size)
            .color(theme.item_description);

        renderer.text(
            "Configuration panel coming soon...",
            (bounds.x + padding) as f64,
            (bounds.y + bounds.height as i32 / 2) as f64,
            &coming_style,
        )?;

        Ok(())
    }

    fn handle_event(&mut self, _event: &InputEvent) -> PanelAction {
        PanelAction::None
    }

    fn on_mouse_move(&mut self, _x: i32, _y: i32) -> bool {
        false
    }

    fn reset(&mut self) {}

    fn apply(&mut self) -> Result<()> {
        Ok(())
    }

    fn update_status(&mut self, _status: serde_json::Value) {}
}
