//! Configuration panel for gartray (system tray & quick settings)

use crate::ipc::adapters::GartrayAdapter;
use crate::ui::widgets::{Button, Label, Section, Toggle, WidgetEvent};
use crate::ui::{Panel, PanelAction};
use anyhow::Result;
use gartk_core::{InputEvent, Rect, Theme};
use gartk_render::{Renderer, TextStyle};

/// Gartray configuration panel
pub struct GartrayPanel {
    adapter: GartrayAdapter,

    // Sections
    status_section: Section,
    actions_section: Section,

    // Status display
    visible_toggle: Toggle,
    tray_icons_label: Label,

    // Action buttons
    show_button: Button,
    hide_button: Button,
    toggle_button: Button,
    refresh_button: Button,

    // State
    original_visible: bool,
    dirty: bool,
    scroll_offset: i32,
    instant_apply: bool,
    pending_change: Option<&'static str>,
}

impl GartrayPanel {
    pub fn new(mut adapter: GartrayAdapter) -> Self {
        let (visible, tray_icons) = if adapter.is_connected() {
            if let Ok(status) = adapter.status() {
                (status.visible, format!("{} tray icon(s)", status.tray_icons))
            } else {
                (false, "Unknown".into())
            }
        } else {
            (false, "Not connected".into())
        };

        Self {
            adapter,
            status_section: Section::new("Status"),
            actions_section: Section::new("Actions"),
            visible_toggle: Toggle::new("Visible", visible),
            tray_icons_label: Label::new("Tray Icons", &tray_icons),
            show_button: Button::new("Show"),
            hide_button: Button::new("Hide"),
            toggle_button: Button::new("Toggle").primary(),
            refresh_button: Button::new("Refresh"),
            original_visible: visible,
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
            self.visible_toggle.value = status.visible;
            self.original_visible = status.visible;
            self.tray_icons_label.text = format!("{} tray icon(s)", status.tray_icons);
            self.dirty = false;
        }
    }

    fn check_dirty(&mut self) {
        self.dirty = self.visible_toggle.value != self.original_visible;
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
            self.visible_toggle.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;

            self.tray_icons_label.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;
        }

        y += padding / 2;

        // Actions section
        self.actions_section.bounds =
            Rect::new(bounds.x + padding, y, widget_width, section_height as u32);
        y += section_height;

        if self.actions_section.expanded {
            let button_width = 80;
            let button_spacing = 8;

            self.show_button.bounds = Rect::new(
                bounds.x + padding + 16,
                y,
                button_width,
                32,
            );

            self.hide_button.bounds = Rect::new(
                bounds.x + padding + 16 + button_width as i32 + button_spacing,
                y,
                button_width,
                32,
            );

            self.toggle_button.bounds = Rect::new(
                bounds.x + padding + 16 + (button_width as i32 + button_spacing) * 2,
                y,
                button_width,
                32,
            );

            self.refresh_button.bounds = Rect::new(
                bounds.x + padding + 16 + (button_width as i32 + button_spacing) * 3,
                y,
                button_width,
                32,
            );
        }
    }
}

impl Panel for GartrayPanel {
    fn name(&self) -> &str {
        "gartray"
    }

    fn description(&self) -> &str {
        "System tray & quick settings"
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn render(&mut self, renderer: &mut Renderer, bounds: Rect, theme: &Theme) -> Result<()> {
        self.layout_widgets(bounds, theme);

        // Render sections
        self.status_section.render(renderer, theme)?;
        if self.status_section.expanded {
            self.visible_toggle.render(renderer, theme)?;
            self.tray_icons_label.render(renderer, theme)?;
        }

        self.actions_section.render(renderer, theme)?;
        if self.actions_section.expanded {
            self.show_button.render(renderer, theme)?;
            self.hide_button.render(renderer, theme)?;
            self.toggle_button.render(renderer, theme)?;
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
                if let WidgetEvent::Changed = self.visible_toggle.on_click(x, y) {
                    self.check_dirty();
                    self.pending_change = Some("visible");
                    return if self.instant_apply { PanelAction::InstantApply } else { PanelAction::Redraw };
                }

                // Action buttons
                if let WidgetEvent::Clicked = self.show_button.on_click(x, y) {
                    if self.adapter.is_connected() {
                        let _ = self.adapter.show();
                        self.refresh_status();
                    }
                    return PanelAction::Redraw;
                }
                if let WidgetEvent::Clicked = self.hide_button.on_click(x, y) {
                    if self.adapter.is_connected() {
                        let _ = self.adapter.hide();
                        self.refresh_status();
                    }
                    return PanelAction::Redraw;
                }
                if let WidgetEvent::Clicked = self.toggle_button.on_click(x, y) {
                    if self.adapter.is_connected() {
                        let _ = self.adapter.toggle();
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
        changed |= self.visible_toggle.on_mouse_move(x, y);
        changed |= self.show_button.on_mouse_move(x, y);
        changed |= self.hide_button.on_mouse_move(x, y);
        changed |= self.toggle_button.on_mouse_move(x, y);
        changed |= self.refresh_button.on_mouse_move(x, y);
        changed
    }

    fn reset(&mut self) {
        self.visible_toggle.value = self.original_visible;
        self.dirty = false;
    }

    fn apply(&mut self) -> Result<()> {
        if !self.adapter.is_connected() {
            self.adapter.connect()?;
        }

        if self.visible_toggle.value != self.original_visible {
            if self.visible_toggle.value {
                self.adapter.show()?;
            } else {
                self.adapter.hide()?;
            }
            self.original_visible = self.visible_toggle.value;
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
                "visible" => {
                    if self.visible_toggle.value != self.original_visible {
                        if self.visible_toggle.value {
                            self.adapter.show()?;
                        } else {
                            self.adapter.hide()?;
                        }
                        self.original_visible = self.visible_toggle.value;
                    }
                }
                _ => {}
            }
            self.check_dirty();
        }

        Ok(())
    }
}
