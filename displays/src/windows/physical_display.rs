use edid_rs::EDID;

use crate::{
    display::Brightness,
    physical_display::{PhysicalDisplay, PhysicalDisplayMetadata, PhysicalDisplayState},
};

use super::physical_manager::PhysicalDisplayQueryError;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalDisplayWindowsMetadata {
    /// \\?\DISPLAY#LEN66F9#7&289ec95a&0&UID264
    pub path: String,
    /// E.g: "Lenovo Y32p-30"
    pub name: String,
    pub serial_number: String,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalDisplayWindowsState {
    pub brightness: Brightness,
    pub scale_factor: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalDisplayWindows {
    pub metadata: PhysicalDisplayWindowsMetadata,
    pub state: PhysicalDisplayWindowsState,
}

impl From<PhysicalDisplayWindowsMetadata> for PhysicalDisplayMetadata {
    fn from(value: PhysicalDisplayWindowsMetadata) -> Self {
        Self {
            path: value.path,
            name: value.name,
            serial_number: value.serial_number,
        }
    }
}

impl From<PhysicalDisplayWindowsState> for PhysicalDisplayState {
    fn from(value: PhysicalDisplayWindowsState) -> Self {
        Self {
            brightness: value.brightness,
            scale_factor: value.scale_factor,
        }
    }
}

impl From<PhysicalDisplayWindows> for PhysicalDisplay {
    fn from(value: PhysicalDisplayWindows) -> Self {
        Self {
            metadata: value.metadata.into(),
            state: value.state.into(),
        }
    }
}

impl TryFrom<(String, EDID)> for PhysicalDisplayWindowsMetadata {
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
