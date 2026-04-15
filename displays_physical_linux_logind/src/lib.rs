#![warn(missing_docs)]
//! Linux brightness updates through the systemd-logind DBus API.

mod error;
mod manager;

pub use error::ApplyError;
pub use manager::PhysicalDisplayManagerLinuxLogind;
