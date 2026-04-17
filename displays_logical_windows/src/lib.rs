mod error;
mod manager;
mod types;

pub use error::{ApplyError, QueryError};
pub use manager::LogicalDisplayManager;
pub use types::{
    LogicalDisplay, LogicalDisplayMetadata, LogicalDisplayState, LogicalDisplayUpdate,
    LogicalDisplayUpdateContent,
};
