//! Clickable button widget

use super::{point_in_rect, WidgetEvent, WidgetState};
use anyhow::Result;
use gartk_core::{Color, Rect, Theme};
use gartk_render::{Renderer, TextStyle};

/// A clickable button
pub struct Button {
    pub label: String,
    pub bounds: Rect,
    pub state: WidgetState,
    pub primary: bool,
}

impl Button {
    /// Create a new button
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            bounds: Rect::new(0, 0, 100, 32),
            state: WidgetState::Normal,
            primary: false,
        }
    }

    /// Mark as primary (highlighted) button
    pub fn primary(mut self) -> Self {
        self.primary = true;
        self
    }

    /// Set bounds
    pub fn with_bounds(mut self, bounds: Rect) -> Self {
        self.bounds = bounds;
        self
    }

    /// Handle mouse move. Returns true if state changed (needs redraw).
    pub fn on_mouse_move(&mut self, x: i32, y: i32) -> bool {
        if self.state == WidgetState::Disabled {
            return false;
        }
        let old_state = self.state;
        if point_in_rect(x, y, self.bounds) {
            self.state = WidgetState::Hovered;
        } else {
            self.state = WidgetState::Normal;
        }
        old_state != self.state
    }

    /// Handle mouse click
    pub fn on_click(&mut self, x: i32, y: i32) -> WidgetEvent {
        if self.state != WidgetState::Disabled && point_in_rect(x, y, self.bounds) {
            WidgetEvent::Clicked
        } else {
            WidgetEvent::None
        }
    }

    /// Render the button
    pub fn render(&self, renderer: &mut Renderer, theme: &Theme) -> Result<()> {
        // Background color based on state
        let bg_color = match self.state {
            WidgetState::Hovered => {
                if self.primary {
                    Color::from_u8(0x6c, 0xa4, 0xf8, 0xff) // Lighter primary
                } else {
                    theme.item_hover_background
                }
            }
            WidgetState::Disabled => Color::from_u8(0x40, 0x40, 0x40, 0xff),
            _ => {
                if self.primary {
                    Color::from_u8(0x5c, 0x94, 0xe8, 0xff) // Primary blue
                } else {
                    theme.input_background
                }
            }
        };

        // Draw background
        renderer.fill_rounded_rect(self.bounds, theme.border_radius / 2.0, bg_color)?;

        // Draw border
        renderer.stroke_rounded_rect(
            self.bounds,
            theme.border_radius / 2.0,
            theme.border,
            1.0,
        )?;

        // Draw text centered
        let text_style = TextStyle::new()
            .font_family(&theme.font_family)
            .font_size(theme.font_size)
            .color(if self.state == WidgetState::Disabled {
                theme.item_description
            } else {
                theme.foreground
            });

        let text_size = renderer.measure_text(&self.label, &text_style)?;
        let text_x = self.bounds.x + (self.bounds.width as i32 - text_size.width as i32) / 2;
        let text_y = self.bounds.y + (self.bounds.height as i32 - text_size.height as i32) / 2;

        renderer.text(&self.label, text_x as f64, text_y as f64, &text_style)?;

        Ok(())
    }
}
