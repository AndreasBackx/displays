use thiserror::Error;

/// Errors that can occur while applying Linux brightness updates through logind.
#[derive(Debug, Error)]
pub enum ApplyError {
    /// Connecting to the system bus failed.
    #[error("failed to connect to the system bus: {source}")]
    Connect {
        #[source]
        /// The underlying DBus error.
        source: dbus::Error,
    },
    /// Calling the logind `SetBrightness` method failed.
    #[error("failed to call logind SetBrightness for '{class}/{id}': {source}")]
    SetBrightness {
        /// The brightness device class passed to logind.
        class: String,
        /// The device identifier passed to logind.
        id: String,
        #[source]
        /// The underlying DBus error.
        source: dbus::Error,
    },
}
