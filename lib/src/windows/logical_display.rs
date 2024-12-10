use anyhow::bail;
use tracing::instrument;
use windows::Win32::{
    Devices::Display::{
        DisplayConfigGetDeviceInfo, DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
        DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_TARGET_DEVICE_NAME,
    },
    Foundation::ERROR_SUCCESS,
    Graphics::Gdi::DISPLAYCONFIG_PATH_ACTIVE,
};

use crate::display::{DisplayIdentifier, DisplayIdentifierInner, DisplayUpdateInner};

use super::utils::try_utf16_cstring;

#[derive(Debug, Default, Clone)]
pub struct LogicalDisplayUpdateContent {
    pub is_enabled: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct LogicalDisplayUpdate {
    pub id: DisplayIdentifierInner,
    pub content: LogicalDisplayUpdateContent,
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
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

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct TargetDevice {
    pub name: String,
    pub path: String,
    pub source_id: u32,
}

impl TryFrom<DISPLAYCONFIG_PATH_INFO> for LogicalDisplayWindows {
    type Error = anyhow::Error;

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

        let status = unsafe { DisplayConfigGetDeviceInfo(&mut target_device_name.header) };

        if status as u32 != ERROR_SUCCESS.0 {
            bail!("Failed to query device info. Error code: {:?}", status);
        }

        let target = (value, target_device_name).try_into()?;
        let is_enabled = value.flags & DISPLAYCONFIG_PATH_ACTIVE == DISPLAYCONFIG_PATH_ACTIVE;
        Ok(Self { target, is_enabled })
    }
}

impl TryFrom<(DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_TARGET_DEVICE_NAME)> for TargetDevice {
    type Error = anyhow::Error;

    #[instrument(ret, skip_all, level = "debug")]
    fn try_from(
        (path_info, target): (DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_TARGET_DEVICE_NAME),
    ) -> Result<Self, Self::Error> {
        let Ok(name) = try_utf16_cstring(&target.monitorFriendlyDeviceName) else {
            bail!("Invalid UTF16 passed for device name");
        };
        let Ok(path) = try_utf16_cstring(&target.monitorDevicePath) else {
            bail!("Invalid UTF16 passed for device path");
        };

        if name.is_empty() || path.is_empty() {
            bail!("Empty device name or path");
        }

        Ok(Self {
            name,
            path,
            source_id: path_info.sourceInfo.id,
        })
    }
}

impl From<DisplayUpdateInner> for Option<LogicalDisplayUpdate> {
    fn from(value: DisplayUpdateInner) -> Self {
        value.logical.map(|content| LogicalDisplayUpdate {
            id: value.id,
            content,
        })
    }
}
