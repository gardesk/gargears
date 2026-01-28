//! Configuration panel for garterm (terminal emulator)

use crate::ipc::adapters::GartermAdapter;
use crate::ui::widgets::{Button, Label, Section, WidgetEvent};
use crate::ui::{Panel, PanelAction};
use anyhow::Result;
use gartk_core::{InputEvent, Rect, Theme};
use gartk_render::{Renderer, TextStyle};

/// Garterm configuration panel
pub struct GartermPanel {
    adapter: GartermAdapter,

    // Sections
    status_section: Section,
    actions_section: Section,

    // Status display
    pid_label: Label,
    title_label: Label,
    tabs_label: Label,
    instances_label: Label,

    // Action buttons
    new_window_button: Button,
    new_tab_button: Button,
    refresh_button: Button,

    // State
    scroll_offset: i32,
}

impl GartermPanel {
    pub fn new(mut adapter: GartermAdapter) -> Self {
        let (pid, title, tabs) = if adapter.is_connected() {
            if let Ok(status) = adapter.status() {
                (
                    status.pid.map(|p| format!("PID: {}", p)).unwrap_or_else(|| "PID: -".into()),
                    status.title.unwrap_or_else(|| "No title".into()),
                    status.tabs.map(|t| format!("{} tab(s)", t)).unwrap_or_else(|| "- tabs".into()),
                )
            } else {
                ("PID: -".into(), "Not connected".into(), "- tabs".into())
            }
        } else {
            ("PID: -".into(), "Not running".into(), "- tabs".into())
        };

        let instances = GartermAdapter::list_instances();
        let instances_text = if instances.is_empty() {
            "No instances running".into()
        } else {
            format!("{} instance(s): {}", instances.len(),
                instances.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", "))
        };

        Self {
            adapter,
            status_section: Section::new("Status"),
            actions_section: Section::new("Actions"),
            pid_label: Label::new("Process", &pid),
            title_label: Label::new("Title", &title),
            tabs_label: Label::new("Tabs", &tabs),
            instances_label: Label::new("Instances", &instances_text),
            new_window_button: Button::new("New Window").primary(),
            new_tab_button: Button::new("New Tab"),
            refresh_button: Button::new("Refresh"),
            scroll_offset: 0,
        }
    }

    fn refresh_status(&mut self) {
        // Reconnect to potentially new focused instance
        let _ = self.adapter.connect();

        let (pid, title, tabs) = if self.adapter.is_connected() {
            if let Ok(status) = self.adapter.status() {
                (
                    status.pid.map(|p| format!("PID: {}", p)).unwrap_or_else(|| "PID: -".into()),
                    status.title.unwrap_or_else(|| "No title".into()),
                    status.tabs.map(|t| format!("{} tab(s)", t)).unwrap_or_else(|| "- tabs".into()),
                )
            } else {
                ("PID: -".into(), "Error getting status".into(), "- tabs".into())
            }
        } else {
            ("PID: -".into(), "Not running".into(), "- tabs".into())
        };

        let instances = GartermAdapter::list_instances();
        let instances_text = if instances.is_empty() {
            "No instances running".into()
        } else {
            format!("{} instance(s): {}", instances.len(),
                instances.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", "))
        };

        self.pid_label.text = pid;
        self.title_label.text = title;
        self.tabs_label.text = tabs;
        self.instances_label.text = instances_text;
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
            self.pid_label.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;

            self.title_label.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;

            self.tabs_label.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;

            self.instances_label.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;
        }

        y += padding / 2;

        // Actions section
        self.actions_section.bounds =
            Rect::new(bounds.x + padding, y, widget_width, section_height as u32);
        y += section_height;

        if self.actions_section.expanded {
            let button_width = 100;
            let button_spacing = 8;

            self.new_window_button.bounds = Rect::new(
                bounds.x + padding + 16,
                y,
                button_width,
                32,
            );

            self.new_tab_button.bounds = Rect::new(
                bounds.x + padding + 16 + button_width as i32 + button_spacing,
                y,
                button_width,
                32,
            );

            self.refresh_button.bounds = Rect::new(
                bounds.x + padding + 16 + (button_width as i32 + button_spacing) * 2,
                y,
                button_width,
                32,
            );
        }
    }
}

impl Panel for GartermPanel {
    fn name(&self) -> &str {
        "garterm"
    }

    fn description(&self) -> &str {
        "Terminal emulator"
    }

    fn is_dirty(&self) -> bool {
        false // Status display only, no editable settings yet
    }

    fn render(&mut self, renderer: &mut Renderer, bounds: Rect, theme: &Theme) -> Result<()> {
        self.layout_widgets(bounds, theme);

        // Render sections
        self.status_section.render(renderer, theme)?;
        if self.status_section.expanded {
            self.pid_label.render(renderer, theme)?;
            self.title_label.render(renderer, theme)?;
            self.tabs_label.render(renderer, theme)?;
            self.instances_label.render(renderer, theme)?;
        }

        self.actions_section.render(renderer, theme)?;
        if self.actions_section.expanded {
            self.new_window_button.render(renderer, theme)?;
            self.new_tab_button.render(renderer, theme)?;
            self.refresh_button.render(renderer, theme)?;
        }

        // Connection status at bottom
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
            "Connected to focused instance"
        } else {
            "No garterm instance focused"
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

                // Action buttons
                if let WidgetEvent::Clicked = self.new_window_button.on_click(x, y) {
                    if self.adapter.is_connected() {
                        let _ = self.adapter.new_window();
                    }
                    return PanelAction::Redraw;
                }
                if let WidgetEvent::Clicked = self.new_tab_button.on_click(x, y) {
                    if self.adapter.is_connected() {
                        let _ = self.adapter.new_tab();
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
        changed |= self.new_window_button.on_mouse_move(x, y);
        changed |= self.new_tab_button.on_mouse_move(x, y);
        changed |= self.refresh_button.on_mouse_move(x, y);
        changed
    }

    fn reset(&mut self) {
        // Nothing to reset
    }

    fn apply(&mut self) -> Result<()> {
        // No settings to apply
        Ok(())
    }

    fn update_status(&mut self, _status: serde_json::Value) {
        self.refresh_status();
    }
}
