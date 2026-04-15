use std::io;

use displays_windows_common::error::WindowsError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum QueryError {
    #[error("expected '{key}' to exist in the registry but was missing")]
    RegistryKeyMissing { source: io::Error, key: String },
    #[error("invalid EDID for '{key}': {message}")]
    EDIDInvalid { message: String, key: String },
    #[error(transparent)]
    WindowsError {
        #[from]
        source: WindowsError,
    },
}

#[derive(Error, Debug)]
pub enum ApplyError {
    #[error(transparent)]
    WindowsError {
        #[from]
        source: WindowsError,
    },
    #[error("the following action is not (yet) supported: {message}")]
    Unsupported { message: String },
}
