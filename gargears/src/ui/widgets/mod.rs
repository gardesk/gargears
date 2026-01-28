//! Reusable UI widgets for configuration panels

mod button;
mod color_picker;
mod dropdown;
mod label;
mod list;
mod number_input;
mod section;
mod slider;
mod text_input;
mod toggle;

pub use button::Button;
pub use color_picker::ColorPicker;
pub use dropdown::Dropdown;
pub use label::Label;
pub use list::List;
pub use number_input::NumberInput;
pub use section::Section;
pub use slider::Slider;
pub use text_input::TextInput;
pub use toggle::Toggle;

use gartk_core::Rect;

/// Common widget state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetState {
    Normal,
    Hovered,
    Focused,
    Disabled,
}

/// Result of handling an event
#[derive(Debug, Clone)]
pub enum WidgetEvent {
    /// No action needed
    None,
    /// Value was changed
    Changed,
    /// Widget was clicked/activated
    Clicked,
    /// Request focus
    Focus,
    /// Release focus
    Blur,
}

/// Check if a point is within a rect
pub fn point_in_rect(x: i32, y: i32, rect: Rect) -> bool {
    x >= rect.x
        && x < rect.x + rect.width as i32
        && y >= rect.y
        && y < rect.y + rect.height as i32
}
