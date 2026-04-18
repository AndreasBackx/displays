use crate::{LogicalDisplayMetadata, LogicalDisplayState};

/// A logical display and its current state.
#[derive(Debug, Clone, Default, PartialOrd, Ord, PartialEq, Eq)]
pub struct LogicalDisplay {
    /// Logical display metadata.
    pub metadata: LogicalDisplayMetadata,
    /// Current logical display state.
    pub state: LogicalDisplayState,
}
