//! Shared physical display data types used across platform backends.

mod physical_display;
mod physical_display_metadata;
mod physical_display_state;
mod physical_display_update_content;

pub use physical_display::PhysicalDisplay;
pub use physical_display_metadata::PhysicalDisplayMetadata;
pub use physical_display_state::PhysicalDisplayState;
pub use physical_display_update_content::PhysicalDisplayUpdateContent;
