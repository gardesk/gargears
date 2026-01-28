//! Sidebar navigation component

use crate::ipc::discovery::DaemonStatus;
use crate::panels::Component;
use crate::ui::Layout;
use anyhow::Result;
use gartk_core::{Color, Rect, Theme};
use gartk_render::{Renderer, TextStyle};

/// Item height in sidebar
const ITEM_HEIGHT: i32 = 36;
/// Status indicator radius
const STATUS_RADIUS: f64 = 4.0;

/// Sidebar navigation
pub struct Sidebar {
    hovered: Option<Component>,
}

impl Sidebar {
    /// Create a new sidebar
    pub fn new() -> Self {
        Self { hovered: None }
    }

    /// Get the component at a given y position
    pub fn component_at_y(
        &self,
        y: i32,
        layout: &Layout,
        theme: &Theme,
    ) -> Option<Component> {
        let bounds = layout.sidebar_bounds();
        let start_y = bounds.y + theme.padding as i32;

        let components = Component::all();
        for (i, component) in components.iter().enumerate() {
            let item_y = start_y + (i as i32 * ITEM_HEIGHT);
            if y >= item_y && y < item_y + ITEM_HEIGHT {
                return Some(*component);
            }
        }
        None
    }

    /// Render the sidebar
    pub fn render(
        &self,
        renderer: &mut Renderer,
        layout: &Layout,
        theme: &Theme,
        selected: Component,
        daemon_status: &[DaemonStatus],
    ) -> Result<()> {
        let bounds = layout.sidebar_bounds();

        // Draw sidebar background
        renderer.fill_rounded_rect(bounds, theme.border_radius, theme.item_background)?;

        // Draw title
        let title_style = TextStyle::new()
            .font_family(&theme.font_family)
            .font_size(theme.font_size * 1.2)
            .color(theme.foreground);

        renderer.text(
            "Components",
            (bounds.x + theme.padding as i32) as f64,
            (bounds.y + theme.padding as i32) as f64,
            &title_style,
        )?;

        // Draw components
        let start_y = bounds.y + theme.padding as i32 + theme.font_size as i32 + 16;
        let components = Component::all();

        for (i, component) in components.iter().enumerate() {
            let item_y = start_y + (i as i32 * ITEM_HEIGHT);
            let item_rect = Rect::new(
                bounds.x + 4,
                item_y,
                bounds.width - 8,
                ITEM_HEIGHT as u32,
            );

            // Highlight selected
            if *component == selected {
                renderer.fill_rounded_rect(
                    item_rect,
                    theme.border_radius / 2.0,
                    theme.item_selected_background,
                )?;
            }

            // Draw status indicator
            let status = daemon_status.iter().find(|s| s.component == *component);
            let indicator_color = match status {
                Some(s) if s.running => Color::from_u8(0x50, 0xfa, 0x7b, 0xff), // Green
                _ => Color::from_u8(0x6c, 0x75, 0x7d, 0xff),                     // Gray
            };

            let indicator_x = bounds.x + theme.padding as i32 + STATUS_RADIUS as i32;
            let indicator_y = item_y + (ITEM_HEIGHT / 2);

            renderer.fill_circle(
                indicator_x as f64,
                indicator_y as f64,
                STATUS_RADIUS,
                indicator_color,
            )?;

            // Draw component name
            let name_style = TextStyle::new()
                .font_family(&theme.font_family)
                .font_size(theme.font_size)
                .color(if *component == selected {
                    theme.item_selected_foreground
                } else {
                    theme.item_foreground
                });

            let text_x = bounds.x + theme.padding as i32 + (STATUS_RADIUS * 2.0) as i32 + 12;
            let text_y = item_y + (ITEM_HEIGHT - theme.font_size as i32) / 2;

            renderer.text(
                component.display_name(),
                text_x as f64,
                text_y as f64,
                &name_style,
            )?;
        }

        Ok(())
    }
}
