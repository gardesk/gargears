//! Static text label widget

use anyhow::Result;
use gartk_core::{Color, Rect, Theme};
use gartk_render::{Renderer, TextStyle};

/// A static text label with optional key-value display
pub struct Label {
    pub label: String,
    pub text: String,
    pub bounds: Rect,
    pub color: Option<Color>,
    pub font_size_multiplier: f64,
}

impl Label {
    /// Create a new label
    pub fn new(label: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            text: text.into(),
            bounds: Rect::new(0, 0, 0, 0),
            color: None,
            font_size_multiplier: 1.0,
        }
    }

    /// Create a label with just text (no key)
    pub fn text_only(text: impl Into<String>) -> Self {
        Self {
            label: String::new(),
            text: text.into(),
            bounds: Rect::new(0, 0, 0, 0),
            color: None,
            font_size_multiplier: 1.0,
        }
    }

    /// Set custom color
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Set font size multiplier
    pub fn with_size(mut self, multiplier: f64) -> Self {
        self.font_size_multiplier = multiplier;
        self
    }

    /// Render the label at a specific position
    pub fn render_at(&self, renderer: &mut Renderer, x: i32, y: i32, theme: &Theme) -> Result<()> {
        let style = TextStyle::new()
            .font_family(&theme.font_family)
            .font_size(theme.font_size * self.font_size_multiplier)
            .color(self.color.unwrap_or(theme.foreground));

        renderer.text(&self.text, x as f64, y as f64, &style)?;
        Ok(())
    }

    /// Render the label using its bounds (with optional label prefix)
    pub fn render(&self, renderer: &mut Renderer, theme: &Theme) -> Result<()> {
        let y = self.bounds.y + (self.bounds.height as i32 - theme.font_size as i32) / 2;

        // Draw label if present
        if !self.label.is_empty() {
            let label_style = TextStyle::new()
                .font_family(&theme.font_family)
                .font_size(theme.font_size * self.font_size_multiplier)
                .color(theme.foreground);

            renderer.text(&self.label, self.bounds.x as f64, y as f64, &label_style)?;

            // Draw value on the right side
            let value_style = TextStyle::new()
                .font_family(&theme.font_family)
                .font_size(theme.font_size * self.font_size_multiplier)
                .color(self.color.unwrap_or(theme.item_description));

            let value_x = self.bounds.x + self.bounds.width as i32 / 2;
            renderer.text(&self.text, value_x as f64, y as f64, &value_style)?;
        } else {
            let style = TextStyle::new()
                .font_family(&theme.font_family)
                .font_size(theme.font_size * self.font_size_multiplier)
                .color(self.color.unwrap_or(theme.foreground));

            renderer.text(&self.text, self.bounds.x as f64, y as f64, &style)?;
        }

        Ok(())
    }

    /// Measure the label size
    pub fn measure(&self, renderer: &Renderer, theme: &Theme) -> Result<(u32, u32)> {
        let style = TextStyle::new()
            .font_family(&theme.font_family)
            .font_size(theme.font_size * self.font_size_multiplier);

        let size = renderer.measure_text(&self.text, &style)?;
        Ok((size.width, size.height))
    }
}
