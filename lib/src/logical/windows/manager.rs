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

use crate::logical;

use super::display::{LogicalDisplayUpdate, LogicalDisplayWindows};

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
            .partition(|display| display.is_enabled);

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
        updates: Vec<LogicalDisplayUpdate>,
        validate: bool,
    ) -> anyhow::Result<Vec<LogicalDisplayUpdate>> {
        let mut used_source_ids = self.get_used_source_ids()?;
        let mut remaining_updates = updates.clone();
        for path in self.paths.iter_mut() {
            // Invalidate all mode configs.
            path.sourceInfo.Anonymous.modeInfoIdx = DISPLAYCONFIG_PATH_MODE_IDX_INVALID;
            path.targetInfo.Anonymous.modeInfoIdx = DISPLAYCONFIG_PATH_MODE_IDX_INVALID;

            let Ok(logical_display): anyhow::Result<LogicalDisplayWindows> = (*path).try_into()
            else {
                continue;
            };

            let Some(matching_update) = remaining_updates
                .iter()
                .position(|update| logical_display.matches(&update.id))
                .map(|index| remaining_updates.remove(index))
            else {
                continue;
            };

            info!("Found setup: {matching_update:?}");
            let Some(should_enable) = matching_update.content.is_enabled else {
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

        Ok(remaining_updates)
    }
}
