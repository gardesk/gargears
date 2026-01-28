//! Text input widget

use super::{point_in_rect, WidgetEvent, WidgetState};
use anyhow::Result;
use gartk_core::{Key, Rect, Theme};
use gartk_render::{Renderer, TextStyle};

/// A text input field
pub struct TextInput {
    pub label: String,
    pub value: String,
    pub placeholder: String,
    pub bounds: Rect,
    pub state: WidgetState,
    pub cursor: usize,
}

impl TextInput {
    /// Create a new text input
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        let value = value.into();
        let cursor = value.len();
        Self {
            label: label.into(),
            value,
            placeholder: String::new(),
            bounds: Rect::new(0, 0, 250, 28),
            state: WidgetState::Normal,
            cursor,
        }
    }

    /// Set placeholder text
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Get input field bounds
    fn input_bounds(&self) -> Rect {
        let input_width = 150;
        Rect::new(
            self.bounds.x + self.bounds.width as i32 - input_width,
            self.bounds.y,
            input_width as u32,
            self.bounds.height,
        )
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
                self.value.insert(self.cursor, *c);
                self.cursor += 1;
                WidgetEvent::Changed
            }
            Key::Space => {
                self.value.insert(self.cursor, ' ');
                self.cursor += 1;
                WidgetEvent::Changed
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

    /// Render the text input
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

        // Draw value or placeholder
        let display_text = if self.value.is_empty() {
            &self.placeholder
        } else {
            &self.value
        };

        let text_color = if self.value.is_empty() {
            theme.input_placeholder
        } else {
            theme.input_foreground
        };

        let text_style = TextStyle::new()
            .font_family(&theme.font_family)
            .font_size(theme.font_size)
            .color(text_color)
            .ellipsize(true)
            .max_width(input.width as i32 - 12);

        let text_x = input.x + 6;
        let text_y = input.y + (input.height as i32 - theme.font_size as i32) / 2;
        renderer.text(display_text, text_x as f64, text_y as f64, &text_style)?;

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

        Ok(())
    }
}
