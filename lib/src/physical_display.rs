use crate::{display::DisplayUpdateInner, display_identifier::DisplayIdentifierInner};

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
