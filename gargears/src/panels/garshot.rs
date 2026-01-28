//! Configuration panel for garshot (screenshot tool)

use crate::ipc::adapters::GarshotAdapter;
use crate::ui::widgets::{Button, Label, Section, WidgetEvent};
use crate::ui::{Panel, PanelAction};
use anyhow::Result;
use gartk_core::{InputEvent, Rect, Theme};
use gartk_render::{Renderer, TextStyle};

/// Garshot configuration panel
pub struct GarshotPanel {
    adapter: GarshotAdapter,

    // Sections
    status_section: Section,
    info_section: Section,

    // Status display
    ready_label: Label,
    monitors_label: Label,

    // Action buttons
    refresh_button: Button,

    // State
    scroll_offset: i32,
}

impl GarshotPanel {
    pub fn new(mut adapter: GarshotAdapter) -> Self {
        let ready = if adapter.is_connected() {
            if let Ok(status) = adapter.status() {
                if status.ready { "Ready" } else { "Not ready" }
            } else {
                "Unknown"
            }
        } else {
            "Not connected"
        };

        let monitors = if adapter.is_connected() {
            if let Ok(Some(monitors)) = adapter.list_monitors() {
                if let Some(arr) = monitors.as_array() {
                    format!("{} monitor(s) detected", arr.len())
                } else {
                    "Monitors: Unknown format".into()
                }
            } else {
                "No monitor info".into()
            }
        } else {
            "Not connected".into()
        };

        Self {
            adapter,
            status_section: Section::new("Status"),
            info_section: Section::new("Information"),
            ready_label: Label::new("Status", ready),
            monitors_label: Label::new("Monitors", &monitors),
            refresh_button: Button::new("Refresh").primary(),
            scroll_offset: 0,
        }
    }

    fn refresh_status(&mut self) {
        if !self.adapter.is_connected() {
            let _ = self.adapter.connect();
        }

        let ready = if self.adapter.is_connected() {
            if let Ok(status) = self.adapter.status() {
                if status.ready { "Ready" } else { "Not ready" }
            } else {
                "Error"
            }
        } else {
            "Not connected"
        };

        let monitors = if self.adapter.is_connected() {
            if let Ok(Some(monitors)) = self.adapter.list_monitors() {
                if let Some(arr) = monitors.as_array() {
                    format!("{} monitor(s) detected", arr.len())
                } else {
                    "Monitors: Unknown format".into()
                }
            } else {
                "No monitor info".into()
            }
        } else {
            "Not connected".into()
        };

        self.ready_label.text = ready.into();
        self.monitors_label.text = monitors;
    }

    fn layout_widgets(&mut self, bounds: Rect, theme: &Theme) {
        let padding = theme.padding as i32;
        let row_height = 32;
        let section_height = 32;
        let widget_width = bounds.width - (padding * 2) as u32;

        let mut y = bounds.y + padding - self.scroll_offset;

        // Status section
        self.status_section.bounds =
            Rect::new(bounds.x + padding, y, widget_width, section_height as u32);
        y += section_height;

        if self.status_section.expanded {
            self.ready_label.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;
        }

        y += padding / 2;

        // Info section
        self.info_section.bounds =
            Rect::new(bounds.x + padding, y, widget_width, section_height as u32);
        y += section_height;

        if self.info_section.expanded {
            self.monitors_label.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;

            self.refresh_button.bounds = Rect::new(
                bounds.x + padding + 16,
                y + 8,
                100,
                32,
            );
        }
    }
}

impl Panel for GarshotPanel {
    fn name(&self) -> &str {
        "garshot"
    }

    fn description(&self) -> &str {
        "Screenshot tool"
    }

    fn is_dirty(&self) -> bool {
        false
    }

    fn render(&mut self, renderer: &mut Renderer, bounds: Rect, theme: &Theme) -> Result<()> {
        self.layout_widgets(bounds, theme);

        // Render sections
        self.status_section.render(renderer, theme)?;
        if self.status_section.expanded {
            self.ready_label.render(renderer, theme)?;
        }

        self.info_section.render(renderer, theme)?;
        if self.info_section.expanded {
            self.monitors_label.render(renderer, theme)?;
            self.refresh_button.render(renderer, theme)?;
        }

        // Hint text
        let hint_style = TextStyle::new()
            .font_family(&theme.font_family)
            .font_size(theme.font_size * 0.85)
            .color(theme.item_description);

        renderer.text(
            "Use keybinds or CLI to capture screenshots",
            (bounds.x + theme.padding as i32) as f64,
            (bounds.y + bounds.height as i32 - theme.padding as i32 - 40) as f64,
            &hint_style,
        )?;

        // Connection status
        let status_color = if self.adapter.is_connected() {
            gartk_core::Color::from_u8(0x50, 0xfa, 0x7b, 0xff)
        } else {
            gartk_core::Color::from_u8(0xff, 0x55, 0x55, 0xff)
        };

        let status_style = TextStyle::new()
            .font_family(&theme.font_family)
            .font_size(theme.font_size * 0.85)
            .color(status_color);

        let status_text = if self.adapter.is_connected() {
            "Connected"
        } else {
            "Not running"
        };

        renderer.text(
            status_text,
            (bounds.x + theme.padding as i32) as f64,
            (bounds.y + bounds.height as i32 - theme.padding as i32 - 16) as f64,
            &status_style,
        )?;

        Ok(())
    }

    fn handle_event(&mut self, event: &InputEvent) -> PanelAction {
        match event {
            InputEvent::MousePress(me) => {
                let x = me.position.x;
                let y = me.position.y;

                // Check sections
                if let WidgetEvent::Changed = self.status_section.on_click(x, y) {
                    return PanelAction::Redraw;
                }
                if let WidgetEvent::Changed = self.info_section.on_click(x, y) {
                    return PanelAction::Redraw;
                }

                // Refresh button
                if let WidgetEvent::Clicked = self.refresh_button.on_click(x, y) {
                    self.refresh_status();
                    return PanelAction::Redraw;
                }

                PanelAction::None
            }
            InputEvent::Scroll(se) => {
                self.scroll_offset = (self.scroll_offset - se.delta_y * 20).max(0);
                PanelAction::Redraw
            }
            _ => PanelAction::None,
        }
    }

    fn on_mouse_move(&mut self, x: i32, y: i32) -> bool {
        let mut changed = false;
        changed |= self.status_section.on_mouse_move(x, y);
        changed |= self.info_section.on_mouse_move(x, y);
        changed |= self.refresh_button.on_mouse_move(x, y);
        changed
    }

    fn reset(&mut self) {}

    fn apply(&mut self) -> Result<()> {
        Ok(())
    }

    fn update_status(&mut self, _status: serde_json::Value) {
        self.refresh_status();
    }
}
