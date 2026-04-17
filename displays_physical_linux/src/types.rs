use displays_types::{DisplayIdentifier, DisplayIdentifierInner};

use displays_physical_types::{
    PhysicalDisplay, PhysicalDisplayMetadata, PhysicalDisplayState, PhysicalDisplayUpdateContent,
};

/// Requested changes to Linux physical display state.
#[derive(Debug, Default, Clone)]
pub struct PhysicalDisplayUpdate {
    /// The user-facing identifier used to match displays.
    pub id: DisplayIdentifier,
    /// Requested physical display changes.
    pub content: PhysicalDisplayUpdateContent,
}

fn physical_display_id(display: &PhysicalDisplay) -> DisplayIdentifierInner {
    DisplayIdentifierInner {
        outer: DisplayIdentifier {
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

    pub(crate) fn id(&self) -> DisplayIdentifierInner {
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
    pub(crate) id: DisplayIdentifierInner,
    pub(crate) content: PhysicalDisplayUpdateContent,
    pub(crate) display_index: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct BacklightApplyUpdate {
    pub(crate) id: DisplayIdentifierInner,
    pub(crate) content: PhysicalDisplayUpdateContent,
    pub(crate) path: String,
}

pub(crate) fn remaining_update(
    id: DisplayIdentifierInner,
    brightness: u32,
) -> PhysicalDisplayUpdate {
    PhysicalDisplayUpdate {
        id: id.outer,
        content: PhysicalDisplayUpdateContent {
            brightness: Some(brightness),
        },
    }
}
