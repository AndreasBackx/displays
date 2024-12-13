use windows::Win32::{
    Devices::Display::{GetMonitorBrightness, SetMonitorBrightness, PHYSICAL_MONITOR},
    Foundation::GetLastError,
};

use super::error::WindowsError;

#[derive(Clone, Copy)]
pub(crate) struct PhysicalMonitor(pub(crate) PHYSICAL_MONITOR);

impl PhysicalMonitor {
    pub(crate) fn get_brightness(&self) -> Result<MonitorBrightness, WindowsError> {
        let (mut min_brightness, mut current_brightness, mut max_brightness) = (0, 0, 0);
        let return_code = unsafe {
            GetMonitorBrightness(
                self.0.hPhysicalMonitor,
                &mut min_brightness,
                &mut current_brightness,
                &mut max_brightness,
            )
        };
        if return_code != 1 {
            unsafe { GetLastError() }.ok()?;
        }
        Ok(MonitorBrightness {
            min: min_brightness,
            current: current_brightness,
            max: max_brightness,
        })
    }

    pub(crate) fn set_brightness(&self, brightness: u32) -> Result<(), WindowsError> {
        let return_code = unsafe { SetMonitorBrightness(self.0.hPhysicalMonitor, brightness) };
        if return_code != 1 {
            unsafe { GetLastError() }.ok()?;
        }
        Ok(())
    }
}

impl From<PHYSICAL_MONITOR> for PhysicalMonitor {
    fn from(value: PHYSICAL_MONITOR) -> Self {
        Self(value)
    }
}

pub(crate) struct MonitorBrightness {
    pub(crate) min: u32,
    pub(crate) current: u32,
    pub(crate) max: u32,
}
