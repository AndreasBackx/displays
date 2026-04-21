#![doc = include_str!("../docs/crate.md")]

mod error;
mod manager;
mod monitor;
mod monitor_info;
mod physical_monitor;
mod types;

pub use displays_physical_types::PhysicalDisplayUpdate;
pub use error::{ApplyError, QueryError};
pub use manager::PhysicalDisplayManager;
