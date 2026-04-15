use displays_windows_common::error::WindowsError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum QueryError {
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
}
