//! IPC adapters for gardesk components

mod gar;
mod garbar;
mod garbg;
mod garclip;
mod garfield;
mod garlaunch;
mod garlock;
mod garnotify;
mod garshot;
mod garterm;
mod gartray;

pub use gar::{GarAdapter, GarKeybind, GarRule};
pub use garbar::GarbarAdapter;
pub use garbg::{GarbgAdapter, GarbgEvent};
pub use garclip::GarclipAdapter;
pub use garfield::GarfieldAdapter;
pub use garlaunch::GarlaunchAdapter;
pub use garlock::GarlockAdapter;
pub use garnotify::GarnotifyAdapter;
pub use garshot::GarshotAdapter;
pub use garterm::GartermAdapter;
pub use gartray::GartrayAdapter;
