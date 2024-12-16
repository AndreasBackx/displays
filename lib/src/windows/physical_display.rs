use edid_rs::EDID;

use super::physical_manager::PhysicalDisplayQueryError;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
