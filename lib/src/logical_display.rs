use crate::{display::DisplayUpdateInner, display_identifier::DisplayIdentifierInner};

#[derive(Debug, Clone)]
pub(crate) struct LogicalDisplayUpdate {
    pub(crate) id: DisplayIdentifierInner,
    pub(crate) content: LogicalDisplayUpdateContent,
}

#[derive(Debug, Default, Clone)]
pub struct LogicalDisplayUpdateContent {
    pub is_enabled: Option<bool>,
}

impl From<DisplayUpdateInner> for Option<LogicalDisplayUpdate> {
    fn from(value: DisplayUpdateInner) -> Self {
        value.logical.map(|content| LogicalDisplayUpdate {
            id: value.id,
            content,
        })
    }
}
