use std::{collections::BTreeSet, string::FromUtf16Error};

use anyhow::bail;
use tracing::info;
use windows::Win32::{
    Devices::Display::{
        DisplayConfigGetDeviceInfo, GetDisplayConfigBufferSizes, QueryDisplayConfig,
        SetDisplayConfig, DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME, DISPLAYCONFIG_MODE_INFO,
        DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_TARGET_DEVICE_NAME, QDC_ALL_PATHS,
        SDC_ALLOW_PATH_ORDER_CHANGES, SDC_APPLY, SDC_TOPOLOGY_SUPPLIED, SDC_VALIDATE,
    },
    Foundation::ERROR_SUCCESS,
    Graphics::Gdi::{DISPLAYCONFIG_PATH_ACTIVE, DISPLAYCONFIG_PATH_MODE_IDX_INVALID},
};

#[derive(Debug, PartialEq, Eq)]
pub struct LogicalDisplayWriteWindows {
    pub settings: LogicalDisplayWriteSettingsWindows,
    pub target: TargetDevice,
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct LogicalDisplayWriteSettingsWindows {
    pub is_enabled: Option<bool>,
}

pub trait LogicalDisplay {
    fn is_enabled(&self) -> bool;
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct LogicalDisplayWindows {
    pub target: TargetDevice,
    pub is_enabled: bool,
}

impl LogicalDisplayWindows {}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct TargetDevice {
    pub name: String,
    pub path: String,
}

impl LogicalDisplay for LogicalDisplayWindows {
    fn is_enabled(&self) -> bool {
        self.is_enabled
    }
}

#[derive(Clone)]
pub struct LogicalDisplayManagerWindows {
    paths: Vec<DISPLAYCONFIG_PATH_INFO>,
    modes: Vec<DISPLAYCONFIG_MODE_INFO>,
}

impl LogicalDisplayManagerWindows {
    pub fn try_new() -> anyhow::Result<Self> {
        // Get the current display configuration buffer sizes
        let mut num_path_array_elements: u32 = 0;
        let mut num_mode_info_array_elements: u32 = 0;

        let qdc_flags = QDC_ALL_PATHS;

        let status = unsafe {
            GetDisplayConfigBufferSizes(
                qdc_flags,
                &mut num_path_array_elements,
                &mut num_mode_info_array_elements,
            )
        };

        if status != ERROR_SUCCESS {
            bail!(
                "Failed to get display config buffer sizes. Error code: {:?}",
                status
            );
        }

        // Allocate memory for path and mode info arrays
        let mut paths: Vec<DISPLAYCONFIG_PATH_INFO> =
            vec![Default::default(); num_path_array_elements as usize];
        let mut modes: Vec<DISPLAYCONFIG_MODE_INFO> =
            vec![Default::default(); num_mode_info_array_elements as usize];

        // Query the current display configuration
        let status: windows::Win32::Foundation::WIN32_ERROR = unsafe {
            QueryDisplayConfig(
                qdc_flags,
                &mut num_path_array_elements,
                paths.as_mut_ptr(),
                &mut num_mode_info_array_elements,
                modes.as_mut_ptr(),
                None,
            )
        };

        if status != ERROR_SUCCESS {
            bail!("Failed to query display config. Error code: {:?}", status);
        }

        Ok(Self { paths, modes })
    }

    fn get_used_source_ids(&self) -> anyhow::Result<Vec<u32>> {
        let mut used_source_ids = Vec::<u32>::new();

        for path in self.paths.iter() {
            let is_enabled = path.flags & DISPLAYCONFIG_PATH_ACTIVE == DISPLAYCONFIG_PATH_ACTIVE;
            if is_enabled {
                used_source_ids.push(path.sourceInfo.id);
            }
        }

        Ok(used_source_ids)
    }

    pub fn query(&self) -> anyhow::Result<BTreeSet<LogicalDisplayWindows>> {
        let logical_displays: Vec<LogicalDisplayWindows> = self
            .paths
            .clone()
            .into_iter()
            .map(|path| -> anyhow::Result<_> { path.try_into() })
            .filter_map(|path| path.ok())
            .collect();

        let (enabled_displays, disabled_displays): (BTreeSet<_>, BTreeSet<_>) = logical_displays
            .into_iter()
            .partition(|display| display.is_enabled());

        // A display may be both in enabled and disabled because it may be represented/stored in
        // more than one. So remove the disables displays that are also in an enabled state.
        let only_disabled_displays: BTreeSet<_> = disabled_displays
            .into_iter()
            .filter(|disabled_display| {
                !enabled_displays
                    .iter()
                    .any(|enabled_display| enabled_display.target == disabled_display.target)
            })
            .collect();

        let mut unique_configs = enabled_displays;
        unique_configs.extend(only_disabled_displays);

        Ok(unique_configs)
    }

    pub fn apply(
        mut self,
        mut display_writes: Vec<LogicalDisplayWriteWindows>,
        validate: bool,
    ) -> anyhow::Result<()> {
        let mut used_source_ids = self.get_used_source_ids()?;
        // let mut remaining_setups = display_writes.clone();
        for path in self.paths.iter_mut() {
            // Invalidate all mode configs.
            path.sourceInfo.Anonymous.modeInfoIdx = DISPLAYCONFIG_PATH_MODE_IDX_INVALID;
            path.targetInfo.Anonymous.modeInfoIdx = DISPLAYCONFIG_PATH_MODE_IDX_INVALID;

            let logical_display: LogicalDisplayWindows = (*path).try_into()?;

            let Some(matching_display_write) = display_writes
                .iter()
                .position(|display_write| display_write.target == logical_display.target)
                .map(|index| display_writes.remove(index))
            else {
                continue;
            };

            info!("Found setup: {matching_display_write:?}");
            let Some(should_enable) = matching_display_write.settings.is_enabled else {
                continue;
            };

            let source_id = path.sourceInfo.id;
            let source_is_free = !used_source_ids.contains(&source_id);

            if should_enable && source_is_free {
                info!("Enabling display!");
                // Enable the display
                path.flags |= DISPLAYCONFIG_PATH_ACTIVE;
                used_source_ids.push(source_id);
            } else {
                info!("Disabling display!");

                // Disable the display
                path.flags &= !DISPLAYCONFIG_PATH_ACTIVE;
            }
        }

        let mut sdc_flags = SDC_TOPOLOGY_SUPPLIED | SDC_ALLOW_PATH_ORDER_CHANGES;
        if validate {
            sdc_flags |= SDC_VALIDATE;
        } else {
            sdc_flags |= SDC_APPLY;
        }

        let status: i32 = unsafe { SetDisplayConfig(Some(&self.paths), None, sdc_flags) };

        if status as u32 != ERROR_SUCCESS.0 {
            bail!("Failed to set display config. Error code: {}", status);
        }

        Ok(())
    }
}

fn try_utf16_cstring<const N: usize>(value: &[u16; N]) -> Result<String, FromUtf16Error> {
    let end_index = value
        .iter()
        .position(|&character| character == 0)
        .unwrap_or(0);
    String::from_utf16(&value[..end_index])
}

impl TryFrom<DISPLAYCONFIG_PATH_INFO> for LogicalDisplayWindows {
    type Error = anyhow::Error;

    fn try_from(value: DISPLAYCONFIG_PATH_INFO) -> Result<Self, Self::Error> {
        let mut target_device_name = DISPLAYCONFIG_TARGET_DEVICE_NAME {
            header: Default::default(),
            ..Default::default()
        };
        target_device_name.header.size =
            std::mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32;
        target_device_name.header.adapterId = value.targetInfo.adapterId;
        target_device_name.header.id = value.targetInfo.id;
        target_device_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME;

        let status = unsafe { DisplayConfigGetDeviceInfo(&mut target_device_name.header) };

        if status as u32 != ERROR_SUCCESS.0 {
            bail!("Failed to query device info. Error code: {:?}", status);
        }

        let target = target_device_name.try_into()?;
        let is_enabled = value.flags & DISPLAYCONFIG_PATH_ACTIVE == DISPLAYCONFIG_PATH_ACTIVE;
        Ok(Self { target, is_enabled })
    }
}

impl TryFrom<DISPLAYCONFIG_TARGET_DEVICE_NAME> for TargetDevice {
    type Error = anyhow::Error;

    fn try_from(value: DISPLAYCONFIG_TARGET_DEVICE_NAME) -> Result<Self, Self::Error> {
        let Ok(name) = try_utf16_cstring(&value.monitorFriendlyDeviceName) else {
            bail!("Invalid UTF16 passed for device name");
        };
        let Ok(path) = try_utf16_cstring(&value.monitorDevicePath) else {
            bail!("Invalid UTF16 passed for device path");
        };

        if name.is_empty() || path.is_empty() {
            bail!("Empty device name or path");
        }

        Ok(Self { name, path })
    }
}
