//! Read-only list widget for displaying items

use anyhow::Result;
use gartk_core::{Rect, Theme};
use gartk_render::{Renderer, TextStyle};

/// A read-only list displaying string items
pub struct List {
    pub items: Vec<String>,
    pub bounds: Rect,
    pub scroll_offset: i32,
    pub max_visible: usize,
}

impl List {
    /// Create a new list
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            bounds: Rect::new(0, 0, 200, 100),
            scroll_offset: 0,
            max_visible: 5,
        }
    }

    /// Set items
    pub fn with_items(mut self, items: Vec<String>) -> Self {
        self.items = items;
        self
    }

    /// Set max visible items
    pub fn with_max_visible(mut self, max: usize) -> Self {
        self.max_visible = max;
        self
    }

    /// Update items
    pub fn set_items(&mut self, items: Vec<String>) {
        self.items = items;
        self.scroll_offset = 0;
    }

    /// Scroll up
    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    /// Scroll down
    pub fn scroll_down(&mut self) {
        let max_scroll = (self.items.len() as i32 - self.max_visible as i32).max(0);
        if self.scroll_offset < max_scroll {
            self.scroll_offset += 1;
        }
    }

    /// Render the list
    pub fn render(&self, renderer: &mut Renderer, theme: &Theme) -> Result<()> {
        let item_height = (theme.font_size + 8.0) as i32;
        let mut y = self.bounds.y;

        // Draw background
        renderer.fill_rounded_rect(self.bounds, 4.0, theme.input_background)?;
        renderer.stroke_rounded_rect(self.bounds, 4.0, theme.border, 1.0)?;

        let text_style = TextStyle::new()
            .font_family(&theme.font_family)
            .font_size(theme.font_size * 0.9)
            .color(theme.foreground)
            .ellipsize(true)
            .max_width(self.bounds.width as i32 - 16);

        let start = self.scroll_offset as usize;
        let end = (start + self.max_visible).min(self.items.len());

        for item in self.items.iter().skip(start).take(end - start) {
            if y + item_height > self.bounds.y + self.bounds.height as i32 {
                break;
            }
            renderer.text(
                item,
                (self.bounds.x + 8) as f64,
                (y + 4) as f64,
                &text_style,
            )?;
            y += item_height;
        }

        // Draw scroll indicators if needed
        if self.items.len() > self.max_visible {
            let indicator_style = TextStyle::new()
                .font_family(&theme.font_family)
                .font_size(theme.font_size * 0.75)
                .color(theme.item_description);

            if self.scroll_offset > 0 {
                renderer.text(
                    "▲",
                    (self.bounds.x + self.bounds.width as i32 - 16) as f64,
                    self.bounds.y as f64,
                    &indicator_style,
                )?;
            }

            let max_scroll = (self.items.len() as i32 - self.max_visible as i32).max(0);
            if self.scroll_offset < max_scroll {
                renderer.text(
                    "▼",
                    (self.bounds.x + self.bounds.width as i32 - 16) as f64,
                    (self.bounds.y + self.bounds.height as i32 - 16) as f64,
                    &indicator_style,
                )?;
            }

            // Show count
            let count_text = format!("{}/{}", end, self.items.len());
            renderer.text(
                &count_text,
                (self.bounds.x + self.bounds.width as i32 - 50) as f64,
                (self.bounds.y + self.bounds.height as i32 - 16) as f64,
                &indicator_style,
            )?;
        }

        // Empty state
        if self.items.is_empty() {
            let empty_style = TextStyle::new()
                .font_family(&theme.font_family)
                .font_size(theme.font_size * 0.9)
                .color(theme.item_description);

            renderer.text(
                "No items",
                (self.bounds.x + 8) as f64,
                (self.bounds.y + 4) as f64,
                &empty_style,
            )?;
        }

        Ok(())
    }
}

impl Default for List {
    fn default() -> Self {
        Self::new()
    }
}
