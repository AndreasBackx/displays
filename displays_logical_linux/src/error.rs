use thiserror::Error;

#[derive(Error, Debug)]
pub enum WaylandError {
    #[error("WAYLAND_DISPLAY is not set")]
    MissingWaylandDisplay,
    #[error("failed to connect to the Wayland compositor: {message}")]
    Connect { message: String },
    #[error("the compositor does not expose zwlr_output_manager_v1")]
    MissingOutputManager,
    #[error("Wayland protocol error: {message}")]
    Protocol { message: String },
}

#[derive(Error, Debug)]
pub enum QueryError {
    #[error(transparent)]
    Wayland {
        #[from]
        source: WaylandError,
    },
}

#[derive(Error, Debug)]
pub enum ApplyError {
    #[error(transparent)]
    Wayland {
        #[from]
        source: WaylandError,
    },
    #[error("requested logical size does not map exactly to an available mode and scale")]
    UnsupportedLogicalSize,
    #[error("the compositor rejected the requested configuration")]
    Rejected,
    #[error("the compositor cancelled the configuration before it could be applied")]
    Cancelled,
}
