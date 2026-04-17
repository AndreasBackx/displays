use displays_types::DisplayIdentifierInner;

use crate::PhysicalDisplayUpdateContent;

/// Requested changes to one physical display.
#[derive(Debug, Default, Clone)]
pub struct PhysicalDisplayUpdate {
    pub id: DisplayIdentifierInner,
    pub content: PhysicalDisplayUpdateContent,
}
