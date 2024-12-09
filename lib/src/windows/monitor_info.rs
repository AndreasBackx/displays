use std::mem;

use windows::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MONITORINFO, MONITORINFOEXW,
    };

use super::{
    monitor::Monitor,
    utils::try_utf16_cstring,
};

pub struct MonitorInfo {
    pub(crate) monitor: Monitor,
    pub(crate) info: MONITORINFOEXW,
}

impl TryFrom<Monitor> for MonitorInfo {
    type Error = anyhow::Error;

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
            .ok_or(anyhow::anyhow!("could not get monitor info"))
    }
}

impl MonitorInfo {
    pub(crate) fn path(&self) -> String {
        try_utf16_cstring(&self.info.szDevice).unwrap_or_default()
    }

    pub(crate) fn display_id(&self) -> Option<u32> {
        self.path()
            .chars()
            .last()
            .and_then(|c| c.to_digit(10))
            .map(|digit| digit)
    }
}
