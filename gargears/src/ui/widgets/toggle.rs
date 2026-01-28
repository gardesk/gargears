//! Toggle switch widget

use super::{point_in_rect, WidgetEvent, WidgetState};
use anyhow::Result;
use gartk_core::{Color, Rect, Theme};
use gartk_render::{Renderer, TextStyle};

/// Toggle width and height
const TOGGLE_WIDTH: u32 = 44;
const TOGGLE_HEIGHT: u32 = 24;

/// A toggle switch with label
pub struct Toggle {
    pub label: String,
    pub value: bool,
    pub bounds: Rect,
    pub state: WidgetState,
}

impl Toggle {
    /// Create a new toggle
    pub fn new(label: impl Into<String>, value: bool) -> Self {
        Self {
            label: label.into(),
            value,
            bounds: Rect::new(0, 0, 200, TOGGLE_HEIGHT),
            state: WidgetState::Normal,
        }
    }

    /// Get toggle switch bounds (the clickable part)
    fn switch_bounds(&self) -> Rect {
        Rect::new(
            self.bounds.x + self.bounds.width as i32 - TOGGLE_WIDTH as i32,
            self.bounds.y,
            TOGGLE_WIDTH,
            TOGGLE_HEIGHT,
        )
    }

    /// Handle mouse move. Returns true if state changed (needs redraw).
    pub fn on_mouse_move(&mut self, x: i32, y: i32) -> bool {
        if self.state == WidgetState::Disabled {
            return false;
        }
        let old_state = self.state;
        if point_in_rect(x, y, self.switch_bounds()) {
            self.state = WidgetState::Hovered;
        } else {
            self.state = WidgetState::Normal;
        }
        old_state != self.state
    }

    /// Handle mouse click
    pub fn on_click(&mut self, x: i32, y: i32) -> WidgetEvent {
        if self.state != WidgetState::Disabled && point_in_rect(x, y, self.switch_bounds()) {
            self.value = !self.value;
            WidgetEvent::Changed
        } else {
            WidgetEvent::None
        }
    }

    /// Render the toggle
    pub fn render(&self, renderer: &mut Renderer, theme: &Theme) -> Result<()> {
        // Draw label
        let label_style = TextStyle::new()
            .font_family(&theme.font_family)
            .font_size(theme.font_size)
            .color(theme.foreground);

        let label_y = self.bounds.y + (self.bounds.height as i32 - theme.font_size as i32) / 2;
        renderer.text(&self.label, self.bounds.x as f64, label_y as f64, &label_style)?;

        // Draw toggle switch
        let switch = self.switch_bounds();

        // Track background
        let track_color = if self.value {
            Color::from_u8(0x50, 0xfa, 0x7b, 0xff) // Green when on
        } else {
            Color::from_u8(0x44, 0x47, 0x5a, 0xff) // Gray when off
        };

        renderer.fill_rounded_rect(switch, TOGGLE_HEIGHT as f64 / 2.0, track_color)?;

        // Knob
        let knob_padding = 2;
        let knob_size = TOGGLE_HEIGHT - (knob_padding * 2) as u32;
        let knob_x = if self.value {
            switch.x + switch.width as i32 - knob_size as i32 - knob_padding
        } else {
            switch.x + knob_padding
        };

        let knob_color = if self.state == WidgetState::Hovered {
            Color::from_u8(0xff, 0xff, 0xff, 0xff)
        } else {
            Color::from_u8(0xea, 0xea, 0xea, 0xff)
        };

        renderer.fill_circle(
            (knob_x + knob_size as i32 / 2) as f64,
            (switch.y + TOGGLE_HEIGHT as i32 / 2) as f64,
            knob_size as f64 / 2.0,
            knob_color,
        )?;

        Ok(())
    }
}
