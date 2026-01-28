//! Configuration panel for garlock (screen locker)

use crate::config::GarlockConfig;
use crate::ipc::adapters::GarlockAdapter;
use crate::ui::widgets::{Button, Label, NumberInput, Section, WidgetEvent};
use crate::ui::{Panel, PanelAction};
use anyhow::Result;
use gartk_core::{InputEvent, Rect, Theme};
use gartk_render::{Renderer, TextStyle};

/// Garlock configuration panel
pub struct GarlockPanel {
    adapter: GarlockAdapter,

    // Sections
    status_section: Section,
    settings_section: Section,
    actions_section: Section,

    // Status display
    locked_label: Label,

    // Settings
    idle_timeout: NumberInput,

    // Action buttons
    lock_button: Button,
    refresh_button: Button,
    save_button: Button,

    // State
    original_timeout: i32,
    dirty: bool,
    scroll_offset: i32,
}

impl GarlockPanel {
    pub fn new(mut adapter: GarlockAdapter) -> Self {
        let (locked, timeout): (String, i32) = if adapter.is_connected() {
            if let Ok(status) = adapter.status() {
                let locked_text = if status.locked { "Locked" } else { "Unlocked" };
                let timeout = status.idle_timeout_secs.unwrap_or(300) as i32;
                (locked_text.to_string(), timeout)
            } else {
                ("Unknown".to_string(), 300)
            }
        } else {
            ("Not connected".to_string(), 300)
        };

        Self {
            adapter,
            status_section: Section::new("Status"),
            settings_section: Section::new("Settings"),
            actions_section: Section::new("Actions"),
            locked_label: Label::new("Lock Status", &locked),
            idle_timeout: NumberInput::new("Idle Timeout (seconds)", timeout)
                .with_range(30, 3600)
                .with_step(30),
            lock_button: Button::new("Lock Now").primary(),
            refresh_button: Button::new("Refresh"),
            save_button: Button::new("Save"),
            original_timeout: timeout,
            dirty: false,
            scroll_offset: 0,
        }
    }

    fn refresh_status(&mut self) {
        if !self.adapter.is_connected() {
            let _ = self.adapter.connect();
        }

        if let Ok(status) = self.adapter.status() {
            let locked_text = if status.locked { "Locked" } else { "Unlocked" };
            self.locked_label.text = locked_text.into();

            if let Some(timeout) = status.idle_timeout_secs {
                self.idle_timeout.value = timeout as i32;
                self.original_timeout = timeout as i32;
            }
            self.dirty = false;
        }
    }

    fn check_dirty(&mut self) {
        self.dirty = self.idle_timeout.value != self.original_timeout;
    }

    fn layout_widgets(&mut self, bounds: Rect, theme: &Theme) {
        let padding = theme.padding as i32;
        let row_height = 36;
        let section_height = 32;
        let widget_width = bounds.width - (padding * 2) as u32;

        let mut y = bounds.y + padding - self.scroll_offset;

        // Status section
        self.status_section.bounds =
            Rect::new(bounds.x + padding, y, widget_width, section_height as u32);
        y += section_height;

        if self.status_section.expanded {
            self.locked_label.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;
        }

        y += padding / 2;

        // Settings section
        self.settings_section.bounds =
            Rect::new(bounds.x + padding, y, widget_width, section_height as u32);
        y += section_height;

        if self.settings_section.expanded {
            self.idle_timeout.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;
        }

        y += padding / 2;

        // Actions section
        self.actions_section.bounds =
            Rect::new(bounds.x + padding, y, widget_width, section_height as u32);
        y += section_height;

        if self.actions_section.expanded {
            self.lock_button.bounds = Rect::new(
                bounds.x + padding + 16,
                y,
                100,
                32,
            );

            self.refresh_button.bounds = Rect::new(
                bounds.x + padding + 16 + 108,
                y,
                100,
                32,
            );

            self.save_button.bounds = Rect::new(
                bounds.x + padding + 16 + 216,
                y,
                80,
                32,
            );
        }
    }
}

impl Panel for GarlockPanel {
    fn name(&self) -> &str {
        "garlock"
    }

    fn description(&self) -> &str {
        "Screen locker"
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn render(&mut self, renderer: &mut Renderer, bounds: Rect, theme: &Theme) -> Result<()> {
        self.layout_widgets(bounds, theme);

        // Render sections
        self.status_section.render(renderer, theme)?;
        if self.status_section.expanded {
            self.locked_label.render(renderer, theme)?;
        }

        self.settings_section.render(renderer, theme)?;
        if self.settings_section.expanded {
            self.idle_timeout.render(renderer, theme)?;
        }

        self.actions_section.render(renderer, theme)?;
        if self.actions_section.expanded {
            self.lock_button.render(renderer, theme)?;
            self.refresh_button.render(renderer, theme)?;
            self.save_button.render(renderer, theme)?;
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
                if let WidgetEvent::Changed = self.settings_section.on_click(x, y) {
                    return PanelAction::Redraw;
                }
                if let WidgetEvent::Changed = self.actions_section.on_click(x, y) {
                    return PanelAction::Redraw;
                }

                // Settings
                if let WidgetEvent::Changed = self.idle_timeout.on_click(x, y) {
                    self.check_dirty();
                    return PanelAction::Redraw;
                }

                // Action buttons
                if let WidgetEvent::Clicked = self.lock_button.on_click(x, y) {
                    if self.adapter.is_connected() {
                        let _ = self.adapter.lock();
                    }
                    return PanelAction::Redraw;
                }
                if let WidgetEvent::Clicked = self.refresh_button.on_click(x, y) {
                    self.refresh_status();
                    return PanelAction::Redraw;
                }
                if let WidgetEvent::Clicked = self.save_button.on_click(x, y) {
                    return PanelAction::Save;
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
        changed |= self.settings_section.on_mouse_move(x, y);
        changed |= self.actions_section.on_mouse_move(x, y);
        changed |= self.idle_timeout.on_mouse_move(x, y);
        changed |= self.lock_button.on_mouse_move(x, y);
        changed |= self.refresh_button.on_mouse_move(x, y);
        changed |= self.save_button.on_mouse_move(x, y);
        changed
    }

    fn reset(&mut self) {
        self.idle_timeout.value = self.original_timeout;
        self.dirty = false;
    }

    fn apply(&mut self) -> Result<()> {
        // Note: Timeout changes would need config file edit
        // For now, just update original values
        self.original_timeout = self.idle_timeout.value;
        self.dirty = false;
        Ok(())
    }

    fn update_status(&mut self, _status: serde_json::Value) {
        self.refresh_status();
    }

    fn has_config_file(&self) -> bool {
        true
    }

    fn save_to_config(&mut self) -> Result<()> {
        let mut config = GarlockConfig::load().unwrap_or_default();

        // Update idle timeout
        config.idle_timeout_secs = Some(self.idle_timeout.value as u64);

        // Save config
        config.save()?;

        // Trigger reload (garlock reads on next lock)
        crate::config::reload_component("garlock")?;

        Ok(())
    }
}
