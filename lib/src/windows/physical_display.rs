use edid_rs::EDID;

use crate::display::{DisplayIdentifierInner, DisplayUpdateInner};

use super::physical_manager::PhysicalDisplayQueryError;

#[derive(Debug, Clone, Default)]
pub struct PhysicalDisplayUpdate {
    pub id: DisplayIdentifierInner,
    pub content: PhysicalDisplayUpdateContent,
}

#[derive(Debug, Clone, Default)]
pub struct PhysicalDisplayUpdateContent {
    pub brightness: Option<u32>,
}

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

#[derive(Debug)]
pub struct PhysicalDisplayWindows {
    /// \\?\DISPLAY#LEN66F9#7&289ec95a&0&UID264
    pub(crate) path: String,
    /// E.g: "Lenovo Y32p-30"
    pub(crate) name: String,
    pub(crate) serial_number: String,
}

impl TryFrom<(String, EDID)> for PhysicalDisplayWindows {
    type Error = PhysicalDisplayQueryError;
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
            .ok_or_else(|| PhysicalDisplayQueryError::EDIDInvalid {
                message: "no monitor name found".to_string(),
                key: path.clone(),
            })?;
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
            .ok_or_else(|| PhysicalDisplayQueryError::EDIDInvalid {
                message: "no serial number found".to_string(),
                key: path.clone(),
            })?;
        Ok(Self {
            path,
            name,
            serial_number,
        })
    }
}

impl From<DisplayUpdateInner> for Option<PhysicalDisplayUpdate> {
    fn from(value: DisplayUpdateInner) -> Self {
        value.physical.map(|content| PhysicalDisplayUpdate {
            id: value.id,
            content,
        })
    }
}
