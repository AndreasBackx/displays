use crate::{display::DisplayUpdateInner, display_identifier::DisplayIdentifierInner};
pub use displays_physical_types::{
    PhysicalDisplay, PhysicalDisplayMetadata, PhysicalDisplayState, PhysicalDisplayUpdateContent,
};

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub(crate) struct PhysicalDisplayUpdate {
    pub(crate) id: DisplayIdentifierInner,
    pub(crate) content: PhysicalDisplayUpdateContent,
}

impl From<DisplayUpdateInner> for Option<PhysicalDisplayUpdate> {
    fn from(value: DisplayUpdateInner) -> Self {
        value.physical.map(|content| PhysicalDisplayUpdate {
            id: value.id,
            content,
        })
    }
}
