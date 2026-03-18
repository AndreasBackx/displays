use crate::{
    display::DisplayUpdateInner,
    display_identifier::DisplayIdentifierInner,
    types::{Orientation, PixelFormat, Point},
};

/// Stable metadata describing a logical display.
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct LogicalDisplayMetadata {
    /// Human-readable display name.
    pub name: String,
    /// Platform-specific display path.
    pub path: String,
    /// Windows GDI device id when available.
    pub gdi_device_id: Option<u32>,
}

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

/// A logical display and its current state.
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct LogicalDisplay {
    /// Logical display metadata.
    pub metadata: LogicalDisplayMetadata,
    /// Current logical display state.
    pub state: LogicalDisplayState,
}

#[derive(Debug, Clone)]
#[cfg_attr(target_os = "linux", allow(dead_code))]
pub(crate) struct LogicalDisplayUpdate {
    pub(crate) id: DisplayIdentifierInner,
    pub(crate) content: LogicalDisplayUpdateContent,
}

/// Requested changes to logical display state.
///
/// Linux currently does not support logical display updates.
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

impl From<DisplayUpdateInner> for Option<LogicalDisplayUpdate> {
    fn from(value: DisplayUpdateInner) -> Self {
        value.logical.map(|content| LogicalDisplayUpdate {
            id: value.id,
            content,
        })
    }
}
