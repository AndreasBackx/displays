pub use displays_physical_types::{
    Brightness, PhysicalDisplay, PhysicalDisplayMetadata, PhysicalDisplayState,
    PhysicalDisplayUpdateContent,
};

/// A user-facing identifier used to match one or more Linux physical displays.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalDisplayIdentifier {
    /// Human-readable display name when available.
    pub name: Option<String>,
    /// Physical serial number when available.
    pub serial_number: Option<String>,
}

/// Requested changes to Linux physical display state.
#[derive(Debug, Default, Clone)]
pub struct PhysicalDisplayUpdate {
    /// The user-facing identifier used to match displays.
    pub id: PhysicalDisplayIdentifier,
    /// Requested physical display changes.
    pub content: PhysicalDisplayUpdateContent,
}

impl PhysicalDisplayIdentifier {
    /// Returns `true` when this identifier is a subset of `other`.
    pub fn is_subset(&self, other: &PhysicalDisplayIdentifier) -> bool {
        if let Some(ref name) = self.name {
            if let Some(ref other_name) = other.name {
                if name != other_name {
                    return false;
                }
            }
        }

        if let Some(ref serial_number) = self.serial_number {
            if let Some(ref other_serial_number) = other.serial_number {
                if serial_number != other_serial_number {
                    return false;
                }
            }
        }
        true
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PhysicalDisplayIdentifierInner {
    pub(crate) outer: PhysicalDisplayIdentifier,
    pub(crate) path: Option<String>,
}

fn physical_display_id(display: &PhysicalDisplay) -> PhysicalDisplayIdentifierInner {
    PhysicalDisplayIdentifierInner {
        outer: PhysicalDisplayIdentifier {
            name: Some(display.metadata.name.clone()),
            serial_number: Some(display.metadata.serial_number.clone()),
        },
        path: Some(display.metadata.path.clone()),
    }
}

#[derive(Debug, Clone)]
pub(crate) struct DisplayHandle {
    pub(crate) metadata: PhysicalDisplayMetadata,
    pub(crate) state: PhysicalDisplayState,
    pub(crate) backend: Backend,
}

impl DisplayHandle {
    pub(crate) fn display(&self) -> PhysicalDisplay {
        PhysicalDisplay {
            metadata: self.metadata.clone(),
            state: self.state.clone(),
        }
    }

    pub(crate) fn id(&self) -> PhysicalDisplayIdentifierInner {
        physical_display_id(&self.display())
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Backend {
    Ddc { display_index: usize },
    Backlight { path: String },
}

#[derive(Debug, Clone)]
pub(crate) struct DdcApplyUpdate {
    pub(crate) id: PhysicalDisplayIdentifierInner,
    pub(crate) content: PhysicalDisplayUpdateContent,
    pub(crate) display_index: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct BacklightApplyUpdate {
    pub(crate) id: PhysicalDisplayIdentifierInner,
    pub(crate) content: PhysicalDisplayUpdateContent,
    pub(crate) path: String,
}
