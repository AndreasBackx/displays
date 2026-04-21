#![warn(missing_docs)]
#![doc = include_str!("../docs/crate.md")]

mod error;
mod manager;

pub use error::ApplyError;
pub use manager::PhysicalDisplayManagerLinuxLogind;
