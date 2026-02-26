use crate::{
    display::DisplayUpdateInner,
    display_identifier::DisplayIdentifierInner,
    types::{Orientation, PixelFormat, Point},
};

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct LogicalDisplayMetadata {
    pub name: String,
    pub path: String,
    pub gdi_device_id: Option<u32>,
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Default)]
pub struct LogicalDisplayState {
    pub is_enabled: bool,
    pub orientation: Orientation,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub pixel_format: Option<PixelFormat>,
    pub position: Option<Point>,
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct LogicalDisplay {
    pub metadata: LogicalDisplayMetadata,
    pub state: LogicalDisplayState,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "linux", allow(dead_code))]
pub(crate) struct LogicalDisplayUpdate {
    pub(crate) id: DisplayIdentifierInner,
    pub(crate) content: LogicalDisplayUpdateContent,
}

#[derive(Debug, Default, Clone)]
pub struct LogicalDisplayUpdateContent {
    pub is_enabled: Option<bool>,
    pub orientation: Option<Orientation>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub pixel_format: Option<PixelFormat>,
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
