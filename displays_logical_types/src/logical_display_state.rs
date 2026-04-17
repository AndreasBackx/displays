use displays_types::{Orientation, PixelFormat, Point};

/// The current logical display state.
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Default)]
pub struct LogicalDisplayState {
    /// Whether the logical display is enabled.
    pub is_enabled: bool,
    /// Current display orientation.
    pub orientation: Orientation,
    /// Current logical width in pixels when known.
    pub width: Option<u32>,
    /// Current logical height in pixels when known.
    pub height: Option<u32>,
    /// Current pixel format when known.
    pub pixel_format: Option<PixelFormat>,
    /// Current top-left position when known.
    pub position: Option<Point>,
}
