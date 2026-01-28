//! Configuration panel for garfield (file explorer)

use crate::ipc::adapters::GarfieldAdapter;
use crate::ui::widgets::{Button, Label, Section, TextInput, WidgetEvent};
use crate::ui::{Panel, PanelAction};
use anyhow::Result;
use gartk_core::{InputEvent, Key, Rect, Theme};
use gartk_render::{Renderer, TextStyle};

/// Garfield configuration panel
pub struct GarfieldPanel {
    adapter: GarfieldAdapter,

    // Sections
    status_section: Section,
    navigation_section: Section,

    // Status display
    running_label: Label,
    current_dir_label: Label,

    // Navigation
    open_path: TextInput,

    // Action buttons
    open_button: Button,
    refresh_button: Button,

    // State
    scroll_offset: i32,
}

impl GarfieldPanel {
    pub fn new(mut adapter: GarfieldAdapter) -> Self {
        let (running, current_dir): (String, String) = if adapter.is_connected() {
            if let Ok(status) = adapter.status() {
                let running = if status.running { "Running" } else { "Not running" };
                let dir = status.current_dir.unwrap_or_else(|| "Unknown".to_string());
                (running.to_string(), dir)
            } else {
                ("Unknown".to_string(), "Unknown".to_string())
            }
        } else {
            ("Not connected".to_string(), "N/A".to_string())
        };

        Self {
            adapter,
            status_section: Section::new("Status"),
            navigation_section: Section::new("Navigation"),
            running_label: Label::new("Status", &running),
            current_dir_label: Label::new("Current Directory", &current_dir),
            open_path: TextInput::new("Path", "").with_placeholder("/home/user"),
            open_button: Button::new("Open").primary(),
            refresh_button: Button::new("Refresh"),
            scroll_offset: 0,
        }
    }

    fn refresh_status(&mut self) {
        if !self.adapter.is_connected() {
            let _ = self.adapter.connect();
        }

        if let Ok(status) = self.adapter.status() {
            let running = if status.running { "Running" } else { "Not running" };
            self.running_label.text = running.into();
            self.current_dir_label.text = status.current_dir.unwrap_or_else(|| "Unknown".into());
        }
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
            self.running_label.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;

            self.current_dir_label.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;
        }

        y += padding / 2;

        // Navigation section
        self.navigation_section.bounds =
            Rect::new(bounds.x + padding, y, widget_width, section_height as u32);
        y += section_height;

        if self.navigation_section.expanded {
            self.open_path.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height + 8;

            self.open_button.bounds = Rect::new(
                bounds.x + padding + 16,
                y,
                80,
                32,
            );

            self.refresh_button.bounds = Rect::new(
                bounds.x + padding + 16 + 88,
                y,
                80,
                32,
            );
        }
    }
}

impl Panel for GarfieldPanel {
    fn name(&self) -> &str {
        "garfield"
    }

    fn description(&self) -> &str {
        "File explorer"
    }

    fn is_dirty(&self) -> bool {
        false
    }

    fn render(&mut self, renderer: &mut Renderer, bounds: Rect, theme: &Theme) -> Result<()> {
        self.layout_widgets(bounds, theme);

        // Render sections
        self.status_section.render(renderer, theme)?;
        if self.status_section.expanded {
            self.running_label.render(renderer, theme)?;
            self.current_dir_label.render(renderer, theme)?;
        }

        self.navigation_section.render(renderer, theme)?;
        if self.navigation_section.expanded {
            self.open_path.render(renderer, theme)?;
            self.open_button.render(renderer, theme)?;
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

        Ok(())
    }

    fn handle_event(&mut self, event: &InputEvent) -> PanelAction {
        match event {
            InputEvent::Key(ke) if ke.pressed => {
                // Handle text input
                if let WidgetEvent::Changed = self.open_path.on_key(&ke.key) {
                    return PanelAction::Redraw;
                }

                // Enter to open path
                if matches!(ke.key, Key::Return) && !self.open_path.value.is_empty() {
                    if self.adapter.is_connected() {
                        let _ = self.adapter.open(&self.open_path.value);
                        self.refresh_status();
                    }
                    return PanelAction::Redraw;
                }

                PanelAction::None
            }
            InputEvent::MousePress(me) => {
                let x = me.position.x;
                let y = me.position.y;

                // Check sections
                if let WidgetEvent::Changed = self.status_section.on_click(x, y) {
                    return PanelAction::Redraw;
                }
                if let WidgetEvent::Changed = self.navigation_section.on_click(x, y) {
                    return PanelAction::Redraw;
                }

                // Action buttons
                if let WidgetEvent::Clicked = self.open_button.on_click(x, y) {
                    if self.adapter.is_connected() && !self.open_path.value.is_empty() {
                        let _ = self.adapter.open(&self.open_path.value);
                        self.refresh_status();
                    }
                    return PanelAction::Redraw;
                }
                if let WidgetEvent::Clicked = self.refresh_button.on_click(x, y) {
                    self.refresh_status();
                    return PanelAction::Redraw;
                }

                // Text input - check if focus changed
                if let WidgetEvent::Changed = self.open_path.on_click(x, y) {
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
        changed |= self.navigation_section.on_mouse_move(x, y);
        changed |= self.open_path.on_mouse_move(x, y);
        changed |= self.open_button.on_mouse_move(x, y);
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
