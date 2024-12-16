use tracing::instrument;
use windows::Win32::{
    Devices::Display::{
        DisplayConfigGetDeviceInfo, DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
        DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_TARGET_DEVICE_NAME,
    },
    Foundation::WIN32_ERROR,
    Graphics::Gdi::DISPLAYCONFIG_PATH_ACTIVE,
};

use crate::{
    display::DisplayUpdateInner,
    display_identifier::{DisplayIdentifier, DisplayIdentifierInner},
};

use super::{error::WindowsError, utils::try_utf16_cstring};

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct LogicalDisplayWindows {
    pub target: TargetDevice,
    pub is_enabled: bool,
}

impl LogicalDisplayWindows {
    pub fn id(&self) -> DisplayIdentifierInner {
        DisplayIdentifierInner {
            outer: DisplayIdentifier {
                name: Some(self.target.name.clone()),
                ..Default::default()
            },
            path: Some(self.target.path.clone()),
            ..Default::default()
        }
    }
    pub fn matches(&self, id: &DisplayIdentifierInner) -> bool {
        if let Some(ref name) = id.outer.name {
            if self.target.name.starts_with(name) {
                return false;
            }
        }

        if let Some(ref path) = id.path {
            if self.target.path.starts_with(path) {
                return false;
            }
        }

        true
    }
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct TargetDevice {
    pub name: String,
    pub path: String,
    pub source_id: u32,
}

impl TryFrom<DISPLAYCONFIG_PATH_INFO> for LogicalDisplayWindows {
    type Error = WindowsError;

    fn try_from(value: DISPLAYCONFIG_PATH_INFO) -> Result<Self, Self::Error> {
        let mut target_device_name = DISPLAYCONFIG_TARGET_DEVICE_NAME {
            header: Default::default(),
            ..Default::default()
        };
        target_device_name.header.size =
            std::mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32;
        target_device_name.header.adapterId = value.targetInfo.adapterId;
        target_device_name.header.id = value.targetInfo.id;
        target_device_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME;

        WIN32_ERROR(unsafe { DisplayConfigGetDeviceInfo(&mut target_device_name.header) } as u32)
            .ok()?;

        let target = (value, target_device_name).try_into()?;
        let is_enabled = value.flags & DISPLAYCONFIG_PATH_ACTIVE == DISPLAYCONFIG_PATH_ACTIVE;
        Ok(Self { target, is_enabled })
    }
}

impl TryFrom<(DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_TARGET_DEVICE_NAME)> for TargetDevice {
    type Error = WindowsError;

    #[instrument(ret, skip_all, level = "debug")]
    fn try_from(
        (path_info, target): (DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_TARGET_DEVICE_NAME),
    ) -> Result<Self, Self::Error> {
        let Ok(name) = try_utf16_cstring(&target.monitorFriendlyDeviceName) else {
            return Err(WindowsError::InvalidUtf16 {
                data: target.monitorFriendlyDeviceName.to_vec(),
                origin: "monitorFriendlyDeviceName".to_string(),
            });
        };
        let Ok(path) = try_utf16_cstring(&target.monitorDevicePath) else {
            return Err(WindowsError::InvalidUtf16 {
                data: target.monitorFriendlyDeviceName.to_vec(),
                origin: "monitorDevicePath".to_string(),
            });
        };

        if name.is_empty() || path.is_empty() {
            return Err(WindowsError::Other {
                message: format!(
                    "monitorFriendlyDeviceName ({name}) or monitorDevicePath ({path}) empty"
                ),
            });
        }

        Ok(Self {
            name,
            path,
            source_id: path_info.sourceInfo.id,
        })
    }
}
