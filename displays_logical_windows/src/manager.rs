use std::collections::{BTreeMap, BTreeSet};

use windows::Win32::{
    Devices::Display::{
        GetDisplayConfigBufferSizes, QueryDisplayConfig, SetDisplayConfig, DISPLAYCONFIG_MODE_INFO,
        DISPLAYCONFIG_PATH_INFO, QDC_ALL_PATHS, SDC_ALLOW_CHANGES, SDC_APPLY,
        SDC_USE_SUPPLIED_DISPLAY_CONFIG, SDC_VALIDATE,
    },
    Foundation::WIN32_ERROR,
    Graphics::Gdi::{DISPLAYCONFIG_PATH_ACTIVE, DISPLAYCONFIG_PATH_MODE_IDX_INVALID},
};

use crate::{
    error::{ApplyError, QueryError},
    types::{logical_display_from_path_info, logical_display_matches},
};
use displays_logical_types::{LogicalDisplay, LogicalDisplayUpdate};
use displays_windows_common::error::WindowsError;

#[derive(Clone)]
pub struct LogicalDisplayManager {}

struct DisplayConfig {
    paths: Vec<DISPLAYCONFIG_PATH_INFO>,
    modes: Vec<DISPLAYCONFIG_MODE_INFO>,
}

impl LogicalDisplayManager {
    #[tracing::instrument(ret, level = "trace")]
    pub fn query() -> Result<BTreeSet<LogicalDisplay>, QueryError> {
        let display_config = DisplayConfig::try_new()?;
        let logical_displays: Vec<LogicalDisplay> = display_config
            .get_path_infos()
            .infos
            .iter()
            .map(|path_info| logical_display_from_path_info(path_info))
            .filter_map(|path| path.ok())
            .collect();

        let mut deduped: BTreeMap<String, LogicalDisplay> = BTreeMap::new();
        for display in logical_displays {
            deduped
                .entry(display.metadata.path.clone())
                .and_modify(|existing| {
                    if display_rank(&display) > display_rank(existing) {
                        *existing = display.clone();
                    }
                })
                .or_insert(display);
        }

        Ok(deduped.into_values().collect())
    }

    pub fn apply(
        updates: Vec<LogicalDisplayUpdate>,
        validate: bool,
    ) -> Result<Vec<LogicalDisplayUpdate>, ApplyError> {
        if updates.is_empty() {
            return Ok(updates);
        }

        let display_config = DisplayConfig::try_new()?;
        let mut used_source_ids = display_config.get_used_source_ids();
        let mut remaining_updates = updates.clone();
        let mut path_infos = PathInfos { infos: vec![] };
        let all_path_infos = display_config.get_path_infos();

        for mut path_info in all_path_infos.infos.clone() {
            let Ok(logical_display) = logical_display_from_path_info(&path_info) else {
                continue;
            };

            let Some((matching_update, matching_index)) = remaining_updates
                .iter()
                .position(|update| logical_display_matches(&logical_display, &update.id))
                .and_then(|index| {
                    remaining_updates
                        .get(index)
                        .map(|matching_update| (matching_update, index))
                })
            else {
                continue;
            };

            if let Some(should_enable) = matching_update.content.is_enabled {
                let source_id = path_info.path.sourceInfo.id;
                let is_enabled =
                    path_info.path.flags & DISPLAYCONFIG_PATH_ACTIVE == DISPLAYCONFIG_PATH_ACTIVE;

                if should_enable {
                    if !is_enabled {
                        let source_is_free = !used_source_ids.contains(&source_id);

                        if !source_is_free {
                            continue;
                        }

                        path_info.path.flags |= DISPLAYCONFIG_PATH_ACTIVE;
                        used_source_ids.push(source_id);
                    }
                } else if is_enabled {
                    path_info.path.flags &= !DISPLAYCONFIG_PATH_ACTIVE;
                    used_source_ids.retain(|used_source_id| used_source_id != &source_id);
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

            remaining_updates.remove(matching_index);
            path_infos.infos.push(path_info);
        }

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

fn display_rank(display: &LogicalDisplay) -> (bool, usize) {
    (
        display.state.is_enabled,
        [
        display.state.logical_size.is_some(),
        display.state.mode_size.is_some(),
        display.state.scale_ratio_milli.is_some(),
        display.state.pixel_format.is_some(),
        display.state.mode_position.is_some(),
        display.state.logical_position.is_some(),
    ]
        .into_iter()
        .filter(|present| *present)
        .count(),
    )
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

        (paths, modes)
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

        let mut paths: Vec<DISPLAYCONFIG_PATH_INFO> =
            vec![Default::default(); num_path_array_elements as usize];
        let mut modes: Vec<DISPLAYCONFIG_MODE_INFO> =
            vec![Default::default(); num_mode_info_array_elements as usize];

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
