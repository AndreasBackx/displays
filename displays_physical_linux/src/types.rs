/// A user-facing identifier used to match one or more Linux physical displays.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalDisplayIdentifier {
    /// Human-readable display name when available.
    pub name: Option<String>,
    /// Physical serial number when available.
    pub serial_number: Option<String>,
}

/// Stable metadata describing a Linux physical display.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalDisplayMetadata {
    /// Platform-specific display path.
    pub path: String,
    /// Human-readable display name.
    pub name: String,
    /// Physical serial number.
    pub serial_number: String,
}

/// The current Linux physical display state.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalDisplayState {
    /// Current brightness percentage.
    pub brightness_percent: u8,
    /// Current OS scale factor percentage.
    pub scale_factor: i32,
}

/// A Linux physical display and its current state.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalDisplay {
    /// Physical display metadata.
    pub metadata: PhysicalDisplayMetadata,
    /// Current physical display state.
    pub state: PhysicalDisplayState,
}

/// Requested changes to Linux physical display state.
#[derive(Debug, Default, Clone)]
pub struct PhysicalDisplayUpdate {
    /// The user-facing identifier used to match displays.
    pub id: PhysicalDisplayIdentifier,
    /// Requested brightness percentage in the inclusive range `0..=100`.
    pub brightness_percent: Option<u32>,
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

impl PhysicalDisplay {
    pub(crate) fn id(&self) -> PhysicalDisplayIdentifierInner {
        PhysicalDisplayIdentifierInner {
            outer: PhysicalDisplayIdentifier {
                name: Some(self.metadata.name.clone()),
                serial_number: Some(self.metadata.serial_number.clone()),
            },
            path: Some(self.metadata.path.clone()),
        }
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
        self.display().id()
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
    pub(crate) brightness_percent: Option<u32>,
    pub(crate) display_index: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct BacklightApplyUpdate {
    pub(crate) id: PhysicalDisplayIdentifierInner,
    pub(crate) brightness_percent: Option<u32>,
    pub(crate) path: String,
}
