use displays_types::{Orientation, PixelFormat, Point, Size};

/// The current logical display state.
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Default)]
pub struct LogicalDisplayState {
    /// Whether the logical display is enabled.
    pub is_enabled: bool,
    /// Current display orientation.
    pub orientation: Orientation,
    /// Current logical size in pixels when known.
    pub logical_size: Option<Size>,
    /// Current output mode size in pixels when known.
    pub mode_size: Option<Size>,
    /// Current display scale ratio in milli-units where 1000 == 1.0x.
    pub scale_ratio_milli: Option<u32>,
    /// Current pixel format when known.
    pub pixel_format: Option<PixelFormat>,
    /// Current top-left position in physical pixels when known.
    pub mode_position: Option<Point>,
    /// Current top-left position in logical pixels when known.
    pub logical_position: Option<Point>,
}
