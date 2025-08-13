use crate::{
    display_identifier::{DisplayIdentifier, DisplayIdentifierInner},
    logical_display::LogicalDisplayUpdateContent,
    physical_display::PhysicalDisplayUpdateContent,
    windows::{
        logical_display::{LogicalDisplayWindows, LogicalDisplayWindowsMetadata},
        physical_display::{PhysicalDisplayWindows, PhysicalDisplayWindowsMetadata},
    },
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DisplayMetadata {
    pub physical: Option<PhysicalDisplayWindowsMetadata>,
    // In the future this should allow for more than one, but that future is
    // not now.
    pub logical: LogicalDisplayWindowsMetadata,
}

#[derive(Debug)]
pub struct Display {
    // Displays that do not support DDC/CI will not have a physical display.
    pub physical: Option<PhysicalDisplayWindows>,
    // In the future this should allow for more than one, but that future is
    // not now.
    pub logical: LogicalDisplayWindows,
}

impl DisplayMetadata {
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
            source_id: Some(self.logical.source_id),
            gdi_device_id: Some(self.logical.gdi_device_id),
        }
    }
}

impl Display {
    pub fn metadata(&self) -> DisplayMetadata {
        DisplayMetadata {
            physical: self
                .physical
                .as_ref()
                .map(|physical| physical.metadata.clone()),
            logical: self.logical.metadata.clone(),
        }
    }

    pub fn id(&self) -> DisplayIdentifierInner {
        self.metadata().id()
    }
}

#[derive(Debug, Default, Clone)]
pub struct DisplayUpdate {
    pub id: DisplayIdentifier,
    pub logical: Option<LogicalDisplayUpdateContent>,
    pub physical: Option<PhysicalDisplayUpdateContent>,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct DisplayUpdateInner {
    pub(crate) id: DisplayIdentifierInner,
    pub(crate) logical: Option<LogicalDisplayUpdateContent>,
    pub(crate) physical: Option<PhysicalDisplayUpdateContent>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Brightness(u8);

impl Brightness {
    pub const fn new(value: u8) -> Self {
        if value > 100 {
            // TODO Remove
            panic!("Brightness needs to be between 0 and 100");
        }
        Self(value)
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}
