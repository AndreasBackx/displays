use std::collections::BTreeSet;

use thiserror::Error;
use windows::Win32::{
    Devices::Display::{
        GetDisplayConfigBufferSizes, QueryDisplayConfig, SetDisplayConfig, DISPLAYCONFIG_MODE_INFO,
        DISPLAYCONFIG_PATH_INFO, QDC_ALL_PATHS, SDC_ALLOW_CHANGES, SDC_ALLOW_PATH_ORDER_CHANGES,
        SDC_APPLY, SDC_TOPOLOGY_SUPPLIED, SDC_USE_SUPPLIED_DISPLAY_CONFIG, SDC_VALIDATE,
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
    #[tracing::instrument(ret, level = "trace")]
    pub fn metadata() -> Result<BTreeSet<LogicalDisplayWindows>, LogicalDisplayQueryError> {
        let display_config = DisplayConfig::try_new()?;
        let logical_displays: Vec<LogicalDisplayWindows> = display_config
            .get_path_infos()
            .infos
            .iter()
            .map(|path_info| -> Result<_, _> { path_info.try_into() })
            .filter_map(|path| path.ok())
            .collect();
        // let logical_displays: Vec<LogicalDisplayWindows> = display_config
        //     .paths
        //     .clone()
        //     .into_iter()
        //     .map(|path| -> Result<_, _> { path.try_into() })
        //     .filter_map(|path| path.ok())
        //     .collect();

        let (enabled_displays, disabled_displays): (BTreeSet<_>, BTreeSet<_>) = logical_displays
            .into_iter()
            .partition(|display| display.state.is_enabled);

        // A display may be both in enabled and disabled because it may be represented/stored in
        // more than one. So remove the disabled displays that are also in an enabled state.
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
        let mut any_have_changed = false;

        let mut path_infos = PathInfos { infos: vec![] };
        let mut all_path_infos = display_config.get_path_infos();

        // "Should" be sorted by enabled first.
        for mut path_info in all_path_infos.infos.clone() {
            // Invalidate all mode configs, needed to tell Windows to reuse existing configuration.
            // path_info.path.sourceInfo.Anonymous.modeInfoIdx = DISPLAYCONFIG_PATH_MODE_IDX_INVALID;
            // path_info.path.targetInfo.Anonymous.modeInfoIdx = DISPLAYCONFIG_PATH_MODE_IDX_INVALID;

            let Ok(logical_display): Result<LogicalDisplayWindows, _> = (&path_info).try_into()
            else {
                continue;
            };
            tracing::debug!("logical_display: {logical_display:#?}");

            let Some((matching_update, matching_index)) = remaining_updates
                .iter()
                .position(|update| logical_display.matches(&update.id))
                .and_then(|index| {
                    remaining_updates
                        .get(index)
                        .map(|matching_update| (matching_update, index))
                })
            else {
                // path_infos.infos.push(path_info);
                continue;
            };

            if let Some(should_enable) = matching_update.content.is_enabled {
                let source_id = path_info.path.sourceInfo.id;
                let is_enabled =
                    path_info.path.flags & DISPLAYCONFIG_PATH_ACTIVE == DISPLAYCONFIG_PATH_ACTIVE;

                // Whether the display update was used, if so then remove it from the remaining updates.
                if should_enable {
                    if is_enabled {
                        tracing::trace!("Display is already enabled!");
                    } else {
                        let source_is_free = !used_source_ids.contains(&source_id);

                        if !source_is_free {
                            tracing::trace!("Trying to enable but source {source_id} is not free");
                            continue;
                        }

                        tracing::trace!("Enabling display!");
                        // Enable the display
                        path_info.path.flags |= DISPLAYCONFIG_PATH_ACTIVE;
                        used_source_ids.push(source_id);
                        any_have_changed = true;
                    }
                } else {
                    tracing::trace!("Disabling display!");

                    if !is_enabled {
                        tracing::trace!("Display is already disabled!");
                    } else {
                        // Disable the display
                        path_info.path.flags &= !DISPLAYCONFIG_PATH_ACTIVE;
                        used_source_ids.retain(|used_source_id| used_source_id != &source_id);
                        any_have_changed = true;
                    }
                }
            };

            if let Some(orientation) = &matching_update.content.orientation {
                path_info.path.targetInfo.rotation = orientation.into();
            }

            if let Some(mode_source) = path_info.mode_source {
                let mut source_mode = unsafe { mode_source.Anonymous.sourceMode };

                if let Some(width) = matching_update.content.width {
                    source_mode.width = width;
                }
                if let Some(height) = matching_update.content.height {
                    source_mode.height = height;
                }
                if let Some(pixel_format) = &matching_update.content.pixel_format {
                    source_mode.pixelFormat = pixel_format.into();
                }
                if let Some(position) = &matching_update.content.position {
                    source_mode.position = position.into();
                }

                tracing::warn!("source_mode = {:?}", source_mode);
            }

            // if let Some(mode_target) = path_info.mode_target {
            //     let target_mode = unsafe { mode_target.Anonymous.targetMode };

            //     // if let Some(orientation) = matching_update.content.orientation {
            //     //     target_mode.orientation = orientation.into();
            //     // }
            // }

            remaining_updates.remove(matching_index);
            path_infos.infos.push(path_info);
        }

        // TODO uncommnet again but impleent fully
        // if !any_have_changed {
        //     return Ok(remaining_updates);
        // }

        // let mut sdc_flags = SDC_TOPOLOGY_SUPPLIED | SDC_ALLOW_PATH_ORDER_CHANGES;
        // if validate {
        //     sdc_flags |= SDC_VALIDATE;
        // } else {
        //     sdc_flags |= SDC_APPLY;
        // }

        // WIN32_ERROR(
        //     unsafe { SetDisplayConfig(Some(&display_config.paths), None, sdc_flags) } as u32,
        // )
        // .ok()
        // .map_err(WindowsError::from)?;

        tracing::debug!("Updating {}", path_infos.infos.len());
        let (paths, modes) = path_infos.into_vecs();

        let mut sdc_flags = SDC_USE_SUPPLIED_DISPLAY_CONFIG | SDC_ALLOW_CHANGES;
        if validate {
            sdc_flags |= SDC_VALIDATE;
        } else {
            sdc_flags |= SDC_APPLY;
        }

        WIN32_ERROR(unsafe { SetDisplayConfig(Some(&paths), Some(&modes), sdc_flags) } as u32)
            .ok()
            .map_err(WindowsError::from)?;

        Ok(remaining_updates)
    }
}

pub(crate) struct PathInfos {
    pub(crate) infos: Vec<PathInfo>,
}

impl PathInfos {
    pub(crate) fn into_vecs(self) -> (Vec<DISPLAYCONFIG_PATH_INFO>, Vec<DISPLAYCONFIG_MODE_INFO>) {
        let mut paths = vec![];
        let mut modes = vec![];

        for mut path_info in self.infos {
            let source_index = if let Some(mode_source) = path_info.mode_source {
                modes.push(mode_source);
                (modes.len() - 1) as u32
            } else {
                DISPLAYCONFIG_PATH_MODE_IDX_INVALID
            };

            let target_index = if let Some(mode_target) = path_info.mode_target {
                modes.push(mode_target);
                (modes.len() - 1) as u32
            } else {
                DISPLAYCONFIG_PATH_MODE_IDX_INVALID
            };

            path_info.path.sourceInfo.Anonymous.modeInfoIdx = source_index;
            path_info.path.targetInfo.Anonymous.modeInfoIdx = target_index;

            paths.push(path_info.path)
        }

        return (paths, modes);
    }
}

#[derive(Clone)]
pub(crate) struct PathInfo {
    pub(crate) path: DISPLAYCONFIG_PATH_INFO,
    pub(crate) mode_source: Option<DISPLAYCONFIG_MODE_INFO>,
    pub(crate) mode_target: Option<DISPLAYCONFIG_MODE_INFO>,
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

        paths.truncate(num_path_array_elements as usize);
        modes.truncate(num_mode_info_array_elements as usize);

        Ok(Self { paths, modes })
    }

    fn get_path_infos(&self) -> PathInfos {
        PathInfos {
            infos: self
                .paths
                .iter()
                .map(|path| self.get_path_info(path))
                .collect(),
        }
    }

    fn get_path_info(&self, path: &DISPLAYCONFIG_PATH_INFO) -> PathInfo {
        let mode_source = self
            .modes
            .get(unsafe { path.sourceInfo.Anonymous.modeInfoIdx as usize });

        let mode_target = self
            .modes
            .get(unsafe { path.targetInfo.Anonymous.modeInfoIdx as usize });

        PathInfo {
            path: path.clone(),
            mode_source: mode_source.copied(),
            mode_target: mode_target.copied(),
        }
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
