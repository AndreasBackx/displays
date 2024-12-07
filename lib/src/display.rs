use crate::{
    logical_display::{LogicalDisplay, LogicalDisplayWindows},
    physical_display::{PhysicalDisplay, PhysicalDisplayWindows},
};

#[derive(Debug)]
pub struct DisplayIdentifier {
    pub name: Option<String>,
    pub serial_number: Option<String>,
}

#[derive(Debug)]
pub struct Display {
    // id: DisplayIdentifier,
    pub physical: PhysicalDisplayWindows,
    // In the future this should allow for more than one, but that future is
    // not now.
    pub logical: LogicalDisplayWindows,
}
