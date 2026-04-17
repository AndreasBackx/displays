use crate::{
    display_identifier::{DisplayIdentifier, DisplayIdentifierInner},
    logical_display::{LogicalDisplay, LogicalDisplayMetadata, LogicalDisplayUpdateContent},
    physical_display::{PhysicalDisplay, PhysicalDisplayMetadata, PhysicalDisplayUpdateContent},
};

/// Metadata used to identify a display across logical and physical backends.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DisplayMetadata {
    /// Physical monitor metadata when the display exposes a physical interface.
    pub physical: Option<PhysicalDisplayMetadata>,
    // In the future this should allow for more than one, but that future is
    // not now.
    /// Logical display metadata reported by the operating system.
    pub logical: LogicalDisplayMetadata,
}

/// A display with logical state and, when available, physical monitor state.
#[derive(Debug, Clone)]
pub struct Display {
    // Displays that do not support DDC/CI will not have a physical display.
    /// Physical monitor state when a physical monitor could be matched.
    pub physical: Option<PhysicalDisplay>,
    // In the future this should allow for more than one, but that future is
    // not now.
    /// Logical display state reported by the platform backend.
    pub logical: LogicalDisplay,
}

impl DisplayMetadata {
    /// Builds the best-effort identifier for this display metadata.
    pub fn id(&self) -> DisplayIdentifierInner {
        DisplayIdentifierInner {
            outer: DisplayIdentifier {
                name: self
                    .physical
                    .as_ref()
                    .map(|physical| physical.name.clone())
                    .or_else(|| Some(self.logical.name.clone())),
                serial_number: self
                    .physical
                    .as_ref()
                    .map(|physical| physical.serial_number.clone()),
            },
            path: self
                .physical
                .as_ref()
                .map(|physical| physical.path.clone())
                .or_else(|| Some(self.logical.path.clone())),
            gdi_device_id: self.logical.gdi_device_id,
        }
    }
}

impl Display {
    /// Returns the metadata describing this display.
    pub fn metadata(&self) -> DisplayMetadata {
        DisplayMetadata {
            physical: self
                .physical
                .as_ref()
                .map(|physical| physical.metadata.clone()),
            logical: self.logical.metadata.clone(),
        }
    }

    /// Returns the best-effort identifier for this display.
    pub fn id(&self) -> DisplayIdentifierInner {
        self.metadata().id()
    }
}

/// A request to update one display.
///
/// Logical and physical updates can be combined when the target platform supports
/// them. On Linux, logical updates are currently unsupported.
#[derive(Debug, Default, Clone)]
pub struct DisplayUpdate {
    /// The user-facing identifier used to match displays.
    pub id: DisplayIdentifier,
    /// Requested logical display changes.
    pub logical: Option<LogicalDisplayUpdateContent>,
    /// Requested physical monitor changes.
    pub physical: Option<PhysicalDisplayUpdateContent>,
}

#[derive(Debug, Default, Clone)]
#[allow(dead_code)]
pub(crate) struct DisplayUpdateInner {
    pub(crate) id: DisplayIdentifierInner,
    pub(crate) logical: Option<LogicalDisplayUpdateContent>,
    pub(crate) physical: Option<PhysicalDisplayUpdateContent>,
}
