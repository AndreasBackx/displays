// #![windows_subsystem = "windows"]
#[cfg(all(feature = "windows", feature = "linux"))]
compile_error!("features 'windows' and 'linux' are mutually exclusive");
#[cfg(not(any(feature = "windows", feature = "linux")))]
compile_error!("enable exactly one backend feature: 'windows' or 'linux'");

pub mod display;
pub mod display_identifier;
pub mod logical_display;
pub mod manager;
pub mod physical_display;
pub mod types;

#[cfg(feature = "linux")]
pub mod linux;

#[cfg(feature = "windows")]
pub mod windows;
