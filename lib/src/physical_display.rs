use crate::{
    display::{Brightness, DisplayUpdateInner},
    display_identifier::DisplayIdentifierInner,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalDisplayMetadata {
    pub path: String,
    pub name: String,
    pub serial_number: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalDisplayState {
    pub brightness: Brightness,
    pub scale_factor: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalDisplay {
    pub metadata: PhysicalDisplayMetadata,
    pub state: PhysicalDisplayState,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct PhysicalDisplayUpdate {
    pub(crate) id: DisplayIdentifierInner,
    pub(crate) content: PhysicalDisplayUpdateContent,
}

#[derive(Debug, Clone, Default)]
pub struct PhysicalDisplayUpdateContent {
    pub brightness: Option<u32>,
}

impl From<DisplayUpdateInner> for Option<PhysicalDisplayUpdate> {
    fn from(value: DisplayUpdateInner) -> Self {
        value.physical.map(|content| PhysicalDisplayUpdate {
            id: value.id,
            content,
        })
    }
}
