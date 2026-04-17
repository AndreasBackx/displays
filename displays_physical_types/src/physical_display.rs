use crate::{PhysicalDisplayMetadata, PhysicalDisplayState};

/// A physical monitor and its current state.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalDisplay {
    /// Physical monitor metadata.
    pub metadata: PhysicalDisplayMetadata,
    /// Current physical monitor state.
    pub state: PhysicalDisplayState,
}
