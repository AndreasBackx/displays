use crate::{
    display::{Brightness, DisplayUpdateInner},
    display_identifier::DisplayIdentifierInner,
};

/// Stable metadata describing a physical monitor.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalDisplayMetadata {
    /// Platform-specific monitor path.
    pub path: String,
    /// Human-readable monitor name.
    pub name: String,
    /// Monitor serial number.
    pub serial_number: String,
}

/// The current physical monitor state.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalDisplayState {
    /// Current brightness percentage.
    pub brightness: Brightness,
    /// Current OS scale factor percentage.
    pub scale_factor: i32,
}

/// A physical monitor and its current state.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalDisplay {
    /// Physical monitor metadata.
    pub metadata: PhysicalDisplayMetadata,
    /// Current physical monitor state.
    pub state: PhysicalDisplayState,
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub(crate) struct PhysicalDisplayUpdate {
    pub(crate) id: DisplayIdentifierInner,
    pub(crate) content: PhysicalDisplayUpdateContent,
}

/// Requested changes to physical monitor state.
#[derive(Debug, Clone, Default)]
pub struct PhysicalDisplayUpdateContent {
    /// Requested brightness percentage in the inclusive range `0..=100`.
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
