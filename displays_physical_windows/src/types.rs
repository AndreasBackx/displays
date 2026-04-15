use edid_rs::EDID;

use crate::error::QueryError;
use displays_windows_common::types::{Brightness, DisplayIdentifierInner};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalDisplayMetadata {
    pub path: String,
    pub name: String,
    pub serial_number: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalDisplayState {
    pub brightness: Brightness,
    pub scale_factor: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalDisplay {
    pub metadata: PhysicalDisplayMetadata,
    pub state: PhysicalDisplayState,
}

#[derive(Debug, Clone, Default)]
pub struct PhysicalDisplayUpdate {
    pub id: DisplayIdentifierInner,
    pub content: PhysicalDisplayUpdateContent,
}

#[derive(Debug, Clone, Default)]
pub struct PhysicalDisplayUpdateContent {
    pub brightness: Option<u32>,
}

impl TryFrom<(String, EDID)> for PhysicalDisplayMetadata {
    type Error = QueryError;

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
            .ok_or_else(|| QueryError::EDIDInvalid {
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
            .ok_or_else(|| QueryError::EDIDInvalid {
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
