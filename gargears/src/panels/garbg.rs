//! Configuration panel for garbg (wallpaper daemon)

use crate::config::{GarbgConfig, GarbgSlideshow};
use crate::ipc::adapters::GarbgAdapter;
use crate::ui::widgets::{Button, Label, NumberInput, Section, Toggle, WidgetEvent};
use crate::ui::{Panel, PanelAction};
use anyhow::Result;
use gartk_core::{InputEvent, Key, Rect, Theme};
use gartk_render::{Renderer, TextStyle};

/// Garbg configuration panel
pub struct GarbgPanel {
    adapter: GarbgAdapter,

    // Sections
    current_section: Section,
    playlist_section: Section,
    slideshow_section: Section,
    controls_section: Section,

    // Current wallpaper info (read-only display)
    current_source: Label,
    current_wallpaper: Label,
    playlist_position: Label,

    // Slideshow settings
    interval: NumberInput,
    paused: Toggle,

    // Control buttons
    prev_button: Button,
    next_button: Button,
    pause_button: Button,

    // Action buttons
    apply_button: Button,
    reset_button: Button,
    save_button: Button,

    // State
    original_values: GarbgValues,
    dirty: bool,
    scroll_offset: i32,
    instant_apply: bool,
    pending_change: Option<&'static str>,
}

#[derive(Clone, Default)]
struct GarbgValues {
    interval: i32,
    paused: bool,
}

impl GarbgPanel {
    pub fn new(mut adapter: GarbgAdapter) -> Self {
        let (values, current_source, current_wallpaper, playlist_pos) = if adapter.is_connected() {
            Self::fetch_values(&mut adapter)
        } else {
            (
                GarbgValues::default(),
                "Not connected".to_string(),
                "N/A".to_string(),
                "N/A".to_string(),
            )
        };

        Self {
            adapter,
            current_section: Section::new("Current Wallpaper"),
            playlist_section: Section::new("Playlist"),
            slideshow_section: Section::new("Slideshow"),
            controls_section: Section::new("Controls"),
            current_source: Label::new("Source", &current_source),
            current_wallpaper: Label::new("File", &current_wallpaper),
            playlist_position: Label::new("Position", &playlist_pos),
            interval: NumberInput::new("Interval (sec)", values.interval)
                .with_range(5, 3600)
                .with_step(5),
            paused: Toggle::new("Paused", values.paused),
            prev_button: Button::new("◀ Prev"),
            next_button: Button::new("Next ▶"),
            pause_button: Button::new(if values.paused { "Resume" } else { "Pause" }),
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

    fn fetch_values(
        adapter: &mut GarbgAdapter,
    ) -> (GarbgValues, String, String, String) {
        if let Ok(status) = adapter.status() {
            let source = status.current_source.unwrap_or_else(|| "Unknown".into());
            let wallpaper = status
                .current_wallpaper
                .map(|w| {
                    // Shorten long paths
                    if w.len() > 40 {
                        format!("...{}", &w[w.len() - 37..])
                    } else {
                        w
                    }
                })
                .unwrap_or_else(|| "None".into());
            let playlist_pos = match (status.playlist_index, status.playlist_total) {
                (Some(idx), Some(total)) => format!("{} / {}", idx + 1, total),
                _ => "N/A".into(),
            };

            (
                GarbgValues {
                    interval: status.interval_secs.unwrap_or(300) as i32,
                    paused: status.paused,
                },
                source,
                wallpaper,
                playlist_pos,
            )
        } else {
            (
                GarbgValues {
                    interval: 300,
                    paused: false,
                },
                "Unknown".into(),
                "None".into(),
                "N/A".into(),
            )
        }
    }

    fn refresh_status(&mut self) {
        if let Ok(status) = self.adapter.status() {
            self.current_source.text = status.current_source.unwrap_or_else(|| "Unknown".into());
            self.current_wallpaper.text = status
                .current_wallpaper
                .map(|w| {
                    if w.len() > 40 {
                        format!("...{}", &w[w.len() - 37..])
                    } else {
                        w
                    }
                })
                .unwrap_or_else(|| "None".into());
            self.playlist_position.text = match (status.playlist_index, status.playlist_total) {
                (Some(idx), Some(total)) => format!("{} / {}", idx + 1, total),
                _ => "N/A".into(),
            };
            self.paused.value = status.paused;
            self.pause_button = Button::new(if status.paused { "Resume" } else { "Pause" });
        }
    }

    fn check_dirty(&mut self) {
        self.dirty = self.interval.value != self.original_values.interval
            || self.paused.value != self.original_values.paused;
    }

    fn layout_widgets(&mut self, bounds: Rect, theme: &Theme) {
        let padding = theme.padding as i32;
        let row_height = 36;
        let section_height = 32;
        let widget_width = bounds.width - (padding * 2) as u32;

        let mut y = bounds.y + padding - self.scroll_offset;

        // Current section
        self.current_section.bounds =
            Rect::new(bounds.x + padding, y, widget_width, section_height as u32);
        y += section_height;

        if self.current_section.expanded {
            self.current_source.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;

            self.current_wallpaper.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;
        }

        y += padding / 2;

        // Playlist section
        self.playlist_section.bounds =
            Rect::new(bounds.x + padding, y, widget_width, section_height as u32);
        y += section_height;

        if self.playlist_section.expanded {
            self.playlist_position.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;
        }

        y += padding / 2;

        // Slideshow section
        self.slideshow_section.bounds =
            Rect::new(bounds.x + padding, y, widget_width, section_height as u32);
        y += section_height;

        if self.slideshow_section.expanded {
            self.interval.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;

            self.paused.bounds =
                Rect::new(bounds.x + padding + 16, y, widget_width - 16, row_height as u32);
            y += row_height;
        }

        y += padding / 2;

        // Controls section
        self.controls_section.bounds =
            Rect::new(bounds.x + padding, y, widget_width, section_height as u32);
        y += section_height;

        if self.controls_section.expanded {
            let control_button_width = 80;
            let control_y = y + 4;

            self.prev_button.bounds = Rect::new(
                bounds.x + padding + 16,
                control_y,
                control_button_width as u32,
                28,
            );

            self.pause_button.bounds = Rect::new(
                bounds.x + padding + 16 + control_button_width + 8,
                control_y,
                control_button_width as u32,
                28,
            );

            self.next_button.bounds = Rect::new(
                bounds.x + padding + 16 + (control_button_width + 8) * 2,
                control_y,
                control_button_width as u32,
                28,
            );
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

impl Panel for GarbgPanel {
    fn name(&self) -> &str {
        "garbg"
    }

    fn description(&self) -> &str {
        "Wallpaper daemon"
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn render(&mut self, renderer: &mut Renderer, bounds: Rect, theme: &Theme) -> Result<()> {
        self.layout_widgets(bounds, theme);

        // Render sections and widgets
        self.current_section.render(renderer, theme)?;
        if self.current_section.expanded {
            self.current_source.render(renderer, theme)?;
            self.current_wallpaper.render(renderer, theme)?;
        }

        self.playlist_section.render(renderer, theme)?;
        if self.playlist_section.expanded {
            self.playlist_position.render(renderer, theme)?;
        }

        self.slideshow_section.render(renderer, theme)?;
        if self.slideshow_section.expanded {
            self.interval.render(renderer, theme)?;
            self.paused.render(renderer, theme)?;
        }

        self.controls_section.render(renderer, theme)?;
        if self.controls_section.expanded {
            self.prev_button.render(renderer, theme)?;
            self.pause_button.render(renderer, theme)?;
            self.next_button.render(renderer, theme)?;
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
                if let WidgetEvent::Changed = self.current_section.on_click(x, y) {
                    return PanelAction::Redraw;
                }
                if let WidgetEvent::Changed = self.playlist_section.on_click(x, y) {
                    return PanelAction::Redraw;
                }
                if let WidgetEvent::Changed = self.slideshow_section.on_click(x, y) {
                    return PanelAction::Redraw;
                }
                if let WidgetEvent::Changed = self.controls_section.on_click(x, y) {
                    return PanelAction::Redraw;
                }

                // Check widgets - with instant apply support
                if let WidgetEvent::Changed = self.interval.on_click(x, y) {
                    self.check_dirty();
                    // Interval change currently needs config save, no instant apply
                    return PanelAction::Redraw;
                }
                if let WidgetEvent::Changed = self.paused.on_click(x, y) {
                    self.check_dirty();
                    self.pending_change = Some("paused");
                    return if self.instant_apply { PanelAction::InstantApply } else { PanelAction::Redraw };
                }

                // Control buttons
                if let WidgetEvent::Clicked = self.prev_button.on_click(x, y) {
                    let _ = self.adapter.prev(None);
                    self.refresh_status();
                    return PanelAction::Redraw;
                }
                if let WidgetEvent::Clicked = self.next_button.on_click(x, y) {
                    let _ = self.adapter.next(None);
                    self.refresh_status();
                    return PanelAction::Redraw;
                }
                if let WidgetEvent::Clicked = self.pause_button.on_click(x, y) {
                    let _ = self.adapter.toggle();
                    self.refresh_status();
                    return PanelAction::Redraw;
                }

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
        changed |= self.current_section.on_mouse_move(x, y);
        changed |= self.playlist_section.on_mouse_move(x, y);
        changed |= self.slideshow_section.on_mouse_move(x, y);
        changed |= self.controls_section.on_mouse_move(x, y);

        changed |= self.interval.on_mouse_move(x, y);
        changed |= self.paused.on_mouse_move(x, y);

        changed |= self.prev_button.on_mouse_move(x, y);
        changed |= self.pause_button.on_mouse_move(x, y);
        changed |= self.next_button.on_mouse_move(x, y);

        changed |= self.apply_button.on_mouse_move(x, y);
        changed |= self.reset_button.on_mouse_move(x, y);
        changed |= self.save_button.on_mouse_move(x, y);
        changed
    }

    fn reset(&mut self) {
        self.interval.value = self.original_values.interval;
        self.paused.value = self.original_values.paused;
        self.dirty = false;
    }

    fn apply(&mut self) -> Result<()> {
        if !self.adapter.is_connected() {
            self.adapter.connect()?;
        }

        // Toggle pause state if changed
        if self.paused.value != self.original_values.paused {
            self.adapter.toggle()?;
        }

        // Note: interval changes require config file edit
        // For now we track it but it won't take effect until config save

        // Update original values
        self.original_values = GarbgValues {
            interval: self.interval.value,
            paused: self.paused.value,
        };

        self.dirty = false;
        self.refresh_status();
        Ok(())
    }

    fn update_status(&mut self, _status: serde_json::Value) {
        self.refresh_status();
    }

    fn has_config_file(&self) -> bool {
        true
    }

    fn save_to_config(&mut self) -> Result<()> {
        // Load existing config or create new
        let mut config = GarbgConfig::load().unwrap_or_default();

        // Update slideshow settings
        config.slideshow = Some(GarbgSlideshow {
            enabled: !self.paused.value,
            interval_secs: Some(self.interval.value as u64),
            shuffle: config.slideshow.as_ref().and_then(|s| s.shuffle),
        });

        // Save config
        config.save()?;

        // Trigger reload
        crate::config::reload_component("garbg")?;

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
                "paused" => {
                    // Toggle pause state
                    if self.paused.value != self.original_values.paused {
                        self.adapter.toggle()?;
                        self.original_values.paused = self.paused.value;
                        self.pause_button.label = if self.paused.value {
                            "Resume".to_string()
                        } else {
                            "Pause".to_string()
                        };
                    }
                }
                _ => {}
            }
            self.check_dirty();
            self.refresh_status();
        }

        Ok(())
    }
}
