use anyhow::Context;
use edid_rs::EDID;

use crate::{
    display::{DisplayIdentifier, DisplayUpdate},
    logical::windows::display::LogicalDisplayUpdate,
};

#[derive(Debug, Clone, Default)]
pub struct PhysicalDisplayUpdate {
    pub id: DisplayIdentifier,
    pub content: PhysicalDisplayUpdateContent,
}

#[derive(Debug, Clone, Default)]
pub struct PhysicalDisplayUpdateContent {
    pub brightness: Option<i8>,
}

pub struct Brightness(u8);

impl Brightness {
    pub const fn new(value: u8) -> Self {
        if value > 100 || value < 0 {
            // TODO Remove
            panic!("Brightness needs to be between 0 and 100");
        }
        Self(value)
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}

#[derive(Debug)]
pub struct PhysicalDisplayWindows {
    /// \\?\DISPLAY#LEN66F9#7&289ec95a&0&UID264
    pub(crate) path: String,
    /// E.g: "Lenovo Y32p-30"
    pub(crate) name: String,
    pub(crate) serial_number: String,
}
impl TryFrom<(String, EDID)> for PhysicalDisplayWindows {
    type Error = anyhow::Error;
    fn try_from((path, edid): (String, EDID)) -> Result<Self, Self::Error> {
        let name = edid
            .descriptors
            .0
            .iter()
            .filter_map(|descriptor| match descriptor {
                edid_rs::MonitorDescriptor::MonitorName(name) => Some(name),
                _ => None,
            })
            .nth(0)
            .cloned()
            .context("no monitor name found")?;
        let serial_number = edid
            .descriptors
            .0
            .iter()
            .filter_map(|descriptor| match descriptor {
                edid_rs::MonitorDescriptor::SerialNumber(serial_number) => Some(serial_number),
                _ => None,
            })
            .nth(0)
            .cloned()
            .context("no serial number found")?;
        Ok(Self {
            path,
            name,
            serial_number,
        })
    }
}

impl From<DisplayUpdate> for Option<PhysicalDisplayUpdate> {
    fn from(value: DisplayUpdate) -> Self {
        value.physical.map(|content| PhysicalDisplayUpdate {
            id: value.id,
            content,
        })
    }
}
