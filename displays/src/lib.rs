#![doc = include_str!("../docs/crate.md")]
// #![windows_subsystem = "windows"]
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
compile_error!("displays currently supports only Windows and Linux targets");

pub mod display;
pub mod error;
pub mod manager;
mod manager_types;
pub mod types;
