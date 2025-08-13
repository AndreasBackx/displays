use windows::Win32::Devices::Display::{
    DisplayConfigGetDeviceInfo, GetDisplayConfigBufferSizes, QueryDisplayConfig,
    DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME, DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
    DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_SOURCE_DEVICE_NAME,
    DISPLAYCONFIG_TARGET_DEVICE_NAME, QDC_ALL_PATHS,
};
use windows::Win32::Foundation::{BOOL, ERROR_SUCCESS, LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW,
};

#[derive(Debug)]
struct MonitorInfo {
    hmonitor: HMONITOR,
    device_name: String,
}

unsafe extern "system" fn enum_monitors_callback(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    let monitors = &mut *(lparam.0 as *mut Vec<MonitorInfo>);
    let mut monitor_info: MONITORINFOEXW = std::mem::zeroed();
    monitor_info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

    if GetMonitorInfoW(hmonitor, &mut monitor_info as *mut _ as *mut _).as_bool() {
        let device_name = String::from_utf16_lossy(&monitor_info.szDevice)
            .trim_end_matches('\0')
            .to_string();
        monitors.push(MonitorInfo {
            hmonitor,
            device_name,
        });
    }
    BOOL(1) // Continue enumeration
}

pub fn get_hmonitor_for_path(device_path: &str, source_id: u32) -> Result<HMONITOR, anyhow::Error> {
    let mut path_count = 0;
    let mut mode_count = 0;

    let result =
        unsafe { GetDisplayConfigBufferSizes(QDC_ALL_PATHS, &mut path_count, &mut mode_count) };

    if result != ERROR_SUCCESS {
        return Err(anyhow::anyhow!(
            "GetDisplayConfigBufferSizes failed with {:?}",
            result
        ));
    }

    let mut paths: Vec<DISPLAYCONFIG_PATH_INFO> =
        vec![DISPLAYCONFIG_PATH_INFO::default(); path_count as usize];
    let mut modes: Vec<DISPLAYCONFIG_MODE_INFO> =
        vec![DISPLAYCONFIG_MODE_INFO::default(); mode_count as usize];

    let result = unsafe {
        QueryDisplayConfig(
            QDC_ALL_PATHS,
            &mut path_count,
            paths.as_mut_ptr(),
            &mut mode_count,
            modes.as_mut_ptr(),
            None,
        )
    };

    if result != ERROR_SUCCESS {
        return Err(anyhow::anyhow!(
            "QueryDisplayConfig failed with {:?}",
            result
        ));
    }

    paths.truncate(path_count as usize);
    modes.truncate(mode_count as usize);

    for path in &paths {
        if path.sourceInfo.id == source_id {
            let mut target_name_info: DISPLAYCONFIG_TARGET_DEVICE_NAME =
                unsafe { std::mem::zeroed() };
            target_name_info.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME;
            target_name_info.header.size =
                std::mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32;
            target_name_info.header.adapterId = path.targetInfo.adapterId;
            target_name_info.header.id = path.targetInfo.id;

            if unsafe { DisplayConfigGetDeviceInfo(&mut target_name_info.header) }
                == ERROR_SUCCESS.0 as i32
            {
                let current_device_path =
                    String::from_utf16_lossy(&target_name_info.monitorFriendlyDeviceName)
                        .trim_end_matches('\0')
                        .to_string();

                // This is a workaround as the API sometimes returns a different format
                let current_device_path_api =
                    String::from_utf16_lossy(&target_name_info.monitorDevicePath)
                        .trim_end_matches('\0')
                        .to_string();

                tracing::info!("current_device_path: {current_device_path}");
                tracing::info!("current_device_path_api: {current_device_path_api}");

                if device_path == current_device_path || device_path == current_device_path_api {
                    let mut source_name: DISPLAYCONFIG_SOURCE_DEVICE_NAME =
                        unsafe { std::mem::zeroed() };
                    source_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME;
                    source_name.header.size =
                        std::mem::size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32;
                    source_name.header.adapterId = path.sourceInfo.adapterId;
                    source_name.header.id = path.sourceInfo.id;

                    if unsafe { DisplayConfigGetDeviceInfo(&mut source_name.header) }
                        == ERROR_SUCCESS.0 as i32
                    {
                        let gdi_device_name =
                            String::from_utf16_lossy(&source_name.viewGdiDeviceName)
                                .trim_end_matches('\0')
                                .to_string();

                        tracing::info!("gdi_device_name: {gdi_device_name}");

                        let mut monitors: Vec<MonitorInfo> = Vec::new();
                        unsafe {
                            EnumDisplayMonitors(
                                None,
                                None,
                                Some(enum_monitors_callback),
                                LPARAM(&mut monitors as *mut _ as isize),
                            );
                        }
                        tracing::warn!("monitors: {monitors:#?}");

                        for monitor in monitors {
                            if monitor.device_name == gdi_device_name {
                                return Ok(monitor.hmonitor);
                            }
                        }
                    }
                }
            }
        }
    }

    Err(anyhow::anyhow!(
        "HMonitor not found for the given path and source id"
    ))
}
