use displays_types::DisplayIdentifierInner;

use crate::LogicalDisplayUpdateContent;

/// Requested changes to one logical display.
#[derive(Debug, Clone)]
pub struct LogicalDisplayUpdate {
    pub id: DisplayIdentifierInner,
    pub content: LogicalDisplayUpdateContent,
}
