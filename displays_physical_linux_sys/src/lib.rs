#![warn(missing_docs)]
#![doc = include_str!("../docs/crate.md")]

mod error;
mod manager;
mod types;

pub use error::{ApplyError, QueryError};
pub use manager::{normalize_brightness_update, PhysicalDisplayManagerLinuxSys};
pub use types::{
    BrightnessUpdate, Device, DeviceClass, DeviceIdentifier, DeviceMetadata, DeviceState,
    DeviceUpdate,
};
