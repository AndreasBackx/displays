use std::collections::BTreeSet;

use thiserror::Error;
use tracing::{info, instrument};
use windows::Win32::{
    Devices::Display::{
        GetDisplayConfigBufferSizes, QueryDisplayConfig, SetDisplayConfig, DISPLAYCONFIG_MODE_INFO,
        DISPLAYCONFIG_PATH_INFO, QDC_ALL_PATHS, SDC_ALLOW_PATH_ORDER_CHANGES, SDC_APPLY,
        SDC_TOPOLOGY_SUPPLIED, SDC_VALIDATE,
    },
    Foundation::WIN32_ERROR,
    Graphics::Gdi::{DISPLAYCONFIG_PATH_ACTIVE, DISPLAYCONFIG_PATH_MODE_IDX_INVALID},
};

use crate::logical_display::LogicalDisplayUpdate;

use super::{error::WindowsError, logical_display::LogicalDisplayWindows};

#[derive(Error, Debug)]
pub enum LogicalDisplayQueryError {
    #[error(transparent)]
    WindowsError {
        #[from]
        source: WindowsError,
    },
}

#[derive(Error, Debug)]
pub enum LogicalDisplayApplyError {
    #[error(transparent)]
    WindowsError {
        #[from]
        source: WindowsError,
    },
}

#[derive(Clone)]
pub struct LogicalDisplayManagerWindows {}

struct DisplayConfig {
    paths: Vec<DISPLAYCONFIG_PATH_INFO>,
    modes: Vec<DISPLAYCONFIG_MODE_INFO>,
}

impl LogicalDisplayManagerWindows {
    #[instrument(ret)]
    pub fn metadata() -> Result<BTreeSet<LogicalDisplayWindows>, LogicalDisplayQueryError> {
        let display_config = DisplayConfig::try_new()?;
        let logical_displays: Vec<LogicalDisplayWindows> = display_config
            .paths
            .clone()
            .into_iter()
            .map(|path| -> Result<_, _> { path.try_into() })
            .filter_map(|path| path.ok())
            .collect();

        let (enabled_displays, disabled_displays): (BTreeSet<_>, BTreeSet<_>) = logical_displays
            .into_iter()
            .partition(|display| display.state.is_enabled);

        // A display may be both in enabled and disabled because it may be represented/stored in
        // more than one. So remove the disables displays that are also in an enabled state.
        let only_disabled_displays: BTreeSet<_> = disabled_displays
            .into_iter()
            .filter(|disabled_display| {
                !enabled_displays
                    .iter()
                    .any(|enabled_display| enabled_display.metadata == disabled_display.metadata)
            })
            .collect();

        let mut unique_configs = enabled_displays;
        unique_configs.extend(only_disabled_displays);

        Ok(unique_configs)
    }

    pub(crate) fn apply(
        updates: Vec<LogicalDisplayUpdate>,
        validate: bool,
    ) -> Result<Vec<LogicalDisplayUpdate>, LogicalDisplayApplyError> {
        if updates.len() == 0 {
            return Ok(updates);
        }

        let mut display_config = DisplayConfig::try_new()?;
        let mut used_source_ids = display_config.get_used_source_ids();
        let mut remaining_updates = updates.clone();
        let mut has_changed = false;

        tracing::debug!("Applying updates: {updates:?}");

        // TODO Sort by enabled as we want those first!!!
        for path in display_config.paths.iter_mut() {
            // Invalidate all mode configs.
            path.sourceInfo.Anonymous.modeInfoIdx = DISPLAYCONFIG_PATH_MODE_IDX_INVALID;
            path.targetInfo.Anonymous.modeInfoIdx = DISPLAYCONFIG_PATH_MODE_IDX_INVALID;

            let Ok(logical_display): Result<LogicalDisplayWindows, _> = (*path).try_into() else {
                continue;
            };

            let is_enabled = path.flags & DISPLAYCONFIG_PATH_ACTIVE == DISPLAYCONFIG_PATH_ACTIVE;
            tracing::debug!("Checking display: {logical_display:?} is_enabled = {is_enabled}");

            let Some((matching_update, matching_index)) = remaining_updates
                .iter()
                .position(|update| logical_display.matches(&update.id))
                .and_then(|index| {
                    remaining_updates
                        .get(index)
                        .map(|matching_update| (matching_update, index))
                })
            else {
                continue;
            };

            tracing::info!("Found setup: {matching_update:?}");
            let Some(should_enable) = matching_update.content.is_enabled else {
                continue;
            };

            let source_id = path.sourceInfo.id;
            let source_is_free = !used_source_ids.contains(&source_id);

            // Whether the display update was used, if so then remove it from the remaining updates.
            let mut used = false;
            if should_enable {
                if is_enabled {
                    tracing::info!("Display is already enabled!");
                    used = true;
                } else {
                    if source_is_free {
                        tracing::info!("Enabling display!");
                        // Enable the display
                        path.flags |= DISPLAYCONFIG_PATH_ACTIVE;
                        used_source_ids.push(source_id);
                        used = true;
                        has_changed = true;
                    } else {
                        tracing::trace!("Trying to enable but source {source_id} is not free")
                    }
                }
            } else {
                tracing::info!("Disabling display!");
                used = true;

                if !is_enabled {
                    tracing::info!("Display is already disabled!");
                } else {
                    // Disable the display
                    path.flags &= !DISPLAYCONFIG_PATH_ACTIVE;
                    // used_source_ids.retain(|used_source_id| used_source_id != &source_id);
                    has_changed = true;
                }
            }

            if used {
                remaining_updates.remove(matching_index);
            }
        }

        if !has_changed {
            return Ok(remaining_updates);
        }

        let mut sdc_flags = SDC_TOPOLOGY_SUPPLIED | SDC_ALLOW_PATH_ORDER_CHANGES;
        if validate {
            sdc_flags |= SDC_VALIDATE;
        } else {
            sdc_flags |= SDC_APPLY;
        }

        WIN32_ERROR(
            unsafe { SetDisplayConfig(Some(&display_config.paths), None, sdc_flags) } as u32,
        )
        .ok()
        .map_err(WindowsError::from)?;

        Ok(remaining_updates)
    }
}

impl DisplayConfig {
    fn try_new() -> Result<Self, WindowsError> {
        // Get the current display configuration buffer sizes
        let mut num_path_array_elements: u32 = 0;
        let mut num_mode_info_array_elements: u32 = 0;

        let qdc_flags = QDC_ALL_PATHS;

        unsafe {
            GetDisplayConfigBufferSizes(
                qdc_flags,
                &mut num_path_array_elements,
                &mut num_mode_info_array_elements,
            )
        }
        .ok()?;

        // Allocate memory for path and mode info arrays
        let mut paths: Vec<DISPLAYCONFIG_PATH_INFO> =
            vec![Default::default(); num_path_array_elements as usize];
        let mut modes: Vec<DISPLAYCONFIG_MODE_INFO> =
            vec![Default::default(); num_mode_info_array_elements as usize];

        // Query the current display configuration
        unsafe {
            QueryDisplayConfig(
                qdc_flags,
                &mut num_path_array_elements,
                paths.as_mut_ptr(),
                &mut num_mode_info_array_elements,
                modes.as_mut_ptr(),
                None,
            )
        }
        .ok()?;

        Ok(Self { paths, modes })
    }

    fn get_used_source_ids(&self) -> Vec<u32> {
        let mut used_source_ids = Vec::<u32>::new();

        for path in self.paths.iter() {
            let is_enabled = path.flags & DISPLAYCONFIG_PATH_ACTIVE == DISPLAYCONFIG_PATH_ACTIVE;
            if is_enabled {
                used_source_ids.push(path.sourceInfo.id);
            }
        }

        used_source_ids
    }
}
