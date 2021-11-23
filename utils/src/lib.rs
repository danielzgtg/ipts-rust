pub use heatmap::get_heatmap;
pub use pointers::{deserialize_reports, serialize_reports, Pointers, Report};

pub const SCREEN_X: u16 = 2736;
pub const SCREEN_Y: u16 = 1824;

mod heatmap;
mod pointers;
