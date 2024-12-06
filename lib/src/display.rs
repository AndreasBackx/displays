use crate::{
    logical_display::{LogicalDisplay, LogicalDisplayWindows},
    physical_display::PhysicalDisplay,
};

pub struct DisplayIdentifier {
    pub name: Option<String>,
    pub serial_number: Option<String>,
}

pub struct Display {
    id: DisplayIdentifier,
    physical: PhysicalDisplay,
    // In the future this should allow for more than one, but that future is
    // not now.
    logical: LogicalDisplayWindows,
}
