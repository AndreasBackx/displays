#![doc = include_str!("../docs/crate.md")]

mod error;
mod manager;
mod types;
mod wayland;

pub use error::{ApplyError, QueryError};
pub use manager::LogicalDisplayManager;
pub(crate) use types::logical_display_matches;
