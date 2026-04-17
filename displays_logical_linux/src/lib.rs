mod error;
mod manager;
mod types;
mod wayland;

pub use displays_logical_types::{
    LogicalDisplay, LogicalDisplayMetadata, LogicalDisplayState, LogicalDisplayUpdate,
    LogicalDisplayUpdateContent,
};
pub use error::{ApplyError, QueryError};
pub use manager::LogicalDisplayManager;
pub(crate) use types::logical_display_matches;
