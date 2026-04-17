use displays_types::{Orientation, PixelFormat, Point};

/// Requested changes to logical display state.
#[derive(Debug, Default, Clone)]
pub struct LogicalDisplayUpdateContent {
    /// Enable or disable the display.
    pub is_enabled: Option<bool>,
    /// Requested orientation.
    pub orientation: Option<Orientation>,
    /// Requested width in pixels.
    pub width: Option<u32>,
    /// Requested height in pixels.
    pub height: Option<u32>,
    /// Requested pixel format.
    pub pixel_format: Option<PixelFormat>,
    /// Requested top-left position.
    pub position: Option<Point>,
}
