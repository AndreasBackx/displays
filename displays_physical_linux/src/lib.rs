#![warn(missing_docs)]
//! Linux physical display facade combining DDC, sysfs, and logind backends.

mod ddc;
mod edid;
mod error;
mod manager;
mod types;

pub use displays_physical_types::PhysicalDisplayUpdate;
pub use error::{ApplyError, QueryError};
pub use manager::PhysicalDisplayManager;
