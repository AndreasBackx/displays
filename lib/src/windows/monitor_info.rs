use std::{
    fmt::{self, Display},
    mem,
};

use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MONITORINFO, MONITORINFOEXW};

use crate::windows::utils;

use super::{error::WindowsError, monitor::Monitor, utils::try_utf16_cstring};

pub(crate) struct MonitorInfo {
    pub(crate) monitor: Monitor,
    pub(crate) info: MONITORINFOEXW,
}

impl MonitorInfo {
    pub(crate) fn path(&self) -> String {
        try_utf16_cstring(&self.info.szDevice).unwrap_or_default()
    }

    pub(crate) fn gdi_device_id(&self) -> Option<u32> {
        utils::get_gdi_device_id(&self.path())
    }
}

impl Display for MonitorInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MonitorInfo {{path: {path}, gdi_device_id: {gdi_device_id:?}}}",
            path = self.path(),
            gdi_device_id = self.gdi_device_id(),
        )
    }
}

impl fmt::Debug for MonitorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MonitorInfo")
            // .field("monitor", &self.monitor)
            // .field("info", &self.info)
            .field("path", &self.path())
            .field("display_id", &self.gdi_device_id())
            .finish()
    }
}

impl TryFrom<Monitor> for MonitorInfo {
    type Error = WindowsError;

    fn try_from(value: Monitor) -> Result<Self, Self::Error> {
        let mut monitor_info = MONITORINFOEXW {
            monitorInfo: MONITORINFO {
                cbSize: mem::size_of::<MONITORINFOEXW>() as u32,
                ..Default::default()
            },
            ..Default::default()
        };

        let monitor_info_base = &mut monitor_info as *mut MONITORINFOEXW as *mut MONITORINFO;

        // Get the monitor info for this monitor
        unsafe { GetMonitorInfoW(value.0, monitor_info_base) }
            .as_bool()
            .then(|| MonitorInfo {
                monitor: value,
                info: monitor_info,
            })
            .ok_or(WindowsError::Other {
                message:
                    "Failed to get monitor info via GetMonitorInfoW, no extra info was provided."
                        .to_string(),
            })
    }
}
