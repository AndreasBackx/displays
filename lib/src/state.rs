use crate::display_config::{DisplayConfig, DisplayConfigs};
use anyhow::{bail, Result};
use std::collections::{BTreeSet, HashMap};
use std::fmt::Display;
use tracing::info;
use windows::Win32::Devices::Display::{
    DisplayConfigGetDeviceInfo, GetDisplayConfigBufferSizes, QueryDisplayConfig, SetDisplayConfig,
    DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME, DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO,
    DISPLAYCONFIG_TARGET_DEVICE_NAME, QDC_ALL_PATHS, SDC_ALLOW_PATH_ORDER_CHANGES, SDC_APPLY,
    SDC_TOPOLOGY_SUPPLIED, SDC_VALIDATE,
};
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::Graphics::Gdi::{
    DISPLAYCONFIG_PATH_ACTIVE, DISPLAYCONFIG_PATH_MODE_IDX_INVALID,
};

pub struct State {
    paths: Vec<DISPLAYCONFIG_PATH_INFO>,
    modes: Vec<DISPLAYCONFIG_MODE_INFO>,
}

impl State {
    fn get_used_source_ids(&self) -> Result<Vec<u32>> {
        let mut used_source_ids = Vec::<u32>::new();

        for path in self.paths.iter() {
            let is_enabled = path.flags & DISPLAYCONFIG_PATH_ACTIVE == DISPLAYCONFIG_PATH_ACTIVE;
            if is_enabled {
                used_source_ids.push(path.sourceInfo.id);
            }
        }

        Ok(used_source_ids)
    }

    pub fn try_new() -> Result<State> {
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

        Ok(State { paths, modes })
    }

    pub fn update(&mut self, setups: DisplayConfigs) -> Result<()> {
        let mut used_source_ids = self.get_used_source_ids()?;
        let mut remaining_setups = setups.clone();
        for path in self.paths.iter_mut() {
            let Ok((device_name, device_path)) = get_device_info(path) else {
                continue;
            };

            let Some(setup) = remaining_setups
                .displays
                .iter()
                .filter(|item| {
                    if item.name != device_name {
                        return false;
                    }

                    if let Some(setup_path) = &item.path {
                        return &device_path == setup_path;
                    } else {
                        return true;
                    }
                })
                .nth(0)
            else {
                // Requested setup was not found, don't touch anything else.
                continue;
            };

            info!("Found setup: {setup:?}");

            path.sourceInfo.Anonymous.modeInfoIdx = DISPLAYCONFIG_PATH_MODE_IDX_INVALID;
            path.targetInfo.Anonymous.modeInfoIdx = DISPLAYCONFIG_PATH_MODE_IDX_INVALID;

            let source_id = path.sourceInfo.id;
            let source_is_free = !used_source_ids.contains(&source_id);

            if setup.is_enabled && source_is_free {
                info!("Enabling display!");
                // Enable the display
                path.flags |= DISPLAYCONFIG_PATH_ACTIVE;
                used_source_ids.push(source_id);
                let remove_setup = setup.clone();
                remaining_setups
                    .displays
                    .retain_mut(|item| *item != remove_setup);
            } else {
                info!("Disabling display!");

                // Disable the display
                path.flags &= !DISPLAYCONFIG_PATH_ACTIVE;
            }
        }

        Ok(())
    }

    pub fn apply(&mut self, validate: bool) -> Result<()> {
        // Invalidate all mode configs.
        for path_info in self.paths.iter_mut() {
            path_info.sourceInfo.Anonymous.modeInfoIdx = DISPLAYCONFIG_PATH_MODE_IDX_INVALID;
            path_info.targetInfo.Anonymous.modeInfoIdx = DISPLAYCONFIG_PATH_MODE_IDX_INVALID;
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

    pub fn query(&self) -> Result<BTreeSet<DisplayConfig>> {
        let mut configs = BTreeSet::new();

        for path_info in self.paths.iter() {
            let Ok((name, path)) = get_device_info(path_info) else {
                continue;
            };

            let is_enabled =
                path_info.flags & DISPLAYCONFIG_PATH_ACTIVE == DISPLAYCONFIG_PATH_ACTIVE;
            configs.insert(DisplayConfig {
                name,
                path: Some(path),
                is_enabled,
            });
        }

        let (enabled_configs, disabled_configs): (BTreeSet<_>, BTreeSet<_>) =
            configs.into_iter().partition(|config| config.is_enabled);

        let only_disabled_configs: BTreeSet<_> = disabled_configs
            .into_iter()
            .filter(|disabled_config| {
                let enabled_config = DisplayConfig {
                    is_enabled: true,
                    ..disabled_config.clone()
                };
                !enabled_configs.contains(&enabled_config)
            })
            .collect();

        let mut unique_configs = enabled_configs;
        unique_configs.extend(only_disabled_configs);

        Ok(unique_configs)
    }
}

#[cfg(feature = "cli")]
use prettytable::{row, Row, Table};

#[cfg(feature = "cli")]
impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut display_table_rows = HashMap::<String, Vec<Row>>::new();

        for path in self.paths.iter() {
            let source_mode_info_index = unsafe {
                if path.sourceInfo.Anonymous.modeInfoIdx == DISPLAYCONFIG_PATH_MODE_IDX_INVALID {
                    None
                } else {
                    Some(path.sourceInfo.Anonymous.modeInfoIdx as usize)
                }
            };
            let source_mode_info = source_mode_info_index.and_then(|index| self.modes.get(index));
            let source_mode =
                source_mode_info.and_then(|info| Some(unsafe { info.Anonymous.sourceMode }));

            let target_mode_info_index = unsafe {
                if path.sourceInfo.Anonymous.modeInfoIdx == DISPLAYCONFIG_PATH_MODE_IDX_INVALID {
                    None
                } else {
                    Some(path.sourceInfo.Anonymous.modeInfoIdx as usize)
                }
            };
            let target_mode_info = target_mode_info_index.and_then(|index| self.modes.get(index));
            let target_mode =
                target_mode_info.and_then(|info| Some(unsafe { info.Anonymous.targetMode }));

            let Ok((device_name, device_path)) = get_device_info(path) else {
                continue;
            };

            let is_enabled = path.flags & DISPLAYCONFIG_PATH_ACTIVE == DISPLAYCONFIG_PATH_ACTIVE;

            let display_id = format!("{} - {}", device_name.clone(), device_path.clone());
            let new_row = row![
                is_enabled.to_string(),
                path.sourceInfo.id.to_string(),
                format!(
                    "{}, {}",
                    path.sourceInfo.adapterId.LowPart, path.sourceInfo.adapterId.HighPart
                ),
                format!("{:?}", source_mode_info_index),
                source_mode.map_or("".to_string(), |info| format!(
                    "{}, {}",
                    info.width, info.height
                )),
                source_mode.map_or("".to_string(), |info| format!(
                    "{}, {}",
                    info.position.x, info.position.y
                )),
                source_mode.map_or("".to_string(), |info| format!("{:?}", info.pixelFormat)),
                path.targetInfo.id.to_string(),
                format!(
                    "{}, {}",
                    path.targetInfo.adapterId.LowPart, path.targetInfo.adapterId.HighPart
                ),
                format!("{:?}", target_mode_info_index),
            ];
            let entry = display_table_rows.entry(display_id).or_insert(vec![]);
            entry.push(new_row);
        }

        for (display_id, rows) in display_table_rows.into_iter() {
            // Create the table
            let mut table = Table::new();
            table.set_titles(row![
                "enabled",
                // "mode",
                "source",
                "adapter",
                "mode idx",
                "size",
                "position",
                "pixel format",
                "target",
                "adapter",
                "mode idx",
            ]);

            for row in rows {
                table.add_row(row);
            }
            // Print the table to stdout
            writeln!(f, "{}", display_id)?;
            writeln!(f, "{table}")?;
            writeln!(f, "")?;
        }

        Ok(())
    }
}

fn get_device_info(path: &DISPLAYCONFIG_PATH_INFO) -> Result<(String, String)> {
    let mut target_device_name = DISPLAYCONFIG_TARGET_DEVICE_NAME {
        header: Default::default(),
        ..Default::default()
    };
    target_device_name.header.size = std::mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32;
    target_device_name.header.adapterId = path.targetInfo.adapterId;
    target_device_name.header.id = path.targetInfo.id;
    target_device_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME;

    let status = unsafe { DisplayConfigGetDeviceInfo(&mut target_device_name.header) };

    if status as u32 != ERROR_SUCCESS.0 {
        bail!("Failed to query device info. Error code: {:?}", status);
    }
    let Ok(device_name) = String::from_utf16(
        &target_device_name
            .monitorFriendlyDeviceName
            // Get until null terminator
            .into_iter()
            .take_while(|character| *character != 0)
            .collect::<Vec<_>>(),
    ) else {
        bail!("Invalid UTF16 passed for device name");
    };
    let Ok(device_path) = String::from_utf16(
        &target_device_name
            .monitorDevicePath
            .into_iter()
            .take_while(|character| *character != 0)
            .collect::<Vec<_>>(),
    ) else {
        bail!("Invalid UTF16 passed for device path");
    };

    if device_name.is_empty() || device_path.is_empty() {
        bail!("Empty device name or path");
    }

    Ok((device_name, device_path))
}
