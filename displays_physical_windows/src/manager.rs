use std::{
    collections::BTreeMap,
    io::{self, Cursor},
    ptr,
};

use displays_windows_common::{error::WindowsError, types::DisplayIdentifierInner};
use edid_rs::{Reader, EDID};
use windows::{
    core::BOOL,
    Win32::{
        Foundation::{LPARAM, RECT},
        Graphics::Gdi::{EnumDisplayMonitors, HDC, HMONITOR},
    },
};
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

use crate::{
    error::{ApplyError, QueryError},
    monitor::Monitor,
    monitor_info::MonitorInfo,
    types::{PhysicalDisplayMetadata, PhysicalDisplayState, PhysicalDisplayUpdate},
};

#[derive(Clone)]
pub struct PhysicalDisplayManager {}

impl PhysicalDisplayManager {
    #[tracing::instrument(ret, level = "trace")]
    pub fn metadata() -> Result<Vec<PhysicalDisplayMetadata>, QueryError> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let display_key_path = r"SYSTEM\CurrentControlSet\Enum\DISPLAY";
        let display_key = hklm.open_subkey(display_key_path).map_err(|source| {
            QueryError::RegistryKeyMissing {
                source,
                key: display_key_path.to_string(),
            }
        })?;

        let mut physical_displays = vec![];

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

                if let Ok(edid_data) = instance_key.get_raw_value("EDID") {
                    let mut cursor = Cursor::new(edid_data.bytes);
                    let reader = &mut Reader::new(&mut cursor);
                    let edid = EDID::parse(reader).map_err(|message| QueryError::EDIDInvalid {
                        message: message.to_string(),
                        key: format!("{display_key_path}\\{model_id}\\{instance_id}"),
                    })?;
                    let path = format!(r"\\?\DISPLAY#{model_id}#{instance_id}");
                    if let Ok(physical_display) = (path, edid).try_into() {
                        physical_displays.push(physical_display);
                    }
                }
            }
        }

        Ok(physical_displays)
    }

    #[tracing::instrument(ret, level = "trace")]
    pub fn state(
        ids: Vec<DisplayIdentifierInner>,
    ) -> Result<BTreeMap<DisplayIdentifierInner, PhysicalDisplayState>, QueryError> {
        let monitor_info_by_id: BTreeMap<DisplayIdentifierInner, MonitorInfo> =
            Self::get_monitor_info_by_id(ids)?;

        let state = monitor_info_by_id
            .into_iter()
            .map(|(id, monitor_info)| {
                Ok((
                    id,
                    PhysicalDisplayState {
                        brightness: monitor_info.monitor.get_brightness()?,
                        scale_factor: monitor_info.monitor.get_scale_factor()?,
                    },
                ))
            })
            .collect::<Result<_, QueryError>>()?;

        Ok(state)
    }

    #[tracing::instrument(ret, level = "trace")]
    pub fn get_monitor_info_by_id(
        ids: Vec<DisplayIdentifierInner>,
    ) -> Result<BTreeMap<DisplayIdentifierInner, MonitorInfo>, WindowsError> {
        if ids.is_empty() {
            return Ok(BTreeMap::new());
        }

        let monitor_infos: Vec<MonitorInfo> = Self::get_monitor_infos()?;

        let mut monitor_info_by_gdi_device_id: BTreeMap<_, _> = monitor_infos
            .into_iter()
            .filter_map(|monitor_info| {
                monitor_info
                    .gdi_device_id()
                    .map(|display_id| (display_id, monitor_info))
            })
            .collect();

        Ok(ids
            .into_iter()
            .filter_map(|id| {
                id.gdi_device_id
                    .as_ref()
                    .and_then(|gdi_device_id| monitor_info_by_gdi_device_id.remove(gdi_device_id))
                    .map(|monitor_info| (id, monitor_info))
            })
            .collect())
    }

    fn get_monitors() -> Result<Vec<Monitor>, WindowsError> {
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

    #[tracing::instrument(ret, level = "trace")]
    fn get_monitor_infos() -> Result<Vec<MonitorInfo>, WindowsError> {
        Self::get_monitors()?
            .into_iter()
            .map(|hmonitor| hmonitor.try_into())
            .collect::<Result<_, _>>()
    }

    #[tracing::instrument(level = "debug")]
    pub fn apply(
        updates: Vec<PhysicalDisplayUpdate>,
    ) -> Result<Vec<PhysicalDisplayUpdate>, ApplyError> {
        if updates.is_empty() {
            return Ok(updates);
        }

        let ids = updates.iter().map(|update| update.id.clone()).collect();
        let monitor_info_by_id = Self::get_monitor_info_by_id(ids)?;

        let mut remaining_updates = vec![];
        for update in updates {
            let Some(monitor_info) = monitor_info_by_id.get(&update.id) else {
                remaining_updates.push(update);
                continue;
            };

            if let Some(brightness) = update.content.brightness {
                monitor_info.monitor.set_brightness(brightness)?;
            }
        }
        Ok(remaining_updates)
    }
}
