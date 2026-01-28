//! Layout calculations for gargears

use gartk_core::Rect;

/// Layout constants
const SIDEBAR_WIDTH: u32 = 200;
const PADDING: i32 = 12;
const GAP: i32 = 8;

/// Layout manager
pub struct Layout {
    width: u32,
    height: u32,
}

impl Layout {
    /// Create a new layout for the given window size
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Get the sidebar bounds
    pub fn sidebar_bounds(&self) -> Rect {
        Rect::new(
            PADDING,
            PADDING,
            SIDEBAR_WIDTH,
            self.height - (PADDING * 2) as u32,
        )
    }

    /// Get the content panel bounds
    pub fn content_bounds(&self) -> Rect {
        let x = PADDING + SIDEBAR_WIDTH as i32 + GAP;
        Rect::new(
            x,
            PADDING,
            self.width - x as u32 - PADDING as u32,
            self.height - (PADDING * 2) as u32,
        )
    }

    /// Get the action buttons bounds (bottom of content panel)
    pub fn action_bounds(&self) -> Rect {
        let content = self.content_bounds();
        let button_height = 40;
        Rect::new(
            content.x,
            content.y + content.height as i32 - button_height,
            content.width,
            button_height as u32,
        )
    }

    /// Sidebar width constant
    pub fn sidebar_width(&self) -> u32 {
        SIDEBAR_WIDTH
    }

    /// Padding constant
    pub fn padding(&self) -> i32 {
        PADDING
    }
}
