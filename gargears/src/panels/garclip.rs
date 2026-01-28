//! Configuration panel for garclip (clipboard manager)

use crate::ipc::adapters::GarclipAdapter;
use crate::ui::widgets::{Button, Label, Section, Toggle, WidgetEvent};
use crate::ui::{Panel, PanelAction};
use anyhow::Result;
use gartk_core::{InputEvent, Rect, Theme};
use gartk_render::{Renderer, TextStyle};

/// Garclip configuration panel
pub struct GarclipPanel {
    adapter: GarclipAdapter,

    // Sections
    status_section: Section,
    actions_section: Section,

    // Status display
    history_count_label: Label,
    pinned_count_label: Label,
    owns_clipboard_label: Label,
    watching_primary_label: Label,

    // Action buttons
    clear_button: Button,
    clear_history_button: Button,
    keep_pinned_toggle: Toggle,
    refresh_button: Button,

    // State
    scroll_offset: i32,
}

impl GarclipPanel {
    pub fn new(mut adapter: GarclipAdapter) -> Self {
        let (history, pinned, owns, watching) = if adapter.is_connected() {
            if let Ok(status) = adapter.status() {
                (
                    format!("{} item(s)", status.history_count),
                    format!("{} pinned", status.pinned_count),
                    if status.owns_clipboard { "Yes" } else { "No" },
                    if status.watching_primary { "Yes" } else { "No" },
                )
            } else {
                ("Unknown".into(), "Unknown".into(), "Unknown", "Unknown")
            }
        } else {
            ("N/A".into(), "N/A".into(), "N/A", "N/A")
        };

        Self {
            adapter,
            status_section: Section::new("Status"),
            actions_section: Section::new("Actions"),
            history_count_label: Label::new("History Count", &history),
            pinned_count_label: Label::new("Pinned Count", &pinned),
            owns_clipboard_label: Label::new("Owns Clipboard", owns),
            watching_primary_label: Label::new("Watching PRIMARY", watching),
            clear_button: Button::new("Clear Clipboard"),
            clear_history_button: Button::new("Clear History").primary(),
            keep_pinned_toggle: Toggle::new("Keep Pinned Items", true),
            refresh_button: Button::new("Refresh"),
            scroll_offset: 0,
        }
    }

    fn refresh_status(&mut self) {
        if !self.adapter.is_connected() {
            let _ = self.adapter.connect();
        }

        if let Ok(status) = self.adapter.status() {
            self.history_count_label.text = format!("{} item(s)", status.history_count);
            self.pinned_count_label.text = format!("{} pinned", status.pinned_count);
            self.owns_clipboard_label.text = if status.owns_clipboard { "Yes" } else { "No" }.into();
            self.watching_primary_label.text = if status.watching_primary { "Yes" } else { "No" }.into();
        }
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
            self.history_count_label.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;

            self.pinned_count_label.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;

            self.owns_clipboard_label.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;

            self.watching_primary_label.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;
        }

        y += padding / 2;

        // Actions section
        self.actions_section.bounds =
            Rect::new(bounds.x + padding, y, widget_width, section_height as u32);
        y += section_height;

        if self.actions_section.expanded {
            self.clear_button.bounds = Rect::new(
                bounds.x + padding + 16,
                y,
                120,
                32,
            );

            self.refresh_button.bounds = Rect::new(
                bounds.x + padding + 16 + 128,
                y,
                80,
                32,
            );

            y += 40;

            self.keep_pinned_toggle.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height + 8;

            self.clear_history_button.bounds = Rect::new(
                bounds.x + padding + 16,
                y,
                120,
                32,
            );
        }
    }
}

impl Panel for GarclipPanel {
    fn name(&self) -> &str {
        "garclip"
    }

    fn description(&self) -> &str {
        "Clipboard manager"
    }

    fn is_dirty(&self) -> bool {
        false
    }

    fn render(&mut self, renderer: &mut Renderer, bounds: Rect, theme: &Theme) -> Result<()> {
        self.layout_widgets(bounds, theme);

        // Render sections
        self.status_section.render(renderer, theme)?;
        if self.status_section.expanded {
            self.history_count_label.render(renderer, theme)?;
            self.pinned_count_label.render(renderer, theme)?;
            self.owns_clipboard_label.render(renderer, theme)?;
            self.watching_primary_label.render(renderer, theme)?;
        }

        self.actions_section.render(renderer, theme)?;
        if self.actions_section.expanded {
            self.clear_button.render(renderer, theme)?;
            self.refresh_button.render(renderer, theme)?;
            self.keep_pinned_toggle.render(renderer, theme)?;
            self.clear_history_button.render(renderer, theme)?;
        }

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
                if let WidgetEvent::Changed = self.actions_section.on_click(x, y) {
                    return PanelAction::Redraw;
                }

                // Toggle
                if let WidgetEvent::Changed = self.keep_pinned_toggle.on_click(x, y) {
                    return PanelAction::Redraw;
                }

                // Action buttons
                if let WidgetEvent::Clicked = self.clear_button.on_click(x, y) {
                    if self.adapter.is_connected() {
                        let _ = self.adapter.clear();
                        self.refresh_status();
                    }
                    return PanelAction::Redraw;
                }
                if let WidgetEvent::Clicked = self.clear_history_button.on_click(x, y) {
                    if self.adapter.is_connected() {
                        let _ = self.adapter.clear_history(self.keep_pinned_toggle.value);
                        self.refresh_status();
                    }
                    return PanelAction::Redraw;
                }
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
        changed |= self.actions_section.on_mouse_move(x, y);
        changed |= self.keep_pinned_toggle.on_mouse_move(x, y);
        changed |= self.clear_button.on_mouse_move(x, y);
        changed |= self.clear_history_button.on_mouse_move(x, y);
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
