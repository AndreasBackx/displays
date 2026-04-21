#![warn(missing_docs)]
#![doc = include_str!("../docs/crate.md")]

mod ddc;
mod edid;
mod error;
mod manager;
mod types;

pub use displays_physical_types::PhysicalDisplayUpdate;
pub use error::{ApplyError, QueryError};
pub use manager::PhysicalDisplayManager;
