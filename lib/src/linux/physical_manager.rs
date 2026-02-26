use ddc_hi::{Ddc, Display, FeatureCode};
use thiserror::Error;

use crate::{
    display::{Brightness, DisplayUpdate},
    display_identifier::DisplayIdentifierInner,
    physical_display::{PhysicalDisplay, PhysicalDisplayMetadata, PhysicalDisplayState},
};

#[derive(Error, Debug)]
pub enum PhysicalDisplayQueryError {
    #[error("failed to enumerate DDC displays")]
    Enumerate,
    #[error("missing i2c access for display '{display_id}'")]
    MissingI2cAccess { display_id: String },
    #[error("insufficient permissions for display '{display_id}'")]
    PermissionDenied { display_id: String },
    #[error("display '{display_id}' does not expose brightness via VCP 0x10: {message}")]
    UnsupportedMonitor { display_id: String, message: String },
    #[error("ddc operation failed for display '{display_id}': {message}")]
    DdcOperation { display_id: String, message: String },
}

#[derive(Error, Debug)]
pub enum PhysicalDisplayApplyError {
    #[error(transparent)]
    Query {
        #[from]
        source: PhysicalDisplayQueryError,
    },
    #[error("display '{display_id}' does not expose brightness via VCP 0x10: {message}")]
    UnsupportedMonitor { display_id: String, message: String },
    #[error("insufficient permissions for display '{display_id}'")]
    PermissionDenied { display_id: String },
    #[error("missing i2c access for display '{display_id}'")]
    MissingI2cAccess { display_id: String },
    #[error("failed to set brightness for display '{display_id}': {message}")]
    DdcOperation { display_id: String, message: String },
}

pub struct PhysicalDisplayManagerLinux;

#[derive(Clone)]
struct LinuxDisplayHandle {
    metadata: PhysicalDisplayMetadata,
    state: PhysicalDisplayState,
}

impl PhysicalDisplayManagerLinux {
    pub fn query() -> Result<Vec<PhysicalDisplay>, PhysicalDisplayQueryError> {
        Ok(Self::enumerate_handles()?
            .into_iter()
            .map(|handle| PhysicalDisplay {
                metadata: handle.metadata,
                state: handle.state,
            })
            .collect())
    }

    pub(crate) fn apply_display_updates(
        updates: Vec<DisplayUpdate>,
        validate: bool,
    ) -> Result<Vec<DisplayUpdate>, PhysicalDisplayApplyError> {
        if updates.is_empty() {
            return Ok(Vec::new());
        }

        let mut displays = Display::enumerate();
        let identifiers: Vec<_> = displays
            .iter()
            .map(|display| {
                let metadata = metadata_from_info(&display.info);
                Self::id_from_metadata(&metadata)
            })
            .collect();

        let mut remaining_updates = Vec::new();
        let mut matched_updates = Vec::new();

        for update in updates {
            let matched_indices: Vec<_> = identifiers
                .iter()
                .enumerate()
                .filter_map(|(index, id)| update.id.is_subset(&id.outer).then_some(index))
                .collect();

            if matched_indices.is_empty() {
                remaining_updates.push(update);
                continue;
            }

            if validate {
                continue;
            }

            for index in matched_indices {
                matched_updates.push(PhysicalApplyUpdate {
                    id: identifiers[index].clone(),
                    brightness: update
                        .physical
                        .as_ref()
                        .and_then(|physical| physical.brightness),
                    display_index: index,
                });
            }
        }

        for update in matched_updates {
            let Some(brightness) = update.brightness else {
                continue;
            };

            let display = &mut displays[update.display_index];
            let display_id = display.info.id.clone();
            if let Err(err) = Self::set_brightness(display, brightness) {
                tracing::warn!(
                    "Failed to set brightness for display '{}': {}",
                    display_id,
                    err
                );
                remaining_updates.push(DisplayUpdate {
                    id: update.id.outer,
                    logical: None,
                    physical: Some(crate::physical_display::PhysicalDisplayUpdateContent {
                        brightness: Some(brightness),
                    }),
                });
            }
        }

        Ok(remaining_updates)
    }

    fn enumerate_handles() -> Result<Vec<LinuxDisplayHandle>, PhysicalDisplayQueryError> {
        let mut handles = Vec::new();

        for mut display in Display::enumerate() {
            let info = display.info.clone();
            let display_id = info.id.clone();
            let brightness = match display.handle.get_vcp_feature(FeatureCode::from(0x10)) {
                Ok(vcp) => {
                    let maximum = vcp.maximum();
                    if maximum == 0 {
                        tracing::warn!(
                            "Skipping display '{}' because brightness max was reported as 0",
                            display_id
                        );
                        continue;
                    }
                    ((vcp.value() as f64 / maximum as f64) * 100.0).round() as u8
                }
                Err(err) => {
                    let message = err.to_string();
                    if is_io_error(&message) {
                        tracing::warn!(
                            "Assuming 0% brightness for display '{}' due to I/O error: {}",
                            display_id,
                            message
                        );
                        0
                    } else {
                        tracing::warn!(
                            "Skipping display '{}' due to query error: {}",
                            display_id,
                            message
                        );
                        continue;
                    }
                }
            };

            let metadata = metadata_from_info(&info);

            handles.push(LinuxDisplayHandle {
                metadata,
                state: PhysicalDisplayState {
                    brightness: Brightness::new(brightness.min(100)),
                    scale_factor: 100,
                },
            });
        }

        Ok(handles)
    }

    fn set_brightness(
        display: &mut Display,
        brightness: u32,
    ) -> Result<(), PhysicalDisplayApplyError> {
        let ddc_id = display.info.id.clone();
        let vcp = display
            .handle
            .get_vcp_feature(FeatureCode::from(0x10))
            .map_err(|err| classify_apply_error(ddc_id.clone(), err.to_string()))?;

        let max = vcp.maximum();
        if max == 0 {
            return Err(PhysicalDisplayApplyError::UnsupportedMonitor {
                display_id: ddc_id,
                message: "reported brightness max value is 0".to_string(),
            });
        }

        let percent = brightness.min(100) as f64 / 100.0;
        let target_value = (percent * max as f64).round() as u16;

        display
            .handle
            .set_vcp_feature(FeatureCode::from(0x10), target_value)
            .map_err(|err| classify_apply_error(display.info.id.clone(), err.to_string()))
    }

    fn id_from_metadata(metadata: &PhysicalDisplayMetadata) -> DisplayIdentifierInner {
        DisplayIdentifierInner {
            outer: crate::display_identifier::DisplayIdentifier {
                name: Some(metadata.name.clone()),
                serial_number: Some(metadata.serial_number.clone()),
            },
            path: Some(metadata.path.clone()),
            gdi_device_id: None,
        }
    }
}

struct PhysicalApplyUpdate {
    id: DisplayIdentifierInner,
    brightness: Option<u32>,
    display_index: usize,
}

fn metadata_from_info(info: &ddc_hi::DisplayInfo) -> PhysicalDisplayMetadata {
    let name = info
        .model_name
        .clone()
        .unwrap_or_else(|| format!("Display {}", info.id));

    let serial_number = info
        .serial_number
        .clone()
        .or_else(|| info.serial.map(|serial| serial.to_string()))
        .unwrap_or_else(|| format!("fallback-{}", stable_fallback_id(&info.id)));

    PhysicalDisplayMetadata {
        path: info.id.clone(),
        name,
        serial_number,
    }
}

fn stable_fallback_id(value: &str) -> String {
    let mut hash: u64 = 1469598103934665603;
    for byte in value.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("{hash:016x}")
}

fn classify_apply_error(display_id: String, message: String) -> PhysicalDisplayApplyError {
    let lowercase = message.to_lowercase();
    if lowercase.contains("permission denied") {
        return PhysicalDisplayApplyError::PermissionDenied { display_id };
    }
    if lowercase.contains("/dev/i2c") || lowercase.contains("no such file") {
        return PhysicalDisplayApplyError::MissingI2cAccess { display_id };
    }
    if lowercase.contains("unsupported") || lowercase.contains("vcp") {
        return PhysicalDisplayApplyError::UnsupportedMonitor {
            display_id,
            message,
        };
    }
    PhysicalDisplayApplyError::DdcOperation {
        display_id,
        message,
    }
}

fn is_io_error(message: &str) -> bool {
    let lowercase = message.to_lowercase();
    lowercase.contains("input/output error") || lowercase.contains("os error 5")
}
