//! Slider widget for numeric value selection

use super::{point_in_rect, WidgetEvent, WidgetState};
use anyhow::Result;
use gartk_core::{Color, Rect, Theme};
use gartk_render::{Renderer, TextStyle};

/// Slider track height
const TRACK_HEIGHT: u32 = 6;
/// Slider knob radius
const KNOB_RADIUS: f64 = 8.0;
/// Widget height
const SLIDER_HEIGHT: u32 = 32;

/// A slider widget for selecting numeric values
pub struct Slider {
    pub label: String,
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub bounds: Rect,
    pub state: WidgetState,
    pub show_value: bool,
    dragging: bool,
}

impl Slider {
    /// Create a new slider
    pub fn new(label: impl Into<String>, min: f64, max: f64) -> Self {
        Self {
            label: label.into(),
            value: min,
            min,
            max,
            step: 1.0,
            bounds: Rect::new(0, 0, 300, SLIDER_HEIGHT),
            state: WidgetState::Normal,
            show_value: true,
            dragging: false,
        }
    }

    /// Set initial value
    pub fn with_value(mut self, value: f64) -> Self {
        self.value = value.clamp(self.min, self.max);
        self
    }

    /// Set step size
    pub fn with_step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    /// Enable/disable value display
    pub fn with_show_value(mut self, show: bool) -> Self {
        self.show_value = show;
        self
    }

    /// Set the value
    pub fn set_value(&mut self, value: f64) {
        self.value = value.clamp(self.min, self.max);
    }

    /// Get value as integer
    pub fn value_i32(&self) -> i32 {
        self.value.round() as i32
    }

    /// Get track bounds (the slidable area)
    fn track_bounds(&self) -> Rect {
        let label_width = self.bounds.width / 3;
        let value_width = if self.show_value { 50 } else { 0 };
        let track_width = self.bounds.width - label_width - value_width - 20;

        let track_y = self.bounds.y + (SLIDER_HEIGHT as i32 - TRACK_HEIGHT as i32) / 2;
        Rect::new(
            self.bounds.x + label_width as i32,
            track_y,
            track_width,
            TRACK_HEIGHT,
        )
    }

    /// Get knob position (x coordinate)
    fn knob_x(&self) -> f64 {
        let track = self.track_bounds();
        let ratio = (self.value - self.min) / (self.max - self.min);
        track.x as f64 + (ratio * (track.width as f64 - KNOB_RADIUS * 2.0)) + KNOB_RADIUS
    }

    /// Get knob y coordinate
    fn knob_y(&self) -> f64 {
        let track = self.track_bounds();
        track.y as f64 + TRACK_HEIGHT as f64 / 2.0
    }

    /// Check if point is on/near the knob
    fn point_on_knob(&self, x: i32, y: i32) -> bool {
        let knob_x = self.knob_x();
        let knob_y = self.knob_y();
        let dx = x as f64 - knob_x;
        let dy = y as f64 - knob_y;
        (dx * dx + dy * dy).sqrt() <= KNOB_RADIUS + 4.0
    }

    /// Convert x position to value
    fn x_to_value(&self, x: i32) -> f64 {
        let track = self.track_bounds();
        let track_start = track.x as f64 + KNOB_RADIUS;
        let track_end = track.x as f64 + track.width as f64 - KNOB_RADIUS;
        let ratio = ((x as f64 - track_start) / (track_end - track_start)).clamp(0.0, 1.0);
        let raw_value = self.min + ratio * (self.max - self.min);

        // Snap to step
        if self.step > 0.0 {
            let steps = ((raw_value - self.min) / self.step).round();
            (self.min + steps * self.step).clamp(self.min, self.max)
        } else {
            raw_value.clamp(self.min, self.max)
        }
    }

    /// Handle mouse move
    pub fn on_mouse_move(&mut self, x: i32, y: i32) -> WidgetEvent {
        if self.state == WidgetState::Disabled {
            return WidgetEvent::None;
        }

        if self.dragging {
            let new_value = self.x_to_value(x);
            if (new_value - self.value).abs() > f64::EPSILON {
                self.value = new_value;
                return WidgetEvent::Changed;
            }
        } else {
            let track = self.track_bounds();
            if self.point_on_knob(x, y) || point_in_rect(x, y, track) {
                self.state = WidgetState::Hovered;
            } else {
                self.state = WidgetState::Normal;
            }
        }

        WidgetEvent::None
    }

    /// Handle mouse down
    pub fn on_mouse_down(&mut self, x: i32, y: i32) -> WidgetEvent {
        if self.state == WidgetState::Disabled {
            return WidgetEvent::None;
        }

        let track = self.track_bounds();
        if self.point_on_knob(x, y) {
            self.dragging = true;
            self.state = WidgetState::Focused;
            return WidgetEvent::Focus;
        } else if point_in_rect(x, y, track) {
            // Click on track moves knob to that position
            self.dragging = true;
            self.state = WidgetState::Focused;
            let new_value = self.x_to_value(x);
            if (new_value - self.value).abs() > f64::EPSILON {
                self.value = new_value;
                return WidgetEvent::Changed;
            }
            return WidgetEvent::Focus;
        }

        WidgetEvent::None
    }

    /// Handle mouse up
    pub fn on_mouse_up(&mut self) -> WidgetEvent {
        if self.dragging {
            self.dragging = false;
            self.state = WidgetState::Normal;
            return WidgetEvent::Blur;
        }
        WidgetEvent::None
    }

    /// Check if currently dragging
    pub fn is_dragging(&self) -> bool {
        self.dragging
    }

    /// Render the slider
    pub fn render(&self, renderer: &mut Renderer, theme: &Theme) -> Result<()> {
        // Draw label
        let label_style = TextStyle::new()
            .font_family(&theme.font_family)
            .font_size(theme.font_size)
            .color(theme.foreground);

        let label_y = self.bounds.y + (SLIDER_HEIGHT as i32 - theme.font_size as i32) / 2;
        renderer.text(&self.label, self.bounds.x as f64, label_y as f64, &label_style)?;

        // Draw track
        let track = self.track_bounds();

        // Track background
        let track_bg = Rect::new(
            track.x,
            track.y,
            track.width,
            TRACK_HEIGHT,
        );
        renderer.fill_rounded_rect(track_bg, TRACK_HEIGHT as f64 / 2.0, theme.input_background)?;

        // Filled portion
        let knob_x = self.knob_x();
        let filled_width = (knob_x - track.x as f64) as u32;
        if filled_width > 0 {
            let filled = Rect::new(track.x, track.y, filled_width.min(track.width), TRACK_HEIGHT);
            renderer.fill_rounded_rect(filled, TRACK_HEIGHT as f64 / 2.0, theme.item_selected_background)?;
        }

        // Draw knob
        let knob_color = match self.state {
            WidgetState::Hovered | WidgetState::Focused => Color::from_u8(0xff, 0xff, 0xff, 0xff),
            _ => Color::from_u8(0xea, 0xea, 0xea, 0xff),
        };

        renderer.fill_circle(knob_x, self.knob_y(), KNOB_RADIUS, knob_color)?;

        // Draw subtle shadow around knob using a slightly larger semi-transparent circle
        renderer.fill_circle(
            knob_x,
            self.knob_y(),
            KNOB_RADIUS + 1.0,
            Color::from_u8(0x00, 0x00, 0x00, 0x20),
        )?;
        // Redraw knob on top
        renderer.fill_circle(knob_x, self.knob_y(), KNOB_RADIUS, knob_color)?;

        // Draw value
        if self.show_value {
            let value_text = if self.step >= 1.0 {
                format!("{}", self.value as i32)
            } else {
                format!("{:.1}", self.value)
            };

            let value_style = TextStyle::new()
                .font_family(&theme.font_family)
                .font_size(theme.font_size * 0.9)
                .color(theme.foreground);

            let value_x = track.x + track.width as i32 + 10;
            renderer.text(&value_text, value_x as f64, label_y as f64, &value_style)?;
        }

        Ok(())
    }
}
