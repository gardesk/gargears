//! Color picker widget with hex input and preview

use super::{point_in_rect, WidgetEvent, WidgetState};
use anyhow::Result;
use gartk_core::{Color, Key, Rect, Theme};
use gartk_render::{Renderer, TextStyle};

/// A color picker with hex input and preview square
pub struct ColorPicker {
    pub label: String,
    pub value: String,
    pub bounds: Rect,
    pub state: WidgetState,
    pub cursor: usize,
}

impl ColorPicker {
    /// Create a new color picker
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        let value = value.into();
        let cursor = value.len();
        Self {
            label: label.into(),
            value,
            bounds: Rect::new(0, 0, 250, 28),
            state: WidgetState::Normal,
            cursor,
        }
    }

    /// Parse hex color string to Color
    fn parse_color(&self) -> Option<Color> {
        let hex = self.value.trim_start_matches('#');
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color::from_u8(r, g, b, 0xff))
        } else if hex.len() == 8 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(Color::from_u8(r, g, b, a))
        } else {
            None
        }
    }

    /// Get input field bounds
    fn input_bounds(&self) -> Rect {
        let preview_size = 24;
        let input_width = 100;
        Rect::new(
            self.bounds.x + self.bounds.width as i32 - input_width - preview_size - 8,
            self.bounds.y + 2,
            input_width as u32,
            self.bounds.height - 4,
        )
    }

    /// Get preview square bounds
    fn preview_bounds(&self) -> Rect {
        let preview_size = 24;
        Rect::new(
            self.bounds.x + self.bounds.width as i32 - preview_size - 2,
            self.bounds.y + (self.bounds.height as i32 - preview_size) / 2,
            preview_size as u32,
            preview_size as u32,
        )
    }

    /// Clear focus from this widget
    pub fn blur(&mut self) {
        if self.state == WidgetState::Focused {
            self.state = WidgetState::Normal;
        }
    }

    /// Handle mouse move. Returns true if state changed (needs redraw).
    pub fn on_mouse_move(&mut self, x: i32, y: i32) -> bool {
        if self.state == WidgetState::Disabled || self.state == WidgetState::Focused {
            return false;
        }
        let old_state = self.state;
        if point_in_rect(x, y, self.input_bounds()) {
            self.state = WidgetState::Hovered;
        } else {
            self.state = WidgetState::Normal;
        }
        old_state != self.state
    }

    /// Handle mouse click
    pub fn on_click(&mut self, x: i32, y: i32) -> WidgetEvent {
        if self.state != WidgetState::Disabled && point_in_rect(x, y, self.input_bounds()) {
            self.state = WidgetState::Focused;
            WidgetEvent::Focus
        } else if self.state == WidgetState::Focused {
            self.state = WidgetState::Normal;
            WidgetEvent::Blur
        } else {
            WidgetEvent::None
        }
    }

    /// Handle key press (only when focused)
    pub fn on_key(&mut self, key: &Key) -> WidgetEvent {
        if self.state != WidgetState::Focused {
            return WidgetEvent::None;
        }

        match key {
            Key::Char(c) => {
                // Only allow hex characters and #
                if c.is_ascii_hexdigit() || *c == '#' {
                    self.value.insert(self.cursor, *c);
                    self.cursor += 1;
                    WidgetEvent::Changed
                } else {
                    WidgetEvent::None
                }
            }
            Key::Backspace => {
                if self.cursor > 0 {
                    self.value.remove(self.cursor - 1);
                    self.cursor -= 1;
                    WidgetEvent::Changed
                } else {
                    WidgetEvent::None
                }
            }
            Key::Delete => {
                if self.cursor < self.value.len() {
                    self.value.remove(self.cursor);
                    WidgetEvent::Changed
                } else {
                    WidgetEvent::None
                }
            }
            Key::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
                WidgetEvent::None
            }
            Key::Right => {
                if self.cursor < self.value.len() {
                    self.cursor += 1;
                }
                WidgetEvent::None
            }
            Key::Home => {
                self.cursor = 0;
                WidgetEvent::None
            }
            Key::End => {
                self.cursor = self.value.len();
                WidgetEvent::None
            }
            Key::Escape | Key::Return => {
                self.state = WidgetState::Normal;
                WidgetEvent::Blur
            }
            _ => WidgetEvent::None,
        }
    }

    /// Render the color picker
    pub fn render(&self, renderer: &mut Renderer, theme: &Theme) -> Result<()> {
        // Draw label
        let label_style = TextStyle::new()
            .font_family(&theme.font_family)
            .font_size(theme.font_size)
            .color(theme.foreground);

        let label_y = self.bounds.y + (self.bounds.height as i32 - theme.font_size as i32) / 2;
        renderer.text(&self.label, self.bounds.x as f64, label_y as f64, &label_style)?;

        // Draw input field
        let input = self.input_bounds();

        let bg_color = match self.state {
            WidgetState::Focused => theme.input_background,
            WidgetState::Hovered => theme.item_hover_background,
            _ => theme.input_background,
        };

        renderer.fill_rounded_rect(input, 4.0, bg_color)?;

        let border_color = if self.state == WidgetState::Focused {
            theme.input_cursor
        } else {
            theme.border
        };
        renderer.stroke_rounded_rect(input, 4.0, border_color, 1.0)?;

        // Draw value
        let text_style = TextStyle::new()
            .font_family(&theme.font_family)
            .font_size(theme.font_size)
            .color(theme.input_foreground)
            .ellipsize(true)
            .max_width(input.width as i32 - 12);

        let text_x = input.x + 6;
        let text_y = input.y + (input.height as i32 - theme.font_size as i32) / 2;
        renderer.text(&self.value, text_x as f64, text_y as f64, &text_style)?;

        // Draw cursor if focused
        if self.state == WidgetState::Focused {
            let cursor_text = &self.value[..self.cursor];
            let cursor_style = TextStyle::new()
                .font_family(&theme.font_family)
                .font_size(theme.font_size);
            let cursor_size = renderer.measure_text(cursor_text, &cursor_style)?;
            let cursor_x = text_x + cursor_size.width as i32;

            renderer.fill_rect(
                Rect::new(cursor_x, input.y + 4, 2, input.height - 8),
                theme.input_cursor,
            )?;
        }

        // Draw color preview square
        let preview = self.preview_bounds();

        // Draw checkerboard pattern for transparency indication
        let checker_color = Color::from_u8(0x80, 0x80, 0x80, 0xff);
        renderer.fill_rounded_rect(preview, 3.0, checker_color)?;

        // Draw the actual color
        if let Some(color) = self.parse_color() {
            renderer.fill_rounded_rect(preview, 3.0, color)?;
        }

        // Draw preview border
        renderer.stroke_rounded_rect(preview, 3.0, theme.border, 1.0)?;

        Ok(())
    }
}
