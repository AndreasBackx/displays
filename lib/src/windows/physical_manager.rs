use std::{
    io::{self, Cursor},
    ptr,
};

use edid_rs::{Reader, EDID};
use thiserror::Error;
use tracing::{debug, field, instrument, trace, Span};
use windows::Win32::{
    Foundation::{BOOL, LPARAM, RECT},
    Graphics::Gdi::{EnumDisplayMonitors, HDC, HMONITOR},
};
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

use super::{
    error::WindowsError,
    monitor::Monitor,
    monitor_info::MonitorInfo,
    physical_display::{PhysicalDisplayUpdate, PhysicalDisplayWindows},
};

#[derive(Error, Debug)]
pub enum PhysicalDisplayQueryError {
    #[error("expected '{key}' to exist in the registry but was missing")]
    RegistryKeyMissing { source: io::Error, key: String },
    #[error("invalid EDID for '{key}': {message}")]
    EDIDInvalid { message: String, key: String },
}

#[derive(Error, Debug)]
pub enum PhysicalDisplayApplyError {
    #[error(transparent)]
    WindowsError {
        #[from]
        source: WindowsError,
    },
    #[error("the following action is not (yet) supported: {message}")]
    Unsupported { message: String },
}

#[derive(Clone)]
pub struct PhysicalDisplayManagerWindows {}

impl PhysicalDisplayManagerWindows {
    pub fn query() -> Result<Vec<PhysicalDisplayWindows>, PhysicalDisplayQueryError> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let display_key_path = r"SYSTEM\CurrentControlSet\Enum\DISPLAY";
        let display_key = hklm.open_subkey(display_key_path).map_err(|source| {
            PhysicalDisplayQueryError::RegistryKeyMissing {
                source,
                key: display_key_path.to_string(),
            }
        })?;

        let mut physical_displays = vec![];

        // Iterate over each subkey in DISPLAY (each display device).
        for model_id in display_key.enum_keys() {
            let Ok(model_id) = model_id else {
                tracing::warn!(
                    "Expected child registry keys to exist but did not under: {display_key_path}"
                );
                continue;
            };
            let Ok(model_item) = display_key.open_subkey(&model_id) else {
                tracing::warn!(
                    "Expected registry key to exist but did not: {display_key_path}\\{model_id}"
                );
                continue;
            };

            for instance_id in model_item.enum_keys() {
                let Ok(instance_id) = instance_id else {
                    continue;
                };
                let device_params_path = format!("{instance_id}\\Device Parameters",);
                let Ok(instance_key) = model_item.open_subkey(&device_params_path) else {
                    tracing::warn!("Expected registry key to exist but did not: {display_key_path}\\{model_id}\\{device_params_path}");
                    continue;
                };

                // Check if the EDID value exists within this instance key.
                if let Ok(edid_data) = instance_key.get_raw_value("EDID") {
                    debug!("Found EDID for device {}\\{}:", model_id, instance_id);

                    let mut cursor = Cursor::new(edid_data.bytes);
                    let reader = &mut Reader::new(&mut cursor);
                    let edid = EDID::parse(reader).map_err(|message| {
                        PhysicalDisplayQueryError::EDIDInvalid {
                            message: message.to_string(),
                            key: format!("{display_key_path}\\{model_id}\\{instance_id}"),
                        }
                    })?;
                    trace!("{:#?}", edid);
                    let path = format!(r"\\?\DISPLAY#{model_id}#{instance_id}");
                    if let Ok(physical_display) = (path, edid).try_into() {
                        physical_displays.push(physical_display);
                    }
                } else {
                    debug!("Device found but without EDID: {display_key_path}\\{model_id}\\{instance_id}");
                }
            }
        }

        Ok(physical_displays)
    }

    fn get_monitors() -> Result<Vec<Monitor>, PhysicalDisplayApplyError> {
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
        unsafe { EnumDisplayMonitors(None, None, Some(callback), userdata) }
            .ok()
            .map_err(WindowsError::from)?;
        Ok(monitors
            .into_iter()
            .map(|hmonitor| hmonitor.into())
            .collect())
    }

    fn get_monitor_infos() -> Result<Vec<MonitorInfo>, PhysicalDisplayApplyError> {
        Self::get_monitors()?
            .into_iter()
            .map(|hmonitor| hmonitor.try_into())
            .collect::<Result<_, _>>()
            .map_err(PhysicalDisplayApplyError::from)
    }

    #[instrument(level = "debug")]
    pub fn apply(
        updates: Vec<PhysicalDisplayUpdate>,
    ) -> Result<Vec<PhysicalDisplayUpdate>, PhysicalDisplayApplyError> {
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
