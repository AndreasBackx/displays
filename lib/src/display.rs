use tracing::instrument;

use crate::windows::{
    logical_display::{LogicalDisplayUpdateContent, LogicalDisplayWindows},
    physical_display::{PhysicalDisplayUpdateContent, PhysicalDisplayWindows},
};

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DisplayIdentifier {
    pub name: Option<String>,
    pub serial_number: Option<String>,
}

impl DisplayIdentifier {
    #[instrument(ret, level = "debug")]
    pub fn is_subset(&self, other: &DisplayIdentifier) -> bool {
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
pub struct DisplayIdentifierInner {
    pub outer: DisplayIdentifier,
    pub path: Option<String>,
    pub source_id: Option<u32>,
}

#[derive(Debug)]
pub struct Display {
    // id: DisplayIdentifier,
    pub physical: PhysicalDisplayWindows,
    // In the future this should allow for more than one, but that future is
    // not now.
    pub logical: LogicalDisplayWindows,
}

impl Display {
    pub fn id(&self) -> DisplayIdentifierInner {
        DisplayIdentifierInner {
            outer: DisplayIdentifier {
                name: Some(self.physical.name.clone()),
                serial_number: Some(self.physical.serial_number.clone()),
            },
            path: Some(self.physical.path.clone()),
            source_id: Some(self.logical.target.source_id),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct DisplayUpdate {
    pub id: DisplayIdentifier,
    pub logical: Option<LogicalDisplayUpdateContent>,
    pub physical: Option<PhysicalDisplayUpdateContent>,
}

#[derive(Debug, Default, Clone)]
pub struct DisplayUpdateInner {
    pub id: DisplayIdentifierInner,
    pub logical: Option<LogicalDisplayUpdateContent>,
    pub physical: Option<PhysicalDisplayUpdateContent>,
}
