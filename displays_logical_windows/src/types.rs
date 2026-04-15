use displays_windows_common::types::{
    DisplayIdentifier, DisplayIdentifierInner, Orientation, PixelFormat, Point,
};
use tracing::instrument;
use windows::Win32::{
    Devices::Display::{
        DisplayConfigGetDeviceInfo, DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
        DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME, DISPLAYCONFIG_PATH_INFO,
        DISPLAYCONFIG_PIXELFORMAT, DISPLAYCONFIG_PIXELFORMAT_16BPP,
        DISPLAYCONFIG_PIXELFORMAT_24BPP, DISPLAYCONFIG_PIXELFORMAT_32BPP,
        DISPLAYCONFIG_PIXELFORMAT_8BPP, DISPLAYCONFIG_PIXELFORMAT_NONGDI, DISPLAYCONFIG_ROTATION,
        DISPLAYCONFIG_ROTATION_IDENTITY, DISPLAYCONFIG_ROTATION_ROTATE180,
        DISPLAYCONFIG_ROTATION_ROTATE270, DISPLAYCONFIG_ROTATION_ROTATE90,
        DISPLAYCONFIG_SOURCE_DEVICE_NAME, DISPLAYCONFIG_TARGET_DEVICE_NAME,
    },
    Foundation::{POINTL, WIN32_ERROR},
    Graphics::Gdi::DISPLAYCONFIG_PATH_ACTIVE,
};

use crate::{error::QueryError, manager::PathInfo};
use displays_windows_common::{error::WindowsError, utils, utils::try_utf16_cstring};

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct LogicalDisplayMetadata {
    pub name: String,
    pub path: String,
    pub gdi_device_id: Option<u32>,
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Default)]
pub struct LogicalDisplayState {
    pub is_enabled: bool,
    pub orientation: Orientation,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub pixel_format: Option<PixelFormat>,
    pub position: Option<Point>,
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct LogicalDisplay {
    pub metadata: LogicalDisplayMetadata,
    pub state: LogicalDisplayState,
}

#[derive(Debug, Clone)]
pub struct LogicalDisplayUpdate {
    pub id: DisplayIdentifierInner,
    pub content: LogicalDisplayUpdateContent,
}

#[derive(Debug, Default, Clone)]
pub struct LogicalDisplayUpdateContent {
    pub is_enabled: Option<bool>,
    pub orientation: Option<Orientation>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub pixel_format: Option<PixelFormat>,
    pub position: Option<Point>,
}

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

impl From<&DISPLAYCONFIG_PIXELFORMAT> for PixelFormat {
    fn from(value: &DISPLAYCONFIG_PIXELFORMAT) -> Self {
        match *value {
            DISPLAYCONFIG_PIXELFORMAT_8BPP => PixelFormat::BPP8,
            DISPLAYCONFIG_PIXELFORMAT_16BPP => PixelFormat::BPP16,
            DISPLAYCONFIG_PIXELFORMAT_24BPP => PixelFormat::BPP24,
            DISPLAYCONFIG_PIXELFORMAT_32BPP => PixelFormat::BPP32,
            DISPLAYCONFIG_PIXELFORMAT_NONGDI => PixelFormat::NONGDI,
            _ => unimplemented!("Nonexistent pixel format."),
        }
    }
}

impl From<&PixelFormat> for DISPLAYCONFIG_PIXELFORMAT {
    fn from(value: &PixelFormat) -> Self {
        match *value {
            PixelFormat::BPP8 => DISPLAYCONFIG_PIXELFORMAT_8BPP,
            PixelFormat::BPP16 => DISPLAYCONFIG_PIXELFORMAT_16BPP,
            PixelFormat::BPP24 => DISPLAYCONFIG_PIXELFORMAT_24BPP,
            PixelFormat::BPP32 => DISPLAYCONFIG_PIXELFORMAT_32BPP,
            PixelFormat::NONGDI => DISPLAYCONFIG_PIXELFORMAT_NONGDI,
        }
    }
}

impl From<&DISPLAYCONFIG_ROTATION> for Orientation {
    fn from(value: &DISPLAYCONFIG_ROTATION) -> Self {
        match *value {
            DISPLAYCONFIG_ROTATION_IDENTITY => Orientation::Landscape,
            DISPLAYCONFIG_ROTATION_ROTATE90 => Orientation::Portrait,
            DISPLAYCONFIG_ROTATION_ROTATE180 => Orientation::LandscapeFlipped,
            DISPLAYCONFIG_ROTATION_ROTATE270 => Orientation::PortraitFlipped,
            _ => unimplemented!("Nonexistent display orientation."),
        }
    }
}

impl From<&Orientation> for DISPLAYCONFIG_ROTATION {
    fn from(value: &Orientation) -> Self {
        match *value {
            Orientation::Landscape => DISPLAYCONFIG_ROTATION_IDENTITY,
            Orientation::Portrait => DISPLAYCONFIG_ROTATION_ROTATE90,
            Orientation::LandscapeFlipped => DISPLAYCONFIG_ROTATION_ROTATE180,
            Orientation::PortraitFlipped => DISPLAYCONFIG_ROTATION_ROTATE270,
        }
    }
}

impl From<&POINTL> for Point {
    fn from(value: &POINTL) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<&Point> for POINTL {
    fn from(value: &Point) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
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
