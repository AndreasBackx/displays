use displays_types::{DisplayIdentifier, DisplayIdentifierInner, Orientation, PixelFormat, Point};
use displays_logical_types::{
    LogicalDisplay, LogicalDisplayMetadata, LogicalDisplayState, LogicalDisplayUpdate,
    LogicalDisplayUpdateContent,
};
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

use crate::manager::PathInfo;
use displays_windows_common::{error::WindowsError, utils, utils::try_utf16_cstring};


impl LogicalDisplay {
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

impl TryFrom<&PathInfo> for LogicalDisplay {
    type Error = WindowsError;

    fn try_from(path_info: &PathInfo) -> Result<Self, Self::Error> {
        let path = &path_info.path;

        let mut logical_display: LogicalDisplay = path.try_into()?;

        if let Some(mode_source) = path_info.mode_source {
            let source_mode = unsafe { mode_source.Anonymous.sourceMode };

            logical_display.state.width = Some(source_mode.width);
            logical_display.state.height = Some(source_mode.height);
            logical_display.state.pixel_format = Some((&source_mode.pixelFormat).into());
            logical_display.state.position = Some((&source_mode.position).into());

            tracing::warn!("source_mode = {:?}", source_mode);
        }

        Ok(logical_display)
    }
}

impl TryFrom<&DISPLAYCONFIG_PATH_INFO> for LogicalDisplay {
    type Error = WindowsError;

    fn try_from(path: &DISPLAYCONFIG_PATH_INFO) -> Result<Self, Self::Error> {
        let is_enabled = path.flags & DISPLAYCONFIG_PATH_ACTIVE == DISPLAYCONFIG_PATH_ACTIVE;
        let orientation = (&path.targetInfo.rotation).into();

        let mut target_device_name = DISPLAYCONFIG_TARGET_DEVICE_NAME {
            header: Default::default(),
            ..Default::default()
        };
        target_device_name.header.size =
            std::mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32;
        target_device_name.header.adapterId = path.targetInfo.adapterId;
        target_device_name.header.id = path.targetInfo.id;
        target_device_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME;

        WIN32_ERROR(unsafe { DisplayConfigGetDeviceInfo(&mut target_device_name.header) } as u32)
            .ok()?;

        let mut source_device_name: DISPLAYCONFIG_SOURCE_DEVICE_NAME =
            unsafe { std::mem::zeroed() };
        source_device_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME;
        source_device_name.header.size =
            std::mem::size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32;
        source_device_name.header.adapterId = path.sourceInfo.adapterId;
        source_device_name.header.id = path.sourceInfo.id;

        WIN32_ERROR(unsafe { DisplayConfigGetDeviceInfo(&mut source_device_name.header) } as u32)
            .ok()?;

        let target = (target_device_name, source_device_name).try_into()?;
        Ok(Self {
            metadata: target,
            state: LogicalDisplayState {
                is_enabled,
                orientation,
                ..Default::default()
            },
        })
    }
}

impl
    TryFrom<(
        DISPLAYCONFIG_TARGET_DEVICE_NAME,
        DISPLAYCONFIG_SOURCE_DEVICE_NAME,
    )> for LogicalDisplayMetadata
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
            gdi_device_id: Some(gdi_device_id),
        })
    }
}
