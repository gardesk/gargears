//! Section widget for grouping settings

use super::{point_in_rect, WidgetEvent};
use anyhow::Result;
use gartk_core::{Rect, Theme};
use gartk_render::{Renderer, TextStyle};

/// A collapsible section header
pub struct Section {
    pub title: String,
    pub expanded: bool,
    pub bounds: Rect,
    hovered: bool,
}

impl Section {
    /// Create a new section
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            expanded: true,
            bounds: Rect::new(0, 0, 200, 32),
            hovered: false,
        }
    }

    /// Handle mouse move. Returns true if state changed (needs redraw).
    pub fn on_mouse_move(&mut self, x: i32, y: i32) -> bool {
        let was_hovered = self.hovered;
        self.hovered = point_in_rect(x, y, self.bounds);
        was_hovered != self.hovered
    }

    /// Handle mouse click
    pub fn on_click(&mut self, x: i32, y: i32) -> WidgetEvent {
        if point_in_rect(x, y, self.bounds) {
            self.expanded = !self.expanded;
            WidgetEvent::Changed
        } else {
            WidgetEvent::None
        }
    }

    /// Render the section header
    pub fn render(&self, renderer: &mut Renderer, theme: &Theme) -> Result<()> {
        // Draw background on hover
        if self.hovered {
            renderer.fill_rounded_rect(self.bounds, 4.0, theme.item_hover_background)?;
        }

        // Draw expand/collapse indicator
        let indicator = if self.expanded { "▼" } else { "▶" };
        let indicator_style = TextStyle::new()
            .font_family(&theme.font_family)
            .font_size(theme.font_size * 0.75)
            .color(theme.item_description);

        let indicator_y = self.bounds.y + (self.bounds.height as i32 - theme.font_size as i32) / 2;
        renderer.text(
            indicator,
            (self.bounds.x + 4) as f64,
            indicator_y as f64,
            &indicator_style,
        )?;

        // Draw title
        let title_style = TextStyle::new()
            .font_family(&theme.font_family)
            .font_size(theme.font_size)
            .color(theme.foreground);

        renderer.text(
            &self.title,
            (self.bounds.x + 20) as f64,
            indicator_y as f64,
            &title_style,
        )?;

        // Draw separator line
        let line_y = self.bounds.y + self.bounds.height as i32 - 1;
        renderer.fill_rect(
            Rect::new(self.bounds.x, line_y, self.bounds.width, 1),
            theme.border,
        )?;

        Ok(())
    }
}
