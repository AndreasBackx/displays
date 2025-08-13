use tracing::instrument;
use windows::Win32::{
    Devices::Display::{
        DisplayConfigGetDeviceInfo, DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
        DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME, DISPLAYCONFIG_PATH_INFO,
        DISPLAYCONFIG_SOURCE_DEVICE_NAME, DISPLAYCONFIG_TARGET_DEVICE_NAME,
    },
    Foundation::WIN32_ERROR,
    Graphics::Gdi::DISPLAYCONFIG_PATH_ACTIVE,
};

use crate::{
    display_identifier::{DisplayIdentifier, DisplayIdentifierInner},
    windows::utils,
};

use super::{error::WindowsError, utils::try_utf16_cstring};

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct LogicalDisplayWindows {
    pub metadata: LogicalDisplayWindowsMetadata,
    pub state: LogicalDisplayWindowsState,
}

impl LogicalDisplayWindows {
    pub fn id(&self) -> DisplayIdentifierInner {
        DisplayIdentifierInner {
            outer: DisplayIdentifier {
                name: Some(self.metadata.name.clone()),
                ..Default::default()
            },
            path: Some(self.metadata.path.clone()),
            ..Default::default()
        }
    }
    pub fn matches(&self, id: &DisplayIdentifierInner) -> bool {
        if let Some(ref name) = id.outer.name {
            if !self.metadata.name.starts_with(name) {
                return false;
            }
        }

        if let Some(ref path) = id.path {
            if !self.metadata.path.starts_with(path) {
                return false;
            }
        }

        true
    }
}

/// Wrapper for Windows target device metadata.
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct LogicalDisplayWindowsState {
    pub is_enabled: bool,
}

/// Wrapper for Windows target device metadata.
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct LogicalDisplayWindowsMetadata {
    pub name: String,
    pub path: String,
    pub gdi_device_id: u32,
}

impl TryFrom<DISPLAYCONFIG_PATH_INFO> for LogicalDisplayWindows {
    type Error = WindowsError;

    fn try_from(value: DISPLAYCONFIG_PATH_INFO) -> Result<Self, Self::Error> {
        let is_enabled = value.flags & DISPLAYCONFIG_PATH_ACTIVE == DISPLAYCONFIG_PATH_ACTIVE;

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

        let mut source_device_name: DISPLAYCONFIG_SOURCE_DEVICE_NAME =
            unsafe { std::mem::zeroed() };
        source_device_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME;
        source_device_name.header.size =
            std::mem::size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32;
        source_device_name.header.adapterId = value.sourceInfo.adapterId;
        source_device_name.header.id = value.sourceInfo.id;

        WIN32_ERROR(unsafe { DisplayConfigGetDeviceInfo(&mut source_device_name.header) } as u32)
            .ok()?;

        let target = (target_device_name, source_device_name).try_into()?;
        Ok(Self {
            metadata: target,
            state: LogicalDisplayWindowsState { is_enabled },
        })
    }
}

impl
    TryFrom<(
        DISPLAYCONFIG_TARGET_DEVICE_NAME,
        DISPLAYCONFIG_SOURCE_DEVICE_NAME,
    )> for LogicalDisplayWindowsMetadata
{
    type Error = WindowsError;

    #[instrument(ret, skip_all, level = "debug")]
    fn try_from(
        (target, source): (
            DISPLAYCONFIG_TARGET_DEVICE_NAME,
            DISPLAYCONFIG_SOURCE_DEVICE_NAME,
        ),
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

        let Ok(gdi_device_name) = try_utf16_cstring(&source.viewGdiDeviceName) else {
            return Err(WindowsError::InvalidUtf16 {
                data: target.monitorFriendlyDeviceName.to_vec(),
                origin: "monitorDevicePath".to_string(),
            });
        };

        let Some(gdi_device_id) = utils::get_gdi_device_id(&gdi_device_name) else {
            return Err(WindowsError::Other {
                message: format!("Could not get ID from GDI device name: {gdi_device_name}"),
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
            gdi_device_id,
        })
    }
}
