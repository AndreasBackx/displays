//! Cross-platform display query and update primitives for Windows and Linux.
//!
//! The crate exposes a single high-level entry point, [`manager::DisplayManager`],
//! plus data types for identifying displays, reading logical and physical state,
//! and requesting updates.
//!
//! Platform support is intentionally uneven:
//! - Windows supports both logical and physical display operations.
//! - Linux currently supports querying displays and applying physical brightness updates.
//! - macOS is unsupported.
//!
//! See [`manager::DisplayManager`] for the primary API.
// #![windows_subsystem = "windows"]
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
compile_error!("displays currently supports only Windows and Linux targets");

pub mod display;
pub mod display_identifier;
pub mod logical_display;
pub mod manager;
pub mod physical_display;
pub mod types;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;
