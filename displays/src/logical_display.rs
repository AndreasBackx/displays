use crate::display::DisplayUpdateInner;
pub use displays_logical_types::{
    LogicalDisplay, LogicalDisplayMetadata, LogicalDisplayState, LogicalDisplayUpdate,
    LogicalDisplayUpdateContent,
};

impl From<DisplayUpdateInner> for Option<LogicalDisplayUpdate> {
    fn from(value: DisplayUpdateInner) -> Self {
        value.logical.map(|content| LogicalDisplayUpdate {
            id: value.id,
            content,
        })
    }
}
