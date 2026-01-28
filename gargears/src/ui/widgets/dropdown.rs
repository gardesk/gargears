//! Dropdown selection widget

use super::{point_in_rect, WidgetEvent, WidgetState};
use anyhow::Result;
use gartk_core::{Rect, Theme};
use gartk_render::{Renderer, TextStyle};

/// Dropdown widget height
const DROPDOWN_HEIGHT: u32 = 32;
/// Item height in dropdown
const ITEM_HEIGHT: u32 = 28;
/// Max visible items before scrolling
const MAX_VISIBLE_ITEMS: usize = 6;

/// A dropdown selection widget
pub struct Dropdown {
    pub label: String,
    pub options: Vec<String>,
    pub selected_index: usize,
    pub bounds: Rect,
    pub state: WidgetState,
    pub expanded: bool,
    scroll_offset: usize,
    hovered_item: Option<usize>,
}

impl Dropdown {
    /// Create a new dropdown
    pub fn new(label: impl Into<String>, options: Vec<String>) -> Self {
        Self {
            label: label.into(),
            options,
            selected_index: 0,
            bounds: Rect::new(0, 0, 300, DROPDOWN_HEIGHT),
            state: WidgetState::Normal,
            expanded: false,
            scroll_offset: 0,
            hovered_item: None,
        }
    }

    /// Set initial selection
    pub fn with_selection(mut self, index: usize) -> Self {
        if index < self.options.len() {
            self.selected_index = index;
        }
        self
    }

    /// Get the selected value
    pub fn selected_value(&self) -> Option<&str> {
        self.options.get(self.selected_index).map(|s| s.as_str())
    }

    /// Set selected by value
    pub fn set_selected(&mut self, value: &str) {
        if let Some(idx) = self.options.iter().position(|o| o == value) {
            self.selected_index = idx;
        }
    }

    /// Get the dropdown button bounds (clickable area)
    fn button_bounds(&self) -> Rect {
        Rect::new(
            self.bounds.x + self.bounds.width as i32 / 2,
            self.bounds.y,
            self.bounds.width / 2,
            DROPDOWN_HEIGHT,
        )
    }

    /// Get the expanded dropdown list bounds
    fn list_bounds(&self) -> Rect {
        let visible_count = self.options.len().min(MAX_VISIBLE_ITEMS);
        let list_height = (visible_count as u32 * ITEM_HEIGHT) + 4; // +4 for border
        Rect::new(
            self.bounds.x + self.bounds.width as i32 / 2,
            self.bounds.y + DROPDOWN_HEIGHT as i32,
            self.bounds.width / 2,
            list_height,
        )
    }

    /// Get item bounds for an index (relative to scroll)
    fn item_bounds(&self, display_index: usize) -> Rect {
        let list = self.list_bounds();
        Rect::new(
            list.x + 2,
            list.y + 2 + (display_index as i32 * ITEM_HEIGHT as i32),
            list.width - 4,
            ITEM_HEIGHT,
        )
    }

    /// Handle mouse move
    pub fn on_mouse_move(&mut self, x: i32, y: i32) {
        if self.state == WidgetState::Disabled {
            return;
        }

        if self.expanded {
            // Check if hovering over list items
            let list = self.list_bounds();
            if point_in_rect(x, y, list) {
                let visible_count = self.options.len().min(MAX_VISIBLE_ITEMS);
                for i in 0..visible_count {
                    let item = self.item_bounds(i);
                    if point_in_rect(x, y, item) {
                        self.hovered_item = Some(self.scroll_offset + i);
                        self.state = WidgetState::Hovered;
                        return;
                    }
                }
            }
            self.hovered_item = None;
        }

        // Check button hover
        if point_in_rect(x, y, self.button_bounds()) {
            self.state = WidgetState::Hovered;
        } else if !self.expanded {
            self.state = WidgetState::Normal;
        }
    }

    /// Handle mouse click
    pub fn on_click(&mut self, x: i32, y: i32) -> WidgetEvent {
        if self.state == WidgetState::Disabled {
            return WidgetEvent::None;
        }

        if self.expanded {
            // Check if clicking on an item
            if let Some(hovered) = self.hovered_item {
                if hovered < self.options.len() {
                    self.selected_index = hovered;
                    self.expanded = false;
                    self.hovered_item = None;
                    return WidgetEvent::Changed;
                }
            }

            // Click outside closes dropdown
            let list = self.list_bounds();
            let button = self.button_bounds();
            if !point_in_rect(x, y, list) && !point_in_rect(x, y, button) {
                self.expanded = false;
                self.hovered_item = None;
                return WidgetEvent::None;
            }
        }

        // Toggle expansion
        if point_in_rect(x, y, self.button_bounds()) {
            self.expanded = !self.expanded;
            if self.expanded {
                // Scroll to show selected item
                if self.selected_index >= MAX_VISIBLE_ITEMS {
                    self.scroll_offset = self.selected_index - MAX_VISIBLE_ITEMS + 1;
                } else {
                    self.scroll_offset = 0;
                }
            } else {
                self.hovered_item = None;
            }
            return WidgetEvent::Clicked;
        }

        WidgetEvent::None
    }

    /// Handle scroll (when expanded)
    pub fn on_scroll(&mut self, delta: i32) {
        if !self.expanded || self.options.len() <= MAX_VISIBLE_ITEMS {
            return;
        }

        let max_scroll = self.options.len() - MAX_VISIBLE_ITEMS;
        if delta < 0 && self.scroll_offset > 0 {
            self.scroll_offset = self.scroll_offset.saturating_sub((-delta) as usize);
        } else if delta > 0 {
            self.scroll_offset = (self.scroll_offset + delta as usize).min(max_scroll);
        }
    }

    /// Close the dropdown
    pub fn close(&mut self) {
        self.expanded = false;
        self.hovered_item = None;
    }

    /// Render the dropdown
    pub fn render(&self, renderer: &mut Renderer, theme: &Theme) -> Result<()> {
        // Draw label
        let label_style = TextStyle::new()
            .font_family(&theme.font_family)
            .font_size(theme.font_size)
            .color(theme.foreground);

        let label_y = self.bounds.y + (DROPDOWN_HEIGHT as i32 - theme.font_size as i32) / 2;
        renderer.text(&self.label, self.bounds.x as f64, label_y as f64, &label_style)?;

        // Draw dropdown button
        let button = self.button_bounds();

        // Button background
        let bg_color = if self.state == WidgetState::Hovered && !self.expanded {
            theme.item_hover_background
        } else {
            theme.input_background
        };
        renderer.fill_rounded_rect(button, 4.0, bg_color)?;
        renderer.stroke_rounded_rect(button, 4.0, theme.border, 1.0)?;

        // Selected text
        let text = self
            .options
            .get(self.selected_index)
            .map(|s| s.as_str())
            .unwrap_or("Select...");

        let text_style = TextStyle::new()
            .font_family(&theme.font_family)
            .font_size(theme.font_size * 0.9)
            .color(theme.foreground)
            .ellipsize(true)
            .max_width(button.width as i32 - 28);

        renderer.text(
            text,
            (button.x + 8) as f64,
            (button.y + 8) as f64,
            &text_style,
        )?;

        // Dropdown arrow
        let arrow = if self.expanded { "▲" } else { "▼" };
        let arrow_style = TextStyle::new()
            .font_family(&theme.font_family)
            .font_size(theme.font_size * 0.75)
            .color(theme.item_description);

        renderer.text(
            arrow,
            (button.x + button.width as i32 - 18) as f64,
            (button.y + 10) as f64,
            &arrow_style,
        )?;

        // Draw expanded list
        if self.expanded {
            self.render_list(renderer, theme)?;
        }

        Ok(())
    }

    /// Render the expanded list
    fn render_list(&self, renderer: &mut Renderer, theme: &Theme) -> Result<()> {
        let list = self.list_bounds();

        // List background
        renderer.fill_rounded_rect(list, 4.0, theme.background)?;
        renderer.stroke_rounded_rect(list, 4.0, theme.border, 1.0)?;

        // Items
        let visible_count = self.options.len().min(MAX_VISIBLE_ITEMS);
        for i in 0..visible_count {
            let actual_index = self.scroll_offset + i;
            if actual_index >= self.options.len() {
                break;
            }

            let item = self.item_bounds(i);
            let is_selected = actual_index == self.selected_index;
            let is_hovered = self.hovered_item == Some(actual_index);

            // Item background
            if is_selected {
                renderer.fill_rounded_rect(item, 2.0, theme.item_selected_background)?;
            } else if is_hovered {
                renderer.fill_rounded_rect(item, 2.0, theme.item_hover_background)?;
            }

            // Item text
            let text_color = if is_selected {
                theme.item_selected_foreground
            } else {
                theme.foreground
            };

            let text_style = TextStyle::new()
                .font_family(&theme.font_family)
                .font_size(theme.font_size * 0.9)
                .color(text_color)
                .ellipsize(true)
                .max_width(item.width as i32 - 16);

            renderer.text(
                &self.options[actual_index],
                (item.x + 8) as f64,
                (item.y + 6) as f64,
                &text_style,
            )?;
        }

        // Scroll indicators
        if self.options.len() > MAX_VISIBLE_ITEMS {
            let indicator_style = TextStyle::new()
                .font_family(&theme.font_family)
                .font_size(theme.font_size * 0.6)
                .color(theme.item_description);

            if self.scroll_offset > 0 {
                renderer.text(
                    "▲",
                    (list.x + list.width as i32 - 14) as f64,
                    (list.y + 4) as f64,
                    &indicator_style,
                )?;
            }

            let max_scroll = self.options.len() - MAX_VISIBLE_ITEMS;
            if self.scroll_offset < max_scroll {
                renderer.text(
                    "▼",
                    (list.x + list.width as i32 - 14) as f64,
                    (list.y + list.height as i32 - 14) as f64,
                    &indicator_style,
                )?;
            }
        }

        Ok(())
    }

    /// Get total height including expanded list (for layout purposes)
    pub fn expanded_height(&self) -> u32 {
        if self.expanded {
            let list = self.list_bounds();
            DROPDOWN_HEIGHT + list.height
        } else {
            DROPDOWN_HEIGHT
        }
    }
}
