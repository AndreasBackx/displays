mod error;
mod manager;
mod monitor;
mod monitor_info;
mod physical_monitor;
mod types;

pub use error::{ApplyError, QueryError};
pub use manager::PhysicalDisplayManager;
pub use types::{
    PhysicalDisplay, PhysicalDisplayMetadata, PhysicalDisplayState, PhysicalDisplayUpdate,
    PhysicalDisplayUpdateContent,
};
pub use displays_windows_common::types::{Brightness, DisplayIdentifier, DisplayIdentifierInner};
