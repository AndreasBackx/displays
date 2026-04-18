use thiserror::Error;

#[cfg(target_os = "windows")]
use displays_logical_windows::QueryError as LogicalDisplayQueryError;

#[cfg(target_os = "windows")]
use displays_physical_windows::QueryError as PhysicalDisplayQueryError;

#[cfg(target_os = "linux")]
use displays_logical_linux::QueryError as LogicalDisplayQueryError;

#[cfg(target_os = "linux")]
use displays_physical_linux::QueryError as PhysicalDisplayQueryError;

#[cfg(target_os = "windows")]
use displays_logical_windows::ApplyError as LogicalDisplayApplyError;

#[cfg(target_os = "windows")]
use displays_physical_windows::ApplyError as PhysicalDisplayApplyError;

#[cfg(target_os = "linux")]
use displays_logical_linux::ApplyError as LogicalDisplayApplyError;

#[cfg(target_os = "linux")]
use displays_physical_linux::ApplyError as PhysicalDisplayApplyError;

/// Errors that can occur while querying display state.
#[derive(Error, Debug)]
pub enum DisplayQueryError {
    #[error("physical querying error")]
    Physical {
        #[from]
        source: PhysicalDisplayQueryError,
    },
    #[error("logical querying error")]
    Logical {
        #[from]
        source: LogicalDisplayQueryError,
    },
}

/// Errors that can occur while applying display updates.
#[derive(Error, Debug)]
pub enum DisplayApplyError {
    #[error("error while first querying displays")]
    Query {
        #[from]
        source: DisplayQueryError,
    },
    #[error("physical applying error: {source}")]
    Physical {
        #[from]
        source: PhysicalDisplayApplyError,
    },
    #[error("logical applying error")]
    Logical {
        #[from]
        source: LogicalDisplayApplyError,
    },
}
