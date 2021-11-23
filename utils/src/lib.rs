pub use heatmap::get_heatmap;
pub use pointers::{Pointers, Report};
#[cfg(feature = "tcp")]
pub use tcp::{deserialize_reports, serialize_reports};

pub const SCREEN_X: u16 = 2736;
pub const SCREEN_Y: u16 = 1824;

mod heatmap;
mod pointers;
#[cfg(feature = "tcp")]
mod tcp;
