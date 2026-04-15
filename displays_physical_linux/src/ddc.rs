use std::collections::BTreeMap;
use std::sync::mpsc;
use std::time::Duration;

use ddc_hi::{Ddc, Display as DdcDisplay, DisplayInfo, FeatureCode};

use crate::error::{ApplyError, QueryError};
use crate::types::{
    Backend, DdcApplyUpdate, DisplayHandle, PhysicalDisplayMetadata, PhysicalDisplayState,
    PhysicalDisplayUpdate,
};

const PER_MONITOR_APPLY_TIMEOUT: Duration = Duration::from_millis(3500);

pub(crate) fn enumerate_handles() -> Result<Vec<DisplayHandle>, QueryError> {
    let mut handles = Vec::new();

    for (display_index, mut display) in DdcDisplay::enumerate().into_iter().enumerate() {
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

        handles.push(DisplayHandle {
            metadata: metadata_from_info(&info),
            state: PhysicalDisplayState {
                brightness_percent: brightness.min(100),
                scale_factor: 100,
            },
            backend: Backend::Ddc { display_index },
        });
    }

    Ok(handles)
}

pub(crate) fn apply_updates(updates: Vec<DdcApplyUpdate>) -> Vec<PhysicalDisplayUpdate> {
    if updates.is_empty() {
        return Vec::new();
    }

    let mut remaining_updates = Vec::new();
    let mut display_by_index: BTreeMap<usize, DdcDisplay> =
        DdcDisplay::enumerate().into_iter().enumerate().collect();

    for update in updates {
        let Some(brightness_percent) = update.brightness_percent else {
            continue;
        };

        let Some(display) = display_by_index.remove(&update.display_index) else {
            remaining_updates.push(PhysicalDisplayUpdate {
                id: update.id.outer,
                brightness_percent: Some(brightness_percent),
            });
            continue;
        };

        let display_id = display.info.id.clone();
        if let Err(err) = set_brightness_with_timeout(display, brightness_percent) {
            tracing::warn!(
                "Failed to set brightness for display '{}': {}",
                display_id,
                err
            );
            remaining_updates.push(PhysicalDisplayUpdate {
                id: update.id.outer,
                brightness_percent: Some(brightness_percent),
            });
        }
    }

    remaining_updates
}

fn set_brightness(display: &mut DdcDisplay, brightness_percent: u32) -> Result<(), ApplyError> {
    let ddc_id = display.info.id.clone();
    let normalized = brightness_percent.min(100);
    let target_value = if normalized == 0 {
        0
    } else {
        let vcp = display
            .handle
            .get_vcp_feature(FeatureCode::from(0x10))
            .map_err(|err| classify_apply_error(ddc_id.clone(), err.to_string()))?;

        let max = vcp.maximum();
        if max == 0 {
            return Err(ApplyError::UnsupportedMonitor {
                display_id: ddc_id,
                message: "reported brightness max value is 0".to_string(),
            });
        }

        let percent = normalized as f64 / 100.0;
        (percent * max as f64).round() as u16
    };

    display
        .handle
        .set_vcp_feature(FeatureCode::from(0x10), target_value)
        .map_err(|err| classify_apply_error(display.info.id.clone(), err.to_string()))
}

fn set_brightness_with_timeout(
    display: DdcDisplay,
    brightness_percent: u32,
) -> Result<(), ApplyError> {
    let display_id = display.info.id.clone();
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let mut display = display;
        let result = set_brightness(&mut display, brightness_percent);
        let _ = sender.send(result);
    });

    match receiver.recv_timeout(PER_MONITOR_APPLY_TIMEOUT) {
        Ok(result) => result,
        Err(mpsc::RecvTimeoutError::Timeout) => Err(ApplyError::DdcOperation {
            display_id,
            message: format!("timed out after {:?}", PER_MONITOR_APPLY_TIMEOUT),
        }),
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(ApplyError::DdcOperation {
            display_id,
            message: "apply worker disconnected unexpectedly".to_string(),
        }),
    }
}

fn metadata_from_info(info: &DisplayInfo) -> PhysicalDisplayMetadata {
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

fn classify_apply_error(display_id: String, message: String) -> ApplyError {
    let lowercase = message.to_lowercase();
    if lowercase.contains("permission denied") {
        return ApplyError::PermissionDenied { display_id };
    }
    if lowercase.contains("/dev/i2c") || lowercase.contains("no such file") {
        return ApplyError::MissingI2cAccess { display_id };
    }
    if lowercase.contains("unsupported") || lowercase.contains("vcp") {
        return ApplyError::UnsupportedMonitor {
            display_id,
            message,
        };
    }
    ApplyError::DdcOperation {
        display_id,
        message,
    }
}

fn is_io_error(message: &str) -> bool {
    let lowercase = message.to_lowercase();
    lowercase.contains("input/output error") || lowercase.contains("os error 5")
}
