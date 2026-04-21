#![warn(missing_docs)]
#![doc = include_str!("../docs/crate.md")]
#![doc = ""]
#![doc = include_str!("../docs/fragments/crate-graph-note.md")]

mod error;
mod manager;

pub use error::ApplyError;
pub use manager::PhysicalDisplayManagerLinuxLogind;
