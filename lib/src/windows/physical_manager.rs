use std::{io::Cursor, ptr};

use edid_rs::{Reader, EDID};
use tracing::{debug, field, info, info_span, instrument, trace, Instrument, Span};
use windows::Win32::{
    Foundation::{BOOL, LPARAM, RECT},
    Graphics::Gdi::{EnumDisplayMonitors, HDC, HMONITOR},
};
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

use super::{
    monitor::Monitor,
    monitor_info::MonitorInfo,
    physical_display::{PhysicalDisplayUpdate, PhysicalDisplayWindows},
};

#[derive(Clone)]
pub struct PhysicalDisplayManagerWindows {}

impl PhysicalDisplayManagerWindows {
    pub fn query() -> anyhow::Result<Vec<PhysicalDisplayWindows>> {
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
                    debug!("Found EDID for device {}\\{}:", model_id, instance_id);

                    let mut cursor = Cursor::new(edid_data.bytes);
                    let reader = &mut Reader::new(&mut cursor);
                    let edid = EDID::parse(reader).map_err(|err| anyhow::anyhow!(err))?;
                    trace!("{:#?}", edid);
                    let path = format!(r"\\?\DISPLAY#{model_id}#{instance_id}");
                    if let Ok(physical_display) = (path, edid).try_into() {
                        physical_displays.push(physical_display);
                    }
                } else {
                    debug!("No EDID found for device {}\\{}", model_id, instance_id);
                }
            }
        }

        Ok(physical_displays)
    }

    fn get_monitors() -> anyhow::Result<Vec<Monitor>> {
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

    fn get_monitor_infos() -> anyhow::Result<Vec<MonitorInfo>> {
        Self::get_monitors()?
            .into_iter()
            .map(|hmonitor| hmonitor.try_into())
            .collect::<anyhow::Result<_>>()
    }

    #[instrument(level = "debug")]
    pub fn apply(
        updates: Vec<PhysicalDisplayUpdate>,
    ) -> anyhow::Result<Vec<PhysicalDisplayUpdate>> {
        if updates.is_empty() {
            return Ok(updates);
        }

        let monitor_infos: Vec<MonitorInfo> = Self::get_monitor_infos()?;
        let mut remaining_updates = updates.clone();

        for monitor_info in monitor_infos {
            Span::current().record("monitor_info", field::display(&monitor_info));
            let Some(display_id) = monitor_info.display_id() else {
                continue;
            };
            let Some(matching_update) = remaining_updates
                .iter()
                .filter_map(|update| update.id.source_id)
                .position(|source_id| source_id + 1 == display_id)
                .map(|index| remaining_updates.remove(index))
            else {
                debug!("No update matching {monitor_info}");
                continue;
            };

            if let Some(brightness) = matching_update.content.brightness {
                monitor_info.monitor.set_brightness(brightness)?;
            }
        }

        Ok(remaining_updates)
    }
}
