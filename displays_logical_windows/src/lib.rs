#![doc = include_str!("../docs/crate.md")]
#![doc = ""]
#![doc = include_str!("../docs/fragments/backend-start-with-displays.md")]

mod error;
mod manager;
mod types;

pub use error::{ApplyError, QueryError};
pub use manager::LogicalDisplayManager;
