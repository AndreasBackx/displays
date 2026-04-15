use std::time::Duration;

use dbus::blocking::Connection;
use displays_physical_linux_sys::DeviceClass;

use crate::error::ApplyError;

const LOGIN1_DESTINATION: &str = "org.freedesktop.login1";
const LOGIN1_SESSION_PATH: &str = "/org/freedesktop/login1/session/auto";
const LOGIN1_INTERFACE: &str = "org.freedesktop.login1.Session";
const LOGIN1_TIMEOUT: Duration = Duration::from_secs(5);

/// High-level entry point for setting Linux brightness through systemd-logind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PhysicalDisplayManagerLinuxLogind;

impl PhysicalDisplayManagerLinuxLogind {
    /// Creates a logind brightness manager.
    pub fn new() -> Self {
        Self
    }

    /// Sets the raw brightness value for a Linux brightness device.
    pub fn set_brightness(
        &self,
        class: DeviceClass,
        id: &str,
        value: u32,
    ) -> Result<(), ApplyError> {
        let bus = Connection::new_system().map_err(|source| ApplyError::Connect { source })?;
        let proxy = bus.with_proxy(LOGIN1_DESTINATION, LOGIN1_SESSION_PATH, LOGIN1_TIMEOUT);

        let _: () = proxy
            .method_call(
                LOGIN1_INTERFACE,
                "SetBrightness",
                (class.directory_name(), id, value),
            )
            .map_err(|source| ApplyError::SetBrightness {
                class: class.directory_name().to_string(),
                id: id.to_string(),
                source,
            })?;
        Ok(())
    }
}
