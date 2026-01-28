//! Configuration panel for gar (window manager)

use crate::config::{GarSettings, LuaConfigWriter, LuaValue};
use crate::ipc::adapters::{GarAdapter, GarKeybind, GarRule};
use crate::ui::widgets::{Button, ColorPicker, List, NumberInput, Section, Toggle, WidgetEvent};
use crate::ui::{Panel, PanelAction};
use anyhow::Result;
use gartk_core::{InputEvent, Key, Rect, Theme};
use gartk_render::{Renderer, TextStyle};

/// Gar configuration panel
pub struct GarPanel {
    adapter: GarAdapter,

    // Sections
    borders_section: Section,
    gaps_section: Section,
    titlebar_section: Section,
    behavior_section: Section,

    // Border settings
    border_width: NumberInput,
    border_color_focused: ColorPicker,
    border_color_unfocused: ColorPicker,

    // Gap settings
    gap_inner: NumberInput,
    gap_outer: NumberInput,

    // Titlebar settings
    titlebar_enabled: Toggle,
    titlebar_height: NumberInput,

    // Behavior settings
    focus_follows_mouse: Toggle,
    mouse_follows_focus: Toggle,

    // Keybinds section (read-only)
    keybinds_section: Section,
    keybinds_list: List,

    // Window rules section (read-only)
    rules_section: Section,
    rules_list: List,

    // Action buttons
    apply_button: Button,
    reset_button: Button,
    save_button: Button,

    // State
    original_values: GarValues,
    dirty: bool,
    scroll_offset: i32,
    instant_apply: bool,
    pending_change: Option<&'static str>,
}

#[derive(Clone, Default)]
struct GarValues {
    border_width: i32,
    border_color_focused: String,
    border_color_unfocused: String,
    gap_inner: i32,
    gap_outer: i32,
    titlebar_enabled: bool,
    titlebar_height: i32,
    focus_follows_mouse: bool,
    mouse_follows_focus: bool,
}

impl GarPanel {
    pub fn new(mut adapter: GarAdapter) -> Self {
        // Try to get current values
        let values = if adapter.is_connected() {
            Self::fetch_values(&mut adapter)
        } else {
            GarValues::default()
        };

        // Fetch keybinds and rules
        let keybinds = if adapter.is_connected() {
            adapter.query_keybinds().unwrap_or_default()
        } else {
            vec![]
        };
        let rules = if adapter.is_connected() {
            adapter.query_rules().unwrap_or_default()
        } else {
            vec![]
        };

        let keybind_items: Vec<String> = keybinds.iter().map(|k| k.display()).collect();
        let rule_items: Vec<String> = rules.iter().map(|r| r.display()).collect();

        Self {
            adapter,
            borders_section: Section::new("Borders"),
            gaps_section: Section::new("Gaps"),
            titlebar_section: Section::new("Titlebar"),
            behavior_section: Section::new("Behavior"),
            border_width: NumberInput::new("Border Width", values.border_width)
                .with_range(0, 10)
                .with_step(1),
            border_color_focused: ColorPicker::new("Focused Color", &values.border_color_focused),
            border_color_unfocused: ColorPicker::new("Unfocused Color", &values.border_color_unfocused),
            gap_inner: NumberInput::new("Inner Gap", values.gap_inner)
                .with_range(0, 50)
                .with_step(2),
            gap_outer: NumberInput::new("Outer Gap", values.gap_outer)
                .with_range(0, 50)
                .with_step(2),
            titlebar_enabled: Toggle::new("Enabled", values.titlebar_enabled),
            titlebar_height: NumberInput::new("Height", values.titlebar_height)
                .with_range(0, 48)
                .with_step(2),
            focus_follows_mouse: Toggle::new("Focus Follows Mouse", values.focus_follows_mouse),
            mouse_follows_focus: Toggle::new("Mouse Follows Focus", values.mouse_follows_focus),
            keybinds_section: Section::new("Keybinds (read-only)"),
            keybinds_list: List::new().with_items(keybind_items).with_max_visible(4),
            rules_section: Section::new("Window Rules (read-only)"),
            rules_list: List::new().with_items(rule_items).with_max_visible(4),
            apply_button: Button::new("Apply").primary(),
            reset_button: Button::new("Reset"),
            save_button: Button::new("Save"),
            original_values: values,
            dirty: false,
            scroll_offset: 0,
            instant_apply: true,
            pending_change: None,
        }
    }

    fn fetch_values(adapter: &mut GarAdapter) -> GarValues {
        // Try to query config from gar
        if let Ok(config) = adapter.query_config() {
            GarValues {
                border_width: config.border_width.unwrap_or(2) as i32,
                border_color_focused: config.border_color_focused.unwrap_or_else(|| "#5294e2".into()),
                border_color_unfocused: config
                    .border_color_unfocused
                    .unwrap_or_else(|| "#3c3c3c".into()),
                gap_inner: config.gap_inner.unwrap_or(8) as i32,
                gap_outer: config.gap_outer.unwrap_or(8) as i32,
                titlebar_enabled: config.titlebar_enabled.unwrap_or(false),
                titlebar_height: config.titlebar_height.unwrap_or(24) as i32,
                focus_follows_mouse: config.focus_follows_mouse.unwrap_or(false),
                mouse_follows_focus: config.mouse_follows_focus.unwrap_or(false),
            }
        } else {
            GarValues {
                border_width: 2,
                border_color_focused: "#5294e2".into(),
                border_color_unfocused: "#3c3c3c".into(),
                gap_inner: 8,
                gap_outer: 8,
                titlebar_enabled: false,
                titlebar_height: 24,
                focus_follows_mouse: false,
                mouse_follows_focus: false,
            }
        }
    }

    fn check_dirty(&mut self) {
        self.dirty = self.border_width.value != self.original_values.border_width
            || self.border_color_focused.value != self.original_values.border_color_focused
            || self.border_color_unfocused.value != self.original_values.border_color_unfocused
            || self.gap_inner.value != self.original_values.gap_inner
            || self.gap_outer.value != self.original_values.gap_outer
            || self.titlebar_enabled.value != self.original_values.titlebar_enabled
            || self.titlebar_height.value != self.original_values.titlebar_height
            || self.focus_follows_mouse.value != self.original_values.focus_follows_mouse
            || self.mouse_follows_focus.value != self.original_values.mouse_follows_focus;
    }

    fn layout_widgets(&mut self, bounds: Rect, theme: &Theme) {
        let padding = theme.padding as i32;
        let row_height = 36;
        let section_height = 32;
        let widget_width = bounds.width - (padding * 2) as u32;

        let mut y = bounds.y + padding - self.scroll_offset;

        // Borders section
        self.borders_section.bounds = Rect::new(bounds.x + padding, y, widget_width, section_height as u32);
        y += section_height;

        if self.borders_section.expanded {
            self.border_width.bounds = Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;

            self.border_color_focused.bounds = Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;

            self.border_color_unfocused.bounds = Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;
        }

        y += padding / 2;

        // Gaps section
        self.gaps_section.bounds = Rect::new(bounds.x + padding, y, widget_width, section_height as u32);
        y += section_height;

        if self.gaps_section.expanded {
            self.gap_inner.bounds = Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;

            self.gap_outer.bounds = Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;
        }

        y += padding / 2;

        // Titlebar section
        self.titlebar_section.bounds = Rect::new(bounds.x + padding, y, widget_width, section_height as u32);
        y += section_height;

        if self.titlebar_section.expanded {
            self.titlebar_enabled.bounds = Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;

            self.titlebar_height.bounds = Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;
        }

        y += padding / 2;

        // Behavior section
        self.behavior_section.bounds = Rect::new(bounds.x + padding, y, widget_width, section_height as u32);
        y += section_height;

        if self.behavior_section.expanded {
            self.focus_follows_mouse.bounds = Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;

            self.mouse_follows_focus.bounds = Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;
        }

        y += padding / 2;

        // Keybinds section (read-only)
        self.keybinds_section.bounds = Rect::new(bounds.x + padding, y, widget_width, section_height as u32);
        y += section_height;

        if self.keybinds_section.expanded {
            let list_height = (row_height * 4).min(120) as u32;
            self.keybinds_list.bounds = Rect::new(bounds.x + padding + 16, y, widget_width - 16, list_height);
            y += list_height as i32 + 4;
        }

        y += padding / 2;

        // Rules section (read-only)
        self.rules_section.bounds = Rect::new(bounds.x + padding, y, widget_width, section_height as u32);
        y += section_height;

        if self.rules_section.expanded {
            let list_height = (row_height * 4).min(120) as u32;
            self.rules_list.bounds = Rect::new(bounds.x + padding + 16, y, widget_width - 16, list_height);
        }

        // Action buttons at bottom
        let button_y = bounds.y + bounds.height as i32 - padding - 36;
        let button_width = 80;

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

impl Panel for GarPanel {
    fn name(&self) -> &str {
        "gar"
    }

    fn description(&self) -> &str {
        "Tiling window manager"
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn render(&mut self, renderer: &mut Renderer, bounds: Rect, theme: &Theme) -> Result<()> {
        self.layout_widgets(bounds, theme);

        // Render sections and widgets
        self.borders_section.render(renderer, theme)?;
        if self.borders_section.expanded {
            self.border_width.render(renderer, theme)?;
            self.border_color_focused.render(renderer, theme)?;
            self.border_color_unfocused.render(renderer, theme)?;
        }

        self.gaps_section.render(renderer, theme)?;
        if self.gaps_section.expanded {
            self.gap_inner.render(renderer, theme)?;
            self.gap_outer.render(renderer, theme)?;
        }

        self.titlebar_section.render(renderer, theme)?;
        if self.titlebar_section.expanded {
            self.titlebar_enabled.render(renderer, theme)?;
            self.titlebar_height.render(renderer, theme)?;
        }

        self.behavior_section.render(renderer, theme)?;
        if self.behavior_section.expanded {
            self.focus_follows_mouse.render(renderer, theme)?;
            self.mouse_follows_focus.render(renderer, theme)?;
        }

        self.keybinds_section.render(renderer, theme)?;
        if self.keybinds_section.expanded {
            self.keybinds_list.render(renderer, theme)?;
        }

        self.rules_section.render(renderer, theme)?;
        if self.rules_section.expanded {
            self.rules_list.render(renderer, theme)?;
        }

        // Render action buttons
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
                // Handle text input keys - with instant apply support
                if let WidgetEvent::Changed = self.border_color_focused.on_key(&ke.key) {
                    self.check_dirty();
                    self.pending_change = Some("border_color_focused");
                    return if self.instant_apply { PanelAction::InstantApply } else { PanelAction::Redraw };
                }
                if let WidgetEvent::Changed = self.border_color_unfocused.on_key(&ke.key) {
                    self.check_dirty();
                    self.pending_change = Some("border_color_unfocused");
                    return if self.instant_apply { PanelAction::InstantApply } else { PanelAction::Redraw };
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
                if let WidgetEvent::Changed = self.borders_section.on_click(x, y) {
                    return PanelAction::Redraw;
                }
                if let WidgetEvent::Changed = self.gaps_section.on_click(x, y) {
                    return PanelAction::Redraw;
                }
                if let WidgetEvent::Changed = self.titlebar_section.on_click(x, y) {
                    return PanelAction::Redraw;
                }
                if let WidgetEvent::Changed = self.behavior_section.on_click(x, y) {
                    return PanelAction::Redraw;
                }
                if let WidgetEvent::Changed = self.keybinds_section.on_click(x, y) {
                    return PanelAction::Redraw;
                }
                if let WidgetEvent::Changed = self.rules_section.on_click(x, y) {
                    return PanelAction::Redraw;
                }

                // Check widgets - with instant apply support
                if let WidgetEvent::Changed = self.border_width.on_click(x, y) {
                    self.check_dirty();
                    self.pending_change = Some("border_width");
                    return if self.instant_apply { PanelAction::InstantApply } else { PanelAction::Redraw };
                }
                if let WidgetEvent::Changed = self.gap_inner.on_click(x, y) {
                    self.check_dirty();
                    self.pending_change = Some("gap_inner");
                    return if self.instant_apply { PanelAction::InstantApply } else { PanelAction::Redraw };
                }
                if let WidgetEvent::Changed = self.gap_outer.on_click(x, y) {
                    self.check_dirty();
                    self.pending_change = Some("gap_outer");
                    return if self.instant_apply { PanelAction::InstantApply } else { PanelAction::Redraw };
                }
                if let WidgetEvent::Changed = self.titlebar_enabled.on_click(x, y) {
                    self.check_dirty();
                    self.pending_change = Some("titlebar_enabled");
                    return if self.instant_apply { PanelAction::InstantApply } else { PanelAction::Redraw };
                }
                if let WidgetEvent::Changed = self.titlebar_height.on_click(x, y) {
                    self.check_dirty();
                    self.pending_change = Some("titlebar_height");
                    return if self.instant_apply { PanelAction::InstantApply } else { PanelAction::Redraw };
                }
                if let WidgetEvent::Changed = self.focus_follows_mouse.on_click(x, y) {
                    self.check_dirty();
                    self.pending_change = Some("focus_follows_mouse");
                    return if self.instant_apply { PanelAction::InstantApply } else { PanelAction::Redraw };
                }
                if let WidgetEvent::Changed = self.mouse_follows_focus.on_click(x, y) {
                    self.check_dirty();
                    self.pending_change = Some("mouse_follows_focus");
                    return if self.instant_apply { PanelAction::InstantApply } else { PanelAction::Redraw };
                }

                // Color inputs - clear other focus first, then check clicks
                // This ensures only one color picker is focused at a time
                let input1_bounds = self.border_color_focused.bounds;
                let input2_bounds = self.border_color_unfocused.bounds;
                let in_input1 = x >= input1_bounds.x && x < input1_bounds.x + input1_bounds.width as i32
                    && y >= input1_bounds.y && y < input1_bounds.y + input1_bounds.height as i32;
                let in_input2 = x >= input2_bounds.x && x < input2_bounds.x + input2_bounds.width as i32
                    && y >= input2_bounds.y && y < input2_bounds.y + input2_bounds.height as i32;

                // Clear focus from the other picker when clicking one
                if in_input1 {
                    self.border_color_unfocused.blur();
                } else if in_input2 {
                    self.border_color_focused.blur();
                } else {
                    // Clicking elsewhere - blur both
                    self.border_color_focused.blur();
                    self.border_color_unfocused.blur();
                }

                let focused1 = self.border_color_focused.on_click(x, y);
                let focused2 = self.border_color_unfocused.on_click(x, y);

                // Action buttons
                if let WidgetEvent::Clicked = self.apply_button.on_click(x, y) {
                    return PanelAction::Apply;
                }
                if let WidgetEvent::Clicked = self.reset_button.on_click(x, y) {
                    return PanelAction::Reset;
                }
                if let WidgetEvent::Clicked = self.save_button.on_click(x, y) {
                    return PanelAction::Save;
                }

                // Redraw if focus state changed (Focus or Blur events)
                if matches!(focused1, WidgetEvent::Focus | WidgetEvent::Blur)
                    || matches!(focused2, WidgetEvent::Focus | WidgetEvent::Blur)
                {
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

        changed |= self.borders_section.on_mouse_move(x, y);
        changed |= self.gaps_section.on_mouse_move(x, y);
        changed |= self.titlebar_section.on_mouse_move(x, y);
        changed |= self.behavior_section.on_mouse_move(x, y);
        changed |= self.keybinds_section.on_mouse_move(x, y);
        changed |= self.rules_section.on_mouse_move(x, y);

        changed |= self.border_width.on_mouse_move(x, y);
        changed |= self.border_color_focused.on_mouse_move(x, y);
        changed |= self.border_color_unfocused.on_mouse_move(x, y);
        changed |= self.gap_inner.on_mouse_move(x, y);
        changed |= self.gap_outer.on_mouse_move(x, y);
        changed |= self.titlebar_enabled.on_mouse_move(x, y);
        changed |= self.titlebar_height.on_mouse_move(x, y);
        changed |= self.focus_follows_mouse.on_mouse_move(x, y);
        changed |= self.mouse_follows_focus.on_mouse_move(x, y);

        changed |= self.apply_button.on_mouse_move(x, y);
        changed |= self.reset_button.on_mouse_move(x, y);
        changed |= self.save_button.on_mouse_move(x, y);

        changed
    }

    fn blur_focused(&mut self) {
        self.border_color_focused.blur();
        self.border_color_unfocused.blur();
    }

    fn reset(&mut self) {
        self.border_width.value = self.original_values.border_width;
        self.border_color_focused.value = self.original_values.border_color_focused.clone();
        self.border_color_unfocused.value = self.original_values.border_color_unfocused.clone();
        self.gap_inner.value = self.original_values.gap_inner;
        self.gap_outer.value = self.original_values.gap_outer;
        self.titlebar_enabled.value = self.original_values.titlebar_enabled;
        self.titlebar_height.value = self.original_values.titlebar_height;
        self.focus_follows_mouse.value = self.original_values.focus_follows_mouse;
        self.mouse_follows_focus.value = self.original_values.mouse_follows_focus;
        self.dirty = false;
    }

    fn apply(&mut self) -> Result<()> {
        if !self.adapter.is_connected() {
            self.adapter.connect()?;
        }

        // Send IPC commands for changed values
        if self.border_width.value != self.original_values.border_width {
            self.adapter.set_config_value(
                "border_width",
                serde_json::json!(self.border_width.value),
            )?;
        }

        if self.gap_inner.value != self.original_values.gap_inner {
            self.adapter
                .set_config_value("gap_inner", serde_json::json!(self.gap_inner.value))?;
        }

        if self.gap_outer.value != self.original_values.gap_outer {
            self.adapter
                .set_config_value("gap_outer", serde_json::json!(self.gap_outer.value))?;
        }

        if self.border_color_focused.value != self.original_values.border_color_focused {
            self.adapter.set_config_value(
                "border_color_focused",
                serde_json::json!(self.border_color_focused.value),
            )?;
        }

        if self.border_color_unfocused.value != self.original_values.border_color_unfocused {
            self.adapter.set_config_value(
                "border_color_unfocused",
                serde_json::json!(self.border_color_unfocused.value),
            )?;
        }

        if self.focus_follows_mouse.value != self.original_values.focus_follows_mouse {
            self.adapter.set_config_value(
                "focus_follows_mouse",
                serde_json::json!(self.focus_follows_mouse.value),
            )?;
        }

        if self.mouse_follows_focus.value != self.original_values.mouse_follows_focus {
            self.adapter.set_config_value(
                "mouse_follows_focus",
                serde_json::json!(self.mouse_follows_focus.value),
            )?;
        }

        if self.titlebar_enabled.value != self.original_values.titlebar_enabled {
            self.adapter.set_config_value(
                "titlebar_enabled",
                serde_json::json!(self.titlebar_enabled.value),
            )?;
        }

        if self.titlebar_height.value != self.original_values.titlebar_height {
            self.adapter.set_config_value(
                "titlebar_height",
                serde_json::json!(self.titlebar_height.value),
            )?;
        }

        // Update original values
        self.original_values = GarValues {
            border_width: self.border_width.value,
            border_color_focused: self.border_color_focused.value.clone(),
            border_color_unfocused: self.border_color_unfocused.value.clone(),
            gap_inner: self.gap_inner.value,
            gap_outer: self.gap_outer.value,
            titlebar_enabled: self.titlebar_enabled.value,
            titlebar_height: self.titlebar_height.value,
            focus_follows_mouse: self.focus_follows_mouse.value,
            mouse_follows_focus: self.mouse_follows_focus.value,
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

        // Build settings from current values
        let settings = GarSettings {
            border_width: Some(self.border_width.value as u32),
            gap_inner: Some(self.gap_inner.value as u32),
            gap_outer: Some(self.gap_outer.value as u32),
            border_color_focused: Some(self.border_color_focused.value.clone()),
            border_color_unfocused: Some(self.border_color_unfocused.value.clone()),
            border_color_urgent: None, // Not yet exposed in panel UI
            focus_follows_mouse: Some(self.focus_follows_mouse.value),
        };

        // Write settings as gar.set() calls
        writer.set_values(&settings.to_lua_statements());

        // Save the file
        writer.save()?;

        // Trigger reload
        crate::config::reload_component("gar")?;

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
            let result = match field {
                "border_width" => self.adapter.set_config_value(
                    "border_width",
                    serde_json::json!(self.border_width.value),
                ),
                "gap_inner" => self.adapter.set_config_value(
                    "gap_inner",
                    serde_json::json!(self.gap_inner.value),
                ),
                "gap_outer" => self.adapter.set_config_value(
                    "gap_outer",
                    serde_json::json!(self.gap_outer.value),
                ),
                "border_color_focused" => self.adapter.set_config_value(
                    "border_color_focused",
                    serde_json::json!(self.border_color_focused.value),
                ),
                "border_color_unfocused" => self.adapter.set_config_value(
                    "border_color_unfocused",
                    serde_json::json!(self.border_color_unfocused.value),
                ),
                "focus_follows_mouse" => self.adapter.set_config_value(
                    "focus_follows_mouse",
                    serde_json::json!(self.focus_follows_mouse.value),
                ),
                "mouse_follows_focus" => self.adapter.set_config_value(
                    "mouse_follows_focus",
                    serde_json::json!(self.mouse_follows_focus.value),
                ),
                "titlebar_enabled" => self.adapter.set_config_value(
                    "titlebar_enabled",
                    serde_json::json!(self.titlebar_enabled.value),
                ),
                "titlebar_height" => self.adapter.set_config_value(
                    "titlebar_height",
                    serde_json::json!(self.titlebar_height.value),
                ),
                _ => Ok(()),
            };

            // Update original value for this field if successful
            if result.is_ok() {
                match field {
                    "border_width" => self.original_values.border_width = self.border_width.value,
                    "gap_inner" => self.original_values.gap_inner = self.gap_inner.value,
                    "gap_outer" => self.original_values.gap_outer = self.gap_outer.value,
                    "border_color_focused" => {
                        self.original_values.border_color_focused = self.border_color_focused.value.clone()
                    }
                    "border_color_unfocused" => {
                        self.original_values.border_color_unfocused = self.border_color_unfocused.value.clone()
                    }
                    "focus_follows_mouse" => {
                        self.original_values.focus_follows_mouse = self.focus_follows_mouse.value
                    }
                    "mouse_follows_focus" => {
                        self.original_values.mouse_follows_focus = self.mouse_follows_focus.value
                    }
                    "titlebar_enabled" => {
                        self.original_values.titlebar_enabled = self.titlebar_enabled.value
                    }
                    "titlebar_height" => {
                        self.original_values.titlebar_height = self.titlebar_height.value
                    }
                    _ => {}
                }
                self.check_dirty();
            }

            result
        } else {
            Ok(())
        }
    }
}
