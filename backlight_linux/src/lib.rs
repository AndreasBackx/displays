#![warn(missing_docs)]
//! Linux brightness device query and update primitives.
//!
//! `backlight_linux` provides a small manager-oriented API for working with
//! devices exposed under `/sys/class/backlight` and `/sys/class/leds`.
//!
//! The crate is intentionally library-focused rather than CLI-shaped:
//! it exposes typed identifiers, device state, and update operations while
//! staying close to how Linux makes brightness available through sysfs.
//!
//! # How Linux exposes brightness
//!
//! On Linux, brightness is commonly exposed through sysfs directories and files.
//! Sysfs is the kernel's filesystem-style interface for devices and drivers.
//!
//! This crate reads and writes values from device directories under:
//!
//! - `/sys/class/backlight`
//! - `/sys/class/leds`
//!
//! A typical device directory contains:
//!
//! - `brightness`: the current configured raw brightness value
//! - `max_brightness`: the maximum supported raw brightness value
//! - `actual_brightness`: the effective brightness value when the driver exposes it
//!
//! These values are device-specific integers. They are not universal physical
//! units, so one device's raw range may be `0..=255` while another's is
//! `0..=937` or `0..=3`.
//!
//! Because of that, this crate exposes both raw values and a derived percentage.
//! Raw values stay first-class API data so callers can work directly with Linux
//! semantics when needed.
//!
//! # Permissions
//!
//! Reading sysfs brightness values is often available to regular users, but
//! writing usually depends on system configuration. Direct writes may require:
//!
//! - root privileges
//! - udev rules that grant write access to the relevant `brightness` file
//! - another system-specific permission setup
//!
//! This first version performs direct sysfs writes only.
//!
//! # References
//!
//! - Kernel sysfs docs: <https://www.kernel.org/doc/html/latest/filesystems/sysfs.html>
//! - Kernel backlight docs: <https://www.kernel.org/doc/html/latest/gpu/backlight.html>
//! - Kernel LED class docs: <https://www.kernel.org/doc/html/latest/leds/leds-class.html>
//! - Local man pages worth reading: `man 5 sysfs`, `man 7 udev`, `man 8 udevadm`
//!
//! # Overview
//!
//! The primary entry point is [`BacklightManager`].
//! It can:
//!
//! - enumerate brightness-capable devices with [`BacklightManager::list`]
//! - look up devices by identifier with [`BacklightManager::get`]
//! - apply brightness changes with [`BacklightManager::update`]
//! - validate update requests without writing with [`BacklightManager::validate`]
//!
//! # Example
//!
//! ```rust,no_run
//! use backlight_linux::{
//!     BacklightManager, BrightnessUpdate, DeviceClass, DeviceIdentifier, DeviceUpdate,
//! };
//!
//! let manager = BacklightManager::new();
//! let devices = manager.list()?;
//!
//! for device in &devices {
//!     println!(
//!         "{} {}: {}%",
//!         match device.metadata.class {
//!             DeviceClass::Backlight => "backlight",
//!             DeviceClass::Leds => "leds",
//!         },
//!         device.metadata.id,
//!         device.state.brightness_percent,
//!     );
//! }
//!
//! let remaining = manager.update(vec![DeviceUpdate {
//!     id: DeviceIdentifier {
//!         class: Some(DeviceClass::Backlight),
//!         id: Some("intel_backlight".to_string()),
//!         path: None,
//!     },
//!     brightness: Some(BrightnessUpdate::Percent(50)),
//! }])?;
//!
//! assert!(remaining.is_empty());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod error;
mod manager;
mod types;

pub use error::{ApplyError, QueryError};
pub use manager::BacklightManager;
pub use types::{
    BrightnessUpdate, Device, DeviceClass, DeviceIdentifier, DeviceMetadata, DeviceState,
    DeviceUpdate,
};
