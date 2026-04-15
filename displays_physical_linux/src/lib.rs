#![warn(missing_docs)]
//! Linux brightness device facade combining sysfs and logind backends.

mod error;
mod manager;

pub use error::ApplyError;
pub use manager::PhysicalDisplayManagerLinux;
pub use displays_physical_linux_sys::{
    normalize_brightness_update, BrightnessUpdate, Device, DeviceClass, DeviceIdentifier,
    DeviceMetadata, DeviceState, DeviceUpdate, QueryError,
};
