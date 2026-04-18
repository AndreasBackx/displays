use displays_types::Brightness;
use displays_windows_common::error::WindowsError;
use windows::Win32::{
    Devices::Display::{
        GetNumberOfPhysicalMonitorsFromHMONITOR, GetPhysicalMonitorsFromHMONITOR, PHYSICAL_MONITOR,
    },
    Graphics::Gdi::HMONITOR,
};

use crate::{error::ApplyError, physical_monitor::PhysicalMonitor};

#[derive(Debug)]
pub(crate) struct Monitor(pub(crate) HMONITOR);

impl From<HMONITOR> for Monitor {
    fn from(value: HMONITOR) -> Self {
        Self(value)
    }
}

impl Monitor {
    pub(crate) fn get_physical_monitors(&self) -> Result<Vec<PhysicalMonitor>, WindowsError> {
        let mut monitor_count = 0;
        unsafe { GetNumberOfPhysicalMonitorsFromHMONITOR(self.0, &mut monitor_count) }?;

        let mut physical_monitors = vec![PHYSICAL_MONITOR::default(); monitor_count as usize];
        unsafe { GetPhysicalMonitorsFromHMONITOR(self.0, physical_monitors.as_mut_slice()) }?;

        Ok(physical_monitors
            .into_iter()
            .map(|monitor| monitor.into())
            .collect())
    }

    pub(crate) fn get_brightness(&self) -> Result<Brightness, WindowsError> {
        let physical_monitors = self.get_physical_monitors()?;
        let physical_monitor = physical_monitors
            .first()
            .ok_or_else(|| WindowsError::Other {
                message: "no physical monitor found for HMONITOR".to_string(),
            })?;
        let monitor_brightness = physical_monitor.get_brightness()?;
        Ok(Brightness::new(monitor_brightness.current as u8))
    }

    pub(crate) fn set_brightness(&self, brightness: u32) -> Result<(), ApplyError> {
        let physical_monitors = self.get_physical_monitors()?;
        if physical_monitors.len() != 1 {
            return Err(ApplyError::Unsupported {
                message: format!(
                    "{} physical monitors connected to 1 HMONITOR, this is not (yet) supported.",
                    physical_monitors.len()
                ),
            });
        }
        let physical_monitor = physical_monitors[0];
        physical_monitor.set_brightness(brightness)?;
        Ok(())
    }
}
