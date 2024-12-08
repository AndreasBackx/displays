use crate::{
    logical::windows::display::{LogicalDisplayUpdateContent, LogicalDisplayWindows},
    physical::windows::display::{PhysicalDisplayUpdateContent, PhysicalDisplayWindows},
};

#[derive(Debug, Default, Clone)]
pub struct DisplayIdentifier {
    pub name: Option<String>,
    pub serial_number: Option<String>,
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
    pub fn id(&self) -> DisplayIdentifier {
        DisplayIdentifier {
            name: Some(self.physical.name.clone()),
            serial_number: Some(self.physical.serial_number.clone()),
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
