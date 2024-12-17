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
    pub physical: PhysicalDisplayWindowsMetadata,
    // In the future this should allow for more than one, but that future is
    // not now.
    pub logical: LogicalDisplayWindowsMetadata,
}

#[derive(Debug)]
pub struct Display {
    pub physical: PhysicalDisplayWindows,
    // In the future this should allow for more than one, but that future is
    // not now.
    pub logical: LogicalDisplayWindows,
}

impl DisplayMetadata {
    pub fn id(&self) -> DisplayIdentifierInner {
        DisplayIdentifierInner {
            outer: DisplayIdentifier {
                name: Some(self.physical.name.clone()),
                serial_number: Some(self.physical.serial_number.clone()),
            },
            path: Some(self.physical.path.clone()),
            source_id: Some(self.logical.source_id),
        }
    }
}

impl Display {
    pub fn metadata(&self) -> DisplayMetadata {
        DisplayMetadata {
            physical: self.physical.metadata.clone(),
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
