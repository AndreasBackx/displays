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
