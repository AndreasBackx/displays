use thiserror::Error;

/// Errors that can occur while applying Linux brightness updates.
#[derive(Debug, Error)]
pub enum ApplyError {
    /// Querying Linux brightness devices through sysfs failed.
    #[error(transparent)]
    Query {
        #[from]
        /// The underlying sysfs query error.
        source: displays_physical_linux_sys::QueryError,
    },
    /// The sysfs backend failed before any fallback could happen.
    #[error(transparent)]
    Sys {
        #[from]
        /// The underlying sysfs backend error.
        source: displays_physical_linux_sys::ApplyError,
    },
    /// The logind fallback failed.
    #[error(transparent)]
    Logind {
        #[from]
        /// The underlying logind backend error.
        source: displays_physical_linux_logind::ApplyError,
    },
}
