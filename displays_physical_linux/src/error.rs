use thiserror::Error;

/// Errors that can occur while querying Linux physical displays.
#[derive(Debug, Error)]
pub enum QueryError {
    /// Enumerating DDC displays failed.
    #[error("failed to enumerate DDC displays")]
    Enumerate,
    /// Accessing the Linux backlight backend failed.
    #[error("failed to query Linux backlight devices: {message}")]
    BacklightQuery {
        /// A backend-specific error message.
        message: String,
    },
}

/// Errors that can occur while applying Linux physical display updates.
#[derive(Debug, Error)]
pub enum ApplyError {
    /// Querying current physical displays failed.
    #[error(transparent)]
    Query {
        /// The underlying query error.
        #[from]
        source: QueryError,
    },
    /// The display does not expose brightness via DDC VCP 0x10.
    #[error("display '{display_id}' does not expose brightness via VCP 0x10: {message}")]
    UnsupportedMonitor {
        /// The platform-specific display identifier.
        display_id: String,
        /// The backend error message.
        message: String,
    },
    /// Accessing the display was denied.
    #[error("insufficient permissions for display '{display_id}'")]
    PermissionDenied {
        /// The platform-specific display identifier.
        display_id: String,
    },
    /// Accessing the i2c device failed because it was unavailable.
    #[error("missing i2c access for display '{display_id}'")]
    MissingI2cAccess {
        /// The platform-specific display identifier.
        display_id: String,
    },
    /// A DDC operation failed.
    #[error("failed to set brightness for display '{display_id}': {message}")]
    DdcOperation {
        /// The platform-specific display identifier.
        display_id: String,
        /// The backend error message.
        message: String,
    },
    /// A Linux backlight operation failed.
    #[error("failed to set backlight brightness for display '{display_id}': {message}")]
    BacklightOperation {
        /// The platform-specific display identifier.
        display_id: String,
        /// The backend error message.
        message: String,
    },
}
