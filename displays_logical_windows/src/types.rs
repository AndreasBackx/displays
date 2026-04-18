use displays_logical_types::{LogicalDisplay, LogicalDisplayMetadata, LogicalDisplayState};
use displays_types::{DisplayIdentifierInner, Size};
use tracing::instrument;
use windows::{
    core::BOOL,
    Win32::{
        Foundation::{LPARAM, RECT},
    Devices::Display::{
        DisplayConfigGetDeviceInfo, DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
        DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME, DISPLAYCONFIG_PATH_INFO,
        DISPLAYCONFIG_SOURCE_DEVICE_NAME, DISPLAYCONFIG_TARGET_DEVICE_NAME,
    },
        Foundation::WIN32_ERROR,
        Graphics::Gdi::{
            DISPLAYCONFIG_PATH_ACTIVE, EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR,
            MONITORINFO, MONITORINFOEXW,
        },
        UI::Shell::GetScaleFactorForMonitor,
    },
};

use crate::manager::PathInfo;
use displays_windows_common::{error::WindowsError, utils, utils::try_utf16_cstring};

pub(crate) fn logical_display_matches(display: &LogicalDisplay, id: &DisplayIdentifierInner) -> bool {
    if let Some(ref name) = id.outer.name {
        if !display.metadata.name.starts_with(name) {
            return false;
        }
    }

    if let Some(ref path) = id.path {
        if !display.metadata.path.starts_with(path) {
            return false;
        }
    }

    true
}

pub(crate) fn logical_display_from_path_info(path_info: &PathInfo) -> Result<LogicalDisplay, WindowsError> {
    let mut logical_display = logical_display_from_path(&path_info.path)?;

    if let Some(mode_source) = path_info.mode_source {
        let source_mode = unsafe { mode_source.Anonymous.sourceMode };
        let scale_ratio_milli = if logical_display.state.is_enabled {
            scale_ratio_milli_for_gdi_device_id(logical_display.metadata.gdi_device_id)?
        } else {
            None
        };
        let mode_size = Size {
            width: source_mode.width,
            height: source_mode.height,
        };

        logical_display.state.logical_size = Some(logical_size_from_mode_size(&mode_size, scale_ratio_milli));
        logical_display.state.mode_size = Some(mode_size);
        logical_display.state.scale_ratio_milli = scale_ratio_milli;
        logical_display.state.pixel_format = Some((&source_mode.pixelFormat).into());
        logical_display.state.mode_position = Some((&source_mode.position).into());
        logical_display.state.logical_position = None;

        tracing::warn!("source_mode = {:?}", source_mode);
    }

    Ok(logical_display)
}

fn logical_display_from_path(path: &DISPLAYCONFIG_PATH_INFO) -> Result<LogicalDisplay, WindowsError> {
    let is_enabled = path.flags & DISPLAYCONFIG_PATH_ACTIVE == DISPLAYCONFIG_PATH_ACTIVE;
    let orientation = (&path.targetInfo.rotation).into();

    let mut target_device_name = DISPLAYCONFIG_TARGET_DEVICE_NAME {
        header: Default::default(),
        ..Default::default()
    };
    target_device_name.header.size = std::mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32;
    target_device_name.header.adapterId = path.targetInfo.adapterId;
    target_device_name.header.id = path.targetInfo.id;
    target_device_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME;

    WIN32_ERROR(unsafe { DisplayConfigGetDeviceInfo(&mut target_device_name.header) } as u32)
        .ok()?;

    let mut source_device_name: DISPLAYCONFIG_SOURCE_DEVICE_NAME = unsafe { std::mem::zeroed() };
    source_device_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME;
    source_device_name.header.size = std::mem::size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32;
    source_device_name.header.adapterId = path.sourceInfo.adapterId;
    source_device_name.header.id = path.sourceInfo.id;

    WIN32_ERROR(unsafe { DisplayConfigGetDeviceInfo(&mut source_device_name.header) } as u32)
        .ok()?;

    let metadata = logical_display_metadata_from_device_names(target_device_name, source_device_name)?;
    Ok(LogicalDisplay {
        metadata,
        state: LogicalDisplayState {
            is_enabled,
            orientation,
            ..Default::default()
        },
    })
}

#[instrument(ret, skip_all, level = "debug")]
fn logical_display_metadata_from_device_names(
    target: DISPLAYCONFIG_TARGET_DEVICE_NAME,
    source: DISPLAYCONFIG_SOURCE_DEVICE_NAME,
) -> Result<LogicalDisplayMetadata, WindowsError> {
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
            message: format!("monitorFriendlyDeviceName ({name}) or monitorDevicePath ({path}) empty"),
        });
    }

    Ok(LogicalDisplayMetadata {
        name,
        path,
        manufacturer: None,
        model: None,
        serial_number: None,
        gdi_device_id: Some(gdi_device_id),
    })
}

fn logical_size_from_mode_size(mode_size: &Size, scale_ratio_milli: Option<u32>) -> Size {
    let Some(scale_ratio_milli) = scale_ratio_milli.filter(|scale| *scale != 0) else {
        return mode_size.clone();
    };

    Size {
        width: scaled_dimension(mode_size.width, scale_ratio_milli),
        height: scaled_dimension(mode_size.height, scale_ratio_milli),
    }
}

fn scaled_dimension(value: u32, scale_ratio_milli: u32) -> u32 {
    (((value as u64) * 1000) + ((scale_ratio_milli as u64) / 2))
        .checked_div(scale_ratio_milli as u64)
        .unwrap_or(value as u64) as u32
}

fn scale_ratio_milli_for_gdi_device_id(gdi_device_id: Option<u32>) -> Result<Option<u32>, WindowsError> {
    let Some(gdi_device_id) = gdi_device_id else {
        return Ok(None);
    };

    let mut monitor_infos = monitor_infos()?;
    let Some((_, hmonitor)) = monitor_infos
        .drain(..)
        .find(|(monitor_gdi_device_id, _)| *monitor_gdi_device_id == Some(gdi_device_id))
    else {
        return Ok(None);
    };

    let scale_factor = unsafe { GetScaleFactorForMonitor(hmonitor) }.map_err(WindowsError::from)?;
    Ok(Some((scale_factor.0 as u32) * 10))
}

fn monitor_infos() -> Result<Vec<(Option<u32>, HMONITOR)>, WindowsError> {
    unsafe extern "system" fn callback(
        monitor: HMONITOR,
        _hdc_monitor: HDC,
        _lprc: *mut RECT,
        userdata: LPARAM,
    ) -> BOOL {
        let monitors: &mut Vec<HMONITOR> = unsafe { &mut *(userdata.0 as *mut Vec<HMONITOR>) };
        monitors.push(monitor);
        BOOL::from(true)
    }

    let mut monitors = Vec::<HMONITOR>::new();
    let userdata = LPARAM(std::ptr::addr_of_mut!(monitors) as _);
    unsafe { EnumDisplayMonitors(None, None, Some(callback), userdata) }.ok()?;

    monitors
        .into_iter()
        .map(|monitor| Ok((gdi_device_id_for_monitor(monitor)?, monitor)))
        .collect()
}

fn gdi_device_id_for_monitor(hmonitor: HMONITOR) -> Result<Option<u32>, WindowsError> {
    let mut monitor_info = MONITORINFOEXW {
        monitorInfo: MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFOEXW>() as u32,
            ..Default::default()
        },
        ..Default::default()
    };

    let monitor_info_base = &mut monitor_info as *mut MONITORINFOEXW as *mut MONITORINFO;
    unsafe { GetMonitorInfoW(hmonitor, monitor_info_base) }
        .as_bool()
        .then(|| ())
        .ok_or(WindowsError::Other {
            message: "Failed to get monitor info via GetMonitorInfoW, no extra info was provided."
                .to_string(),
        })?;

    Ok(utils::get_gdi_device_id(
        &try_utf16_cstring(&monitor_info.szDevice).unwrap_or_default(),
    ))
}
