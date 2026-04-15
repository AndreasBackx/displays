mod error;
mod manager;
mod types;

pub use error::{ApplyError, QueryError};
pub use manager::LogicalDisplayManager;
pub use types::{
    LogicalDisplay, LogicalDisplayMetadata, LogicalDisplayState, LogicalDisplayUpdate,
    LogicalDisplayUpdateContent,
};
pub use displays_windows_common::types::{
    DisplayIdentifier, DisplayIdentifierInner, Orientation, PixelFormat, Point,
};
