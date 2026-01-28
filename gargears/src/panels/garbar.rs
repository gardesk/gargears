//! Configuration panel for garbar (status bar)

use crate::config::{GarbarSettings, LuaConfigWriter};
use crate::ipc::adapters::GarbarAdapter;
use crate::ui::widgets::{Button, NumberInput, Section, TextInput, Toggle, WidgetEvent};
use crate::ui::{Panel, PanelAction};
use anyhow::Result;
use gartk_core::{InputEvent, Key, Rect, Theme};
use gartk_render::{Renderer, TextStyle};

/// Garbar configuration panel
pub struct GarbarPanel {
    adapter: GarbarAdapter,

    // Sections
    general_section: Section,
    modules_left_section: Section,
    modules_center_section: Section,
    modules_right_section: Section,

    // General settings
    height: NumberInput,
    position: Toggle, // top/bottom
    visible: Toggle,

    // Module lists (display only for now, editing is complex)
    modules_left_display: TextInput,
    modules_center_display: TextInput,
    modules_right_display: TextInput,

    // Action buttons
    apply_button: Button,
    reset_button: Button,
    reload_button: Button,
    save_button: Button,

    // State
    original_values: GarbarValues,
    dirty: bool,
    scroll_offset: i32,
    instant_apply: bool,
    pending_change: Option<&'static str>,
}

#[derive(Clone, Default)]
struct GarbarValues {
    height: i32,
    position_top: bool,
    visible: bool,
    modules_left: String,
    modules_center: String,
    modules_right: String,
}

impl GarbarPanel {
    pub fn new(mut adapter: GarbarAdapter) -> Self {
        let values = if adapter.is_connected() {
            Self::fetch_values(&mut adapter)
        } else {
            GarbarValues::default()
        };

        Self {
            adapter,
            general_section: Section::new("General"),
            modules_left_section: Section::new("Modules Left"),
            modules_center_section: Section::new("Modules Center"),
            modules_right_section: Section::new("Modules Right"),
            height: NumberInput::new("Height", values.height)
                .with_range(16, 64)
                .with_step(2),
            position: Toggle::new("Position Top", values.position_top),
            visible: Toggle::new("Visible", values.visible),
            modules_left_display: TextInput::new("Modules", &values.modules_left)
                .with_placeholder("workspaces, title"),
            modules_center_display: TextInput::new("Modules", &values.modules_center)
                .with_placeholder("clock"),
            modules_right_display: TextInput::new("Modules", &values.modules_right)
                .with_placeholder("cpu, memory, battery"),
            apply_button: Button::new("Apply").primary(),
            reset_button: Button::new("Reset"),
            reload_button: Button::new("Reload"),
            save_button: Button::new("Save"),
            original_values: values,
            dirty: false,
            scroll_offset: 0,
            instant_apply: true,
            pending_change: None,
        }
    }

    fn fetch_values(adapter: &mut GarbarAdapter) -> GarbarValues {
        if let Ok(status) = adapter.status() {
            let position_top = status
                .position
                .as_ref()
                .map(|p| p == "top")
                .unwrap_or(true);
            GarbarValues {
                height: status.height as i32,
                position_top,
                visible: status.visible,
                modules_left: status.modules_left.join(", "),
                modules_center: status.modules_center.join(", "),
                modules_right: status.modules_right.join(", "),
            }
        } else {
            GarbarValues {
                height: 28,
                position_top: true,
                visible: true,
                modules_left: "workspaces".into(),
                modules_center: String::new(),
                modules_right: "clock".into(),
            }
        }
    }

    fn check_dirty(&mut self) {
        self.dirty = self.height.value != self.original_values.height
            || self.position.value != self.original_values.position_top
            || self.visible.value != self.original_values.visible
            || self.modules_left_display.value != self.original_values.modules_left
            || self.modules_center_display.value != self.original_values.modules_center
            || self.modules_right_display.value != self.original_values.modules_right;
    }

    fn layout_widgets(&mut self, bounds: Rect, theme: &Theme) {
        let padding = theme.padding as i32;
        let row_height = 36;
        let section_height = 32;
        let widget_width = bounds.width - (padding * 2) as u32;

        let mut y = bounds.y + padding - self.scroll_offset;

        // General section
        self.general_section.bounds =
            Rect::new(bounds.x + padding, y, widget_width, section_height as u32);
        y += section_height;

        if self.general_section.expanded {
            self.height.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;

            self.position.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;

            self.visible.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;
        }

        y += padding / 2;

        // Modules Left section
        self.modules_left_section.bounds =
            Rect::new(bounds.x + padding, y, widget_width, section_height as u32);
        y += section_height;

        if self.modules_left_section.expanded {
            self.modules_left_display.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;
        }

        y += padding / 2;

        // Modules Center section
        self.modules_center_section.bounds =
            Rect::new(bounds.x + padding, y, widget_width, section_height as u32);
        y += section_height;

        if self.modules_center_section.expanded {
            self.modules_center_display.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;
        }

        y += padding / 2;

        // Modules Right section
        self.modules_right_section.bounds =
            Rect::new(bounds.x + padding, y, widget_width, section_height as u32);
        y += section_height;

        if self.modules_right_section.expanded {
            self.modules_right_display.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
        }

        // Action buttons at bottom
        let button_y = bounds.y + bounds.height as i32 - padding - 36;
        let button_width = 68;

        self.reload_button.bounds = Rect::new(
            bounds.x + bounds.width as i32 - padding - button_width * 4 - 24,
            button_y,
            button_width as u32,
            32,
        );

        self.reset_button.bounds = Rect::new(
            bounds.x + bounds.width as i32 - padding - button_width * 3 - 16,
            button_y,
            button_width as u32,
            32,
        );

        self.apply_button.bounds = Rect::new(
            bounds.x + bounds.width as i32 - padding - button_width * 2 - 8,
            button_y,
            button_width as u32,
            32,
        );

        self.save_button.bounds = Rect::new(
            bounds.x + bounds.width as i32 - padding - button_width,
            button_y,
            button_width as u32,
            32,
        );
    }
}

impl Panel for GarbarPanel {
    fn name(&self) -> &str {
        "garbar"
    }

    fn description(&self) -> &str {
        "Status bar"
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn render(&mut self, renderer: &mut Renderer, bounds: Rect, theme: &Theme) -> Result<()> {
        self.layout_widgets(bounds, theme);

        // Render sections and widgets
        self.general_section.render(renderer, theme)?;
        if self.general_section.expanded {
            self.height.render(renderer, theme)?;
            self.position.render(renderer, theme)?;
            self.visible.render(renderer, theme)?;
        }

        self.modules_left_section.render(renderer, theme)?;
        if self.modules_left_section.expanded {
            self.modules_left_display.render(renderer, theme)?;
        }

        self.modules_center_section.render(renderer, theme)?;
        if self.modules_center_section.expanded {
            self.modules_center_display.render(renderer, theme)?;
        }

        self.modules_right_section.render(renderer, theme)?;
        if self.modules_right_section.expanded {
            self.modules_right_display.render(renderer, theme)?;
        }

        // Render action buttons
        self.reload_button.render(renderer, theme)?;
        self.reset_button.render(renderer, theme)?;
        self.apply_button.render(renderer, theme)?;
        self.save_button.render(renderer, theme)?;

        // Dirty indicator
        if self.dirty {
            let indicator_style = TextStyle::new()
                .font_family(&theme.font_family)
                .font_size(theme.font_size * 0.75)
                .color(gartk_core::Color::from_u8(0xff, 0xb8, 0x6c, 0xff));

            renderer.text(
                "● Unsaved changes",
                (bounds.x + theme.padding as i32) as f64,
                (bounds.y + bounds.height as i32 - theme.padding as i32 - 28) as f64,
                &indicator_style,
            )?;
        }

        Ok(())
    }

    fn handle_event(&mut self, event: &InputEvent) -> PanelAction {
        match event {
            InputEvent::Key(ke) if ke.pressed => {
                // Handle text input keys
                if let WidgetEvent::Changed = self.modules_left_display.on_key(&ke.key) {
                    self.check_dirty();
                    return PanelAction::Redraw;
                }
                if let WidgetEvent::Changed = self.modules_center_display.on_key(&ke.key) {
                    self.check_dirty();
                    return PanelAction::Redraw;
                }
                if let WidgetEvent::Changed = self.modules_right_display.on_key(&ke.key) {
                    self.check_dirty();
                    return PanelAction::Redraw;
                }

                // Scroll with Page Up/Down
                match &ke.key {
                    Key::PageUp => {
                        self.scroll_offset = (self.scroll_offset - 100).max(0);
                        return PanelAction::Redraw;
                    }
                    Key::PageDown => {
                        self.scroll_offset += 100;
                        return PanelAction::Redraw;
                    }
                    _ => {}
                }

                PanelAction::None
            }
            InputEvent::MousePress(me) => {
                let x = me.position.x;
                let y = me.position.y;

                // Check sections
                if let WidgetEvent::Changed = self.general_section.on_click(x, y) {
                    return PanelAction::Redraw;
                }
                if let WidgetEvent::Changed = self.modules_left_section.on_click(x, y) {
                    return PanelAction::Redraw;
                }
                if let WidgetEvent::Changed = self.modules_center_section.on_click(x, y) {
                    return PanelAction::Redraw;
                }
                if let WidgetEvent::Changed = self.modules_right_section.on_click(x, y) {
                    return PanelAction::Redraw;
                }

                // Check widgets - with instant apply support
                if let WidgetEvent::Changed = self.height.on_click(x, y) {
                    self.check_dirty();
                    // Height change needs config save + reload, no instant apply
                    return PanelAction::Redraw;
                }
                if let WidgetEvent::Changed = self.position.on_click(x, y) {
                    self.check_dirty();
                    // Position change needs config save + reload, no instant apply
                    return PanelAction::Redraw;
                }
                if let WidgetEvent::Changed = self.visible.on_click(x, y) {
                    self.check_dirty();
                    self.pending_change = Some("visible");
                    return if self.instant_apply { PanelAction::InstantApply } else { PanelAction::Redraw };
                }

                // Text inputs
                self.modules_left_display.on_click(x, y);
                self.modules_center_display.on_click(x, y);
                self.modules_right_display.on_click(x, y);

                // Action buttons
                if let WidgetEvent::Clicked = self.apply_button.on_click(x, y) {
                    return PanelAction::Apply;
                }
                if let WidgetEvent::Clicked = self.reset_button.on_click(x, y) {
                    return PanelAction::Reset;
                }
                if let WidgetEvent::Clicked = self.reload_button.on_click(x, y) {
                    // Special: reload config
                    let _ = self.adapter.reload();
                    return PanelAction::Redraw;
                }
                if let WidgetEvent::Clicked = self.save_button.on_click(x, y) {
                    return PanelAction::Save;
                }

                PanelAction::Redraw
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
        changed |= self.general_section.on_mouse_move(x, y);
        changed |= self.modules_left_section.on_mouse_move(x, y);
        changed |= self.modules_center_section.on_mouse_move(x, y);
        changed |= self.modules_right_section.on_mouse_move(x, y);

        changed |= self.height.on_mouse_move(x, y);
        changed |= self.position.on_mouse_move(x, y);
        changed |= self.visible.on_mouse_move(x, y);
        changed |= self.modules_left_display.on_mouse_move(x, y);
        changed |= self.modules_center_display.on_mouse_move(x, y);
        changed |= self.modules_right_display.on_mouse_move(x, y);

        changed |= self.apply_button.on_mouse_move(x, y);
        changed |= self.reset_button.on_mouse_move(x, y);
        changed |= self.reload_button.on_mouse_move(x, y);
        changed |= self.save_button.on_mouse_move(x, y);
        changed
    }

    fn reset(&mut self) {
        self.height.value = self.original_values.height;
        self.position.value = self.original_values.position_top;
        self.visible.value = self.original_values.visible;
        self.modules_left_display.value = self.original_values.modules_left.clone();
        self.modules_center_display.value = self.original_values.modules_center.clone();
        self.modules_right_display.value = self.original_values.modules_right.clone();
        self.dirty = false;
    }

    fn apply(&mut self) -> Result<()> {
        if !self.adapter.is_connected() {
            self.adapter.connect()?;
        }

        // Toggle visibility if changed
        if self.visible.value != self.original_values.visible {
            if self.visible.value {
                self.adapter.show()?;
            } else {
                self.adapter.hide()?;
            }
        }

        // Note: height and module changes require config file edit + reload
        // For now we just reload to pick up any manual config changes
        if self.height.value != self.original_values.height {
            // Would need to edit Lua config and reload
            self.adapter.reload()?;
        }

        // Update original values
        self.original_values = GarbarValues {
            height: self.height.value,
            position_top: self.position.value,
            visible: self.visible.value,
            modules_left: self.modules_left_display.value.clone(),
            modules_center: self.modules_center_display.value.clone(),
            modules_right: self.modules_right_display.value.clone(),
        };

        self.dirty = false;
        Ok(())
    }

    fn update_status(&mut self, _status: serde_json::Value) {
        // Could refresh values from status
    }

    fn has_config_file(&self) -> bool {
        true
    }

    fn save_to_config(&mut self) -> Result<()> {
        let mut writer = LuaConfigWriter::load()?;

        // Parse module lists from comma-separated strings
        let parse_modules = |s: &str| -> Vec<String> {
            s.split(',')
                .map(|m| m.trim().to_string())
                .filter(|m| !m.is_empty())
                .collect()
        };

        let settings = GarbarSettings {
            height: Some(self.height.value as u32),
            position: Some(if self.position.value { "top" } else { "bottom" }.to_string()),
            modules_left: Some(parse_modules(&self.modules_left_display.value)),
            modules_center: Some(parse_modules(&self.modules_center_display.value)),
            modules_right: Some(parse_modules(&self.modules_right_display.value)),
            ..Default::default()
        };

        writer.set_bar_table(&settings);
        writer.save()?;

        // Trigger reload
        crate::config::reload_component("garbar")?;

        Ok(())
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

        // Apply only the pending change
        if let Some(field) = self.pending_change.take() {
            match field {
                "visible" => {
                    if self.visible.value != self.original_values.visible {
                        if self.visible.value {
                            self.adapter.show()?;
                        } else {
                            self.adapter.hide()?;
                        }
                        self.original_values.visible = self.visible.value;
                    }
                }
                _ => {}
            }
            self.check_dirty();
        }

        Ok(())
    }
}
