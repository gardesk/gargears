//! Configuration panel for garnotify (notification daemon)

use crate::ipc::adapters::GarnotifyAdapter;
use crate::ui::widgets::{Button, Label, Section, Toggle, WidgetEvent};
use crate::ui::{Panel, PanelAction};
use anyhow::Result;
use gartk_core::{InputEvent, Rect, Theme};
use gartk_render::{Renderer, TextStyle};

/// Garnotify configuration panel
pub struct GarnotifyPanel {
    adapter: GarnotifyAdapter,

    // Sections
    status_section: Section,
    actions_section: Section,

    // Status display
    paused_toggle: Toggle,
    pending_count_label: Label,
    history_count_label: Label,

    // Action buttons
    clear_button: Button,
    clear_all_button: Button,
    refresh_button: Button,

    // State
    original_paused: bool,
    dirty: bool,
    scroll_offset: i32,
    instant_apply: bool,
    pending_change: Option<&'static str>,
}

impl GarnotifyPanel {
    pub fn new(mut adapter: GarnotifyAdapter) -> Self {
        let (paused, pending, history) = if adapter.is_connected() {
            if let Ok(status) = adapter.status() {
                (
                    status.paused,
                    format!("{} pending", status.pending_count),
                    format!("{} in history", status.history_count),
                )
            } else {
                (false, "Unknown".into(), "Unknown".into())
            }
        } else {
            (false, "N/A".into(), "N/A".into())
        };

        Self {
            adapter,
            status_section: Section::new("Status"),
            actions_section: Section::new("Actions"),
            paused_toggle: Toggle::new("Paused", paused),
            pending_count_label: Label::new("Pending", &pending),
            history_count_label: Label::new("History", &history),
            clear_button: Button::new("Clear Current"),
            clear_all_button: Button::new("Clear All").primary(),
            refresh_button: Button::new("Refresh"),
            original_paused: paused,
            dirty: false,
            scroll_offset: 0,
            instant_apply: true,
            pending_change: None,
        }
    }

    fn refresh_status(&mut self) {
        if !self.adapter.is_connected() {
            let _ = self.adapter.connect();
        }

        if let Ok(status) = self.adapter.status() {
            self.paused_toggle.value = status.paused;
            self.original_paused = status.paused;
            self.pending_count_label.text = format!("{} pending", status.pending_count);
            self.history_count_label.text = format!("{} in history", status.history_count);
            self.dirty = false;
        }
    }

    fn check_dirty(&mut self) {
        self.dirty = self.paused_toggle.value != self.original_paused;
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
            self.paused_toggle.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;

            self.pending_count_label.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;

            self.history_count_label.bounds =
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
                110,
                32,
            );

            self.clear_all_button.bounds = Rect::new(
                bounds.x + padding + 16 + 118,
                y,
                90,
                32,
            );

            self.refresh_button.bounds = Rect::new(
                bounds.x + padding + 16 + 118 + 98,
                y,
                80,
                32,
            );
        }
    }
}

impl Panel for GarnotifyPanel {
    fn name(&self) -> &str {
        "garnotify"
    }

    fn description(&self) -> &str {
        "Notification daemon"
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn render(&mut self, renderer: &mut Renderer, bounds: Rect, theme: &Theme) -> Result<()> {
        self.layout_widgets(bounds, theme);

        // Render sections
        self.status_section.render(renderer, theme)?;
        if self.status_section.expanded {
            self.paused_toggle.render(renderer, theme)?;
            self.pending_count_label.render(renderer, theme)?;
            self.history_count_label.render(renderer, theme)?;
        }

        self.actions_section.render(renderer, theme)?;
        if self.actions_section.expanded {
            self.clear_button.render(renderer, theme)?;
            self.clear_all_button.render(renderer, theme)?;
            self.refresh_button.render(renderer, theme)?;
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

        // Dirty indicator
        if self.dirty {
            let indicator_style = TextStyle::new()
                .font_family(&theme.font_family)
                .font_size(theme.font_size * 0.75)
                .color(gartk_core::Color::from_u8(0xff, 0xb8, 0x6c, 0xff));

            renderer.text(
                "Unsaved changes",
                (bounds.x + bounds.width as i32 - theme.padding as i32 - 100) as f64,
                (bounds.y + bounds.height as i32 - theme.padding as i32 - 16) as f64,
                &indicator_style,
            )?;
        }

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
                if let WidgetEvent::Changed = self.paused_toggle.on_click(x, y) {
                    self.check_dirty();
                    self.pending_change = Some("paused");
                    return if self.instant_apply { PanelAction::InstantApply } else { PanelAction::Redraw };
                }

                // Action buttons
                if let WidgetEvent::Clicked = self.clear_button.on_click(x, y) {
                    if self.adapter.is_connected() {
                        let _ = self.adapter.clear();
                        self.refresh_status();
                    }
                    return PanelAction::Redraw;
                }
                if let WidgetEvent::Clicked = self.clear_all_button.on_click(x, y) {
                    if self.adapter.is_connected() {
                        let _ = self.adapter.clear_all();
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
        changed |= self.paused_toggle.on_mouse_move(x, y);
        changed |= self.clear_button.on_mouse_move(x, y);
        changed |= self.clear_all_button.on_mouse_move(x, y);
        changed |= self.refresh_button.on_mouse_move(x, y);
        changed
    }

    fn reset(&mut self) {
        self.paused_toggle.value = self.original_paused;
        self.dirty = false;
    }

    fn apply(&mut self) -> Result<()> {
        if !self.adapter.is_connected() {
            self.adapter.connect()?;
        }

        if self.paused_toggle.value != self.original_paused {
            if self.paused_toggle.value {
                self.adapter.pause()?;
            } else {
                self.adapter.resume()?;
            }
            self.original_paused = self.paused_toggle.value;
        }

        self.dirty = false;
        Ok(())
    }

    fn update_status(&mut self, _status: serde_json::Value) {
        self.refresh_status();
    }

    fn set_instant_apply(&mut self, enabled: bool) {
        self.instant_apply = enabled;
    }

    fn instant_apply_enabled(&self) -> bool {
        self.instant_apply
    }

    fn apply_instant(&mut self) -> Result<()> {
        if !self.adapter.is_connected() {
            self.adapter.connect()?;
        }

        if let Some(field) = self.pending_change.take() {
            match field {
                "paused" => {
                    if self.paused_toggle.value != self.original_paused {
                        if self.paused_toggle.value {
                            self.adapter.pause()?;
                        } else {
                            self.adapter.resume()?;
                        }
                        self.original_paused = self.paused_toggle.value;
                    }
                }
                _ => {}
            }
            self.check_dirty();
        }

        Ok(())
    }
}
