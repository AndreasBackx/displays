use std::{io::Cursor, mem, ptr};

use anyhow::bail;
use edid_rs::{Reader, EDID};
use windows::Win32::{
    Devices::Display::{
        GetMonitorBrightness, GetNumberOfPhysicalMonitorsFromHMONITOR,
        GetPhysicalMonitorsFromHMONITOR, SetMonitorBrightness, PHYSICAL_MONITOR,
    },
    Foundation::{BOOL, LPARAM, RECT},
    Graphics::Gdi::{
        EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO, MONITORINFOEXW,
    },
};
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

use crate::logical::windows::utils::try_utf16_cstring;

use super::display::{Brightness, PhysicalDisplayUpdate, PhysicalDisplayWindows};

#[derive(Clone)]
pub struct PhysicalDisplayManagerWindows {}

impl PhysicalDisplayManagerWindows {
    pub fn try_new() -> anyhow::Result<Self> {
        Ok(Self {})
    }

    pub fn query(&self) -> anyhow::Result<Vec<PhysicalDisplayWindows>> {
        // Open the HKEY_LOCAL_MACHINE root key.
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

        // Open the DISPLAY registry key under Enum.
        let display_key_path = r"SYSTEM\CurrentControlSet\Enum\DISPLAY";
        let display_key = hklm.open_subkey(display_key_path)?;

        let mut physical_displays = vec![];

        // Iterate over each subkey in DISPLAY (each display device).
        for model_id in display_key.enum_keys() {
            let model_id = model_id?;
            let model_item = display_key.open_subkey(&model_id)?;

            // Each device may have multiple subkeys, so iterate over them.
            for instance_id in model_item.enum_keys() {
                let instance_id = instance_id?;
                let device_params_key = format!("{instance_id}\\Device Parameters",);
                let instance_key = model_item.open_subkey(&device_params_key)?;

                // Check if the EDID value exists within this instance key.
                if let Ok(edid_data) = instance_key.get_raw_value("EDID") {
                    println!("Found EDID for device {}\\{}:", model_id, instance_id);

                    let mut cursor = Cursor::new(edid_data.bytes);
                    let reader = &mut Reader::new(&mut cursor);
                    let edid = EDID::parse(reader).map_err(|err| anyhow::anyhow!(err))?;
                    println!("{:#?}", edid);
                    let path = format!(r"\\?\DISPLAY#{model_id}#{instance_id}");
                    if let Ok(physical_display) = (path, edid).try_into() {
                        physical_displays.push(physical_display);
                    }
                } else {
                    println!("No EDID found for device {}\\{}", model_id, instance_id);
                }
            }
        }

        Ok(physical_displays)
    }

    fn get_monitors(&self) -> anyhow::Result<Vec<Monitor>> {
        unsafe extern "system" fn callback(
            monitor: HMONITOR,
            _hdc_monitor: HDC,
            _lprc: *mut RECT,
            userdata: LPARAM,
        ) -> BOOL {
            let monitors: &mut Vec<HMONITOR> = &mut *(userdata.0 as *mut Vec<HMONITOR>);
            monitors.push(monitor);
            BOOL::from(true)
        }

        let mut monitors = Vec::<HMONITOR>::new();
        let userdata = LPARAM(ptr::addr_of_mut!(monitors) as _);
        unsafe { EnumDisplayMonitors(None, None, Some(callback), userdata) }.ok()?;
        Ok(monitors
            .into_iter()
            .map(|hmonitor| hmonitor.into())
            .collect())
    }

    fn get_monitor_infos(&self) -> anyhow::Result<Vec<MonitorInfo>> {
        self.get_monitors()?
            .into_iter()
            .map(|hmonitor| hmonitor.try_into())
            .collect::<anyhow::Result<_>>()
    }

    pub fn apply(
        &self,
        updates: Vec<PhysicalDisplayUpdate>,
    ) -> anyhow::Result<Vec<PhysicalDisplayUpdate>> {
        let monitor_infos = self.get_monitor_infos()?;
        let mut remaining_updates = updates.clone();

        for monitor_info in monitor_infos {
            let Some(display_id) = monitor_info.display_id() else {
                continue;
            };
            let Some(matching_update) = remaining_updates
                .iter()
                .filter_map(|update| update.id.source_id)
                .position(|source_id| source_id == display_id)
                .map(|index| remaining_updates.remove(index))
            else {
                continue;
            };

            if let Some(brightness) = matching_update.content.brightness {
                monitor_info.monitor.set_brightness(brightness)?;
            }
        }

        Ok(remaining_updates)
    }
}

pub struct MonitorInfo {
    monitor: Monitor,
    info: MONITORINFOEXW,
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
    fn path(&self) -> String {
        try_utf16_cstring(&self.info.szDevice).unwrap_or_default()
    }

    fn display_id(&self) -> Option<u32> {
        self.path()
            .chars()
            .last()
            .and_then(|c| c.to_digit(10))
            .map(|digit| digit)
    }
}

pub struct Monitor(HMONITOR);

impl From<HMONITOR> for Monitor {
    fn from(value: HMONITOR) -> Self {
        Self(value)
    }
}

impl Monitor {
    fn get_physical_monitors(&self) -> anyhow::Result<Vec<PhysicalMonitor>> {
        let mut monitor_count = 0;
        unsafe { GetNumberOfPhysicalMonitorsFromHMONITOR(self.0, &mut monitor_count) }?;

        let mut physical_monitors = vec![PHYSICAL_MONITOR::default(); monitor_count as usize];
        unsafe { GetPhysicalMonitorsFromHMONITOR(self.0, physical_monitors.as_mut_slice()) }?;

        Ok(physical_monitors
            .into_iter()
            .map(|monitor| monitor.into())
            .collect())
    }

    fn get_brightness(&self) -> anyhow::Result<Brightness> {
        let physical_monitors = self.get_physical_monitors()?;
        if physical_monitors.len() != 1 {
            bail!("Found more physical monitors connected to 1 HMONITOR, not supported!");
        }
        let physical_monitor = physical_monitors[0];
        let monitor_brightness = physical_monitor.get_brightness()?;
        Ok(Brightness::new(monitor_brightness.current as u8))
    }

    fn set_brightness(&self, brightness: u32) -> anyhow::Result<()> {
        let physical_monitors = self.get_physical_monitors()?;
        if physical_monitors.len() != 1 {
            bail!("Found more physical monitors connected to 1 HMONITOR, not supported!");
        }
        let physical_monitor = physical_monitors[0];
        physical_monitor.set_brightness(brightness)?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct PhysicalMonitor(PHYSICAL_MONITOR);

impl PhysicalMonitor {
    fn get_brightness(&self) -> anyhow::Result<MonitorBrightness> {
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
            bail!("failed to get monitor brightness");
        }
        Ok(MonitorBrightness {
            min: min_brightness,
            current: current_brightness,
            max: max_brightness,
        })
    }

    fn set_brightness(&self, brightness: u32) -> anyhow::Result<()> {
        let return_code = unsafe { SetMonitorBrightness(self.0.hPhysicalMonitor, brightness) };
        if return_code != 1 {
            bail!("failed to set monitor brightness");
        }
        Ok(())
    }
}

impl From<PHYSICAL_MONITOR> for PhysicalMonitor {
    fn from(value: PHYSICAL_MONITOR) -> Self {
        Self(value)
    }
}

struct MonitorBrightness {
    min: u32,
    current: u32,
    max: u32,
}
