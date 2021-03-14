pub use heatmap::get_heatmap;
pub use pointers::{Counter, Pointers, Report, serialize_reports, deserialize_reports};

mod pointers;
mod heatmap;
