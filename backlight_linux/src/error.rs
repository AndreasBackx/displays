use std::path::PathBuf;

use thiserror::Error;

/// Errors that can occur while querying Linux brightness devices.
#[derive(Debug, Error)]
pub enum QueryError {
    /// Reading a sysfs class directory such as `/sys/class/backlight` failed.
    #[error("failed to read sysfs class directory '{path}': {source}")]
    ReadClassDirectory {
        /// The class directory that failed to be read.
        path: PathBuf,
        #[source]
        /// The underlying I/O error.
        source: std::io::Error,
    },
    /// Reading metadata for an individual sysfs device directory failed.
    #[error("failed to read device metadata from '{path}': {source}")]
    ReadDeviceDirectory {
        /// The device directory that failed to be inspected.
        path: PathBuf,
        #[source]
        /// The underlying I/O error.
        source: std::io::Error,
    },
    /// A required sysfs file such as `brightness` or `max_brightness` was missing.
    #[error("missing required sysfs file '{path}'")]
    MissingFile {
        /// The missing file path.
        path: PathBuf,
    },
    /// Reading a sysfs file failed.
    #[error("failed to read sysfs file '{path}': {source}")]
    ReadFile {
        /// The file path that failed to be read.
        path: PathBuf,
        #[source]
        /// The underlying I/O error.
        source: std::io::Error,
    },
    /// A sysfs file contained a value that could not be parsed as an integer.
    #[error("failed to parse unsigned integer from '{path}': '{content}'")]
    ParseFile {
        /// The file path that contained the invalid value.
        path: PathBuf,
        /// The invalid trimmed file contents.
        content: String,
    },
}

/// Errors that can occur while applying Linux brightness updates.
#[derive(Debug, Error)]
pub enum ApplyError {
    /// Querying current devices and state failed before the write could happen.
    #[error(transparent)]
    Query {
        #[from]
        /// The underlying query error.
        source: QueryError,
    },
    /// Writing a normalized brightness value to sysfs failed.
    #[error("failed to write brightness to '{path}': {source}")]
    WriteFile {
        /// The target `brightness` file path.
        path: PathBuf,
        #[source]
        /// The underlying I/O error.
        source: std::io::Error,
    },
}
