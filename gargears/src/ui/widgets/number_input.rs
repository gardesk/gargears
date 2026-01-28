//! Number input widget with increment/decrement buttons

use super::{point_in_rect, WidgetEvent, WidgetState};
use anyhow::Result;
use gartk_core::{Color, Rect, Theme};
use gartk_render::{Renderer, TextStyle};

/// Number input with +/- buttons
pub struct NumberInput {
    pub label: String,
    pub value: i32,
    pub min: i32,
    pub max: i32,
    pub step: i32,
    pub bounds: Rect,
    pub state: WidgetState,
    hover_plus: bool,
    hover_minus: bool,
}

impl NumberInput {
    /// Create a new number input
    pub fn new(label: impl Into<String>, value: i32) -> Self {
        Self {
            label: label.into(),
            value,
            min: 0,
            max: 100,
            step: 1,
            bounds: Rect::new(0, 0, 250, 28),
            state: WidgetState::Normal,
            hover_plus: false,
            hover_minus: false,
        }
    }

    /// Set range
    pub fn with_range(mut self, min: i32, max: i32) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    /// Set step
    pub fn with_step(mut self, step: i32) -> Self {
        self.step = step;
        self
    }

    /// Get bounds for minus button
    fn minus_bounds(&self) -> Rect {
        let btn_size = self.bounds.height;
        Rect::new(
            self.bounds.x + self.bounds.width as i32 - (btn_size * 2 + 50) as i32,
            self.bounds.y,
            btn_size,
            btn_size,
        )
    }

    /// Get bounds for plus button
    fn plus_bounds(&self) -> Rect {
        let btn_size = self.bounds.height;
        Rect::new(
            self.bounds.x + self.bounds.width as i32 - btn_size as i32,
            self.bounds.y,
            btn_size,
            btn_size,
        )
    }

    /// Get bounds for value display
    fn value_bounds(&self) -> Rect {
        let btn_size = self.bounds.height;
        Rect::new(
            self.bounds.x + self.bounds.width as i32 - (btn_size + 50) as i32,
            self.bounds.y,
            50,
            btn_size,
        )
    }

    /// Handle mouse move. Returns true if state changed (needs redraw).
    pub fn on_mouse_move(&mut self, x: i32, y: i32) -> bool {
        let old_minus = self.hover_minus;
        let old_plus = self.hover_plus;
        self.hover_minus = point_in_rect(x, y, self.minus_bounds());
        self.hover_plus = point_in_rect(x, y, self.plus_bounds());
        old_minus != self.hover_minus || old_plus != self.hover_plus
    }

    /// Handle mouse click
    pub fn on_click(&mut self, x: i32, y: i32) -> WidgetEvent {
        if self.state == WidgetState::Disabled {
            return WidgetEvent::None;
        }

        if point_in_rect(x, y, self.minus_bounds()) {
            let new_value = (self.value - self.step).max(self.min);
            if new_value != self.value {
                self.value = new_value;
                return WidgetEvent::Changed;
            }
        } else if point_in_rect(x, y, self.plus_bounds()) {
            let new_value = (self.value + self.step).min(self.max);
            if new_value != self.value {
                self.value = new_value;
                return WidgetEvent::Changed;
            }
        }

        WidgetEvent::None
    }

    /// Render the number input
    pub fn render(&self, renderer: &mut Renderer, theme: &Theme) -> Result<()> {
        // Draw label
        let label_style = TextStyle::new()
            .font_family(&theme.font_family)
            .font_size(theme.font_size)
            .color(theme.foreground);

        let label_y = self.bounds.y + (self.bounds.height as i32 - theme.font_size as i32) / 2;
        renderer.text(&self.label, self.bounds.x as f64, label_y as f64, &label_style)?;

        // Draw minus button
        let minus = self.minus_bounds();
        let minus_bg = if self.hover_minus {
            theme.item_hover_background
        } else {
            theme.input_background
        };
        renderer.fill_rounded_rect(minus, 4.0, minus_bg)?;
        renderer.stroke_rounded_rect(minus, 4.0, theme.border, 1.0)?;

        let minus_style = TextStyle::new()
            .font_family(&theme.font_family)
            .font_size(theme.font_size)
            .color(if self.value <= self.min {
                theme.item_description
            } else {
                theme.foreground
            });
        let minus_x = minus.x + (minus.width as i32 - 8) / 2;
        let minus_y = minus.y + (minus.height as i32 - theme.font_size as i32) / 2;
        renderer.text("-", minus_x as f64, minus_y as f64, &minus_style)?;

        // Draw value
        let value_rect = self.value_bounds();
        renderer.fill_rounded_rect(value_rect, 4.0, theme.input_background)?;

        let value_text = self.value.to_string();
        let value_style = TextStyle::new()
            .font_family(&theme.font_family)
            .font_size(theme.font_size)
            .color(theme.foreground);
        let value_size = renderer.measure_text(&value_text, &value_style)?;
        let value_x = value_rect.x + (value_rect.width as i32 - value_size.width as i32) / 2;
        let value_y = value_rect.y + (value_rect.height as i32 - value_size.height as i32) / 2;
        renderer.text(&value_text, value_x as f64, value_y as f64, &value_style)?;

        // Draw plus button
        let plus = self.plus_bounds();
        let plus_bg = if self.hover_plus {
            theme.item_hover_background
        } else {
            theme.input_background
        };
        renderer.fill_rounded_rect(plus, 4.0, plus_bg)?;
        renderer.stroke_rounded_rect(plus, 4.0, theme.border, 1.0)?;

        let plus_style = TextStyle::new()
            .font_family(&theme.font_family)
            .font_size(theme.font_size)
            .color(if self.value >= self.max {
                theme.item_description
            } else {
                theme.foreground
            });
        let plus_x = plus.x + (plus.width as i32 - 8) / 2;
        let plus_y = plus.y + (plus.height as i32 - theme.font_size as i32) / 2;
        renderer.text("+", plus_x as f64, plus_y as f64, &plus_style)?;

        Ok(())
    }
}
