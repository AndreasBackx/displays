#![warn(missing_docs)]
//! Linux physical display facade combining DDC, sysfs, and logind backends.

mod ddc;
mod error;
mod manager;
mod types;

pub use error::{ApplyError, QueryError};
pub use manager::PhysicalDisplayManager;
pub use types::PhysicalDisplayUpdate;
pub use displays_physical_types::{
    Brightness, PhysicalDisplay, PhysicalDisplayMetadata, PhysicalDisplayState,
    PhysicalDisplayUpdateContent,
};
pub use displays_types::DisplayIdentifier;
