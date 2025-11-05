use crate::{
    display::DisplayUpdateInner,
    display_identifier::DisplayIdentifierInner,
    windows::logical_display::{Orientation, PixelFormat, Point},
};

#[derive(Debug, Clone)]
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
