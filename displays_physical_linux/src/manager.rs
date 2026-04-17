use displays_physical_linux_logind::PhysicalDisplayManagerLinuxLogind;
use displays_physical_linux_sys::{
    BrightnessUpdate, Device, DeviceClass, DeviceIdentifier, DeviceUpdate,
    PhysicalDisplayManagerLinuxSys,
};
use displays_types::Brightness;
use std::io::ErrorKind;

use crate::ddc;
use crate::error::{ApplyError, QueryError};
use crate::types::{
    Backend, BacklightApplyUpdate, DdcApplyUpdate, DisplayHandle, PhysicalDisplay,
    PhysicalDisplayMetadata, PhysicalDisplayState, PhysicalDisplayUpdate, PhysicalDisplayUpdateContent,
};

/// High-level entry point for querying and updating Linux physical displays.
pub struct PhysicalDisplayManager;

impl PhysicalDisplayManager {
    /// Queries the current Linux physical display state.
    pub fn query() -> Result<Vec<PhysicalDisplay>, QueryError> {
        Ok(Self::query_handles()?
            .into_iter()
            .map(|handle| handle.display())
            .collect())
    }

    /// Applies the requested Linux physical display updates.
    pub fn apply(
        updates: Vec<PhysicalDisplayUpdate>,
        validate: bool,
    ) -> Result<Vec<PhysicalDisplayUpdate>, ApplyError> {
        if updates.is_empty() {
            return Ok(Vec::new());
        }

        let handles = Self::query_handles()?;
        let mut remaining_updates = Vec::new();
        let mut ddc_updates = Vec::new();
        let mut backlight_updates = Vec::new();

        for update in updates {
            let matched_handles: Vec<_> = handles
                .iter()
                .filter(|handle| update.id.is_subset(&handle.id().outer))
                .collect();

            if matched_handles.is_empty() {
                remaining_updates.push(update);
                continue;
            }

            if validate {
                continue;
            }

            for handle in matched_handles {
                match &handle.backend {
                    Backend::Ddc { display_index } => {
                        ddc_updates.push(DdcApplyUpdate {
                            id: handle.id(),
                            content: update.content.clone(),
                            display_index: *display_index,
                        });
                    }
                    Backend::Backlight { path } => {
                        backlight_updates.push(BacklightApplyUpdate {
                            id: handle.id(),
                            content: update.content.clone(),
                            path: path.clone(),
                        });
                    }
                }
            }
        }

        remaining_updates.extend(ddc::apply_updates(ddc_updates));
        remaining_updates.extend(Self::apply_backlight_updates(backlight_updates)?);
        Ok(remaining_updates)
    }

    /// Applies the requested Linux physical display updates without validation-only mode.
    pub fn update(
        updates: Vec<PhysicalDisplayUpdate>,
    ) -> Result<Vec<PhysicalDisplayUpdate>, ApplyError> {
        Self::apply(updates, false)
    }

    /// Validates the requested Linux physical display updates.
    pub fn validate(
        updates: Vec<PhysicalDisplayUpdate>,
    ) -> Result<Vec<PhysicalDisplayUpdate>, ApplyError> {
        Self::apply(updates, true)
    }

    fn query_handles() -> Result<Vec<DisplayHandle>, QueryError> {
        let mut handles = ddc::enumerate_handles()?;
        handles.extend(Self::enumerate_backlight_handles()?);
        handles.sort_by(|left, right| left.metadata.cmp(&right.metadata));
        Ok(handles)
    }

    fn enumerate_backlight_handles() -> Result<Vec<DisplayHandle>, QueryError> {
        let manager = PhysicalDisplayManagerLinuxSys::new();
        match manager.list_by_classes([DeviceClass::Backlight]) {
            Ok(devices) => devices
                .into_iter()
                .map(backlight_handle_from_device)
                .collect(),
            Err(displays_physical_linux_sys::QueryError::ReadClassDirectory { source, .. })
                if source.kind() == ErrorKind::NotFound =>
            {
                Ok(Vec::new())
            }
            Err(err) => Err(QueryError::BacklightQuery {
                message: err.to_string(),
            }),
        }
    }

    fn apply_backlight_updates(
        updates: Vec<BacklightApplyUpdate>,
    ) -> Result<Vec<PhysicalDisplayUpdate>, ApplyError> {
        if updates.is_empty() {
            return Ok(Vec::new());
        }

        let sys = PhysicalDisplayManagerLinuxSys::new();
        let logind = PhysicalDisplayManagerLinuxLogind::new();
        let mut remaining_updates = Vec::new();

        for update in updates {
            let Some(brightness_percent) = update.content.brightness else {
                continue;
            };

            let request = DeviceUpdate {
                id: DeviceIdentifier {
                    class: Some(DeviceClass::Backlight),
                    id: None,
                    path: Some(update.path.clone()),
                },
                brightness: Some(BrightnessUpdate::Percent(brightness_percent.min(100) as u8)),
            };

            match sys.update(vec![request.clone()]) {
                Ok(remaining) if remaining.is_empty() => {}
                Ok(_) => remaining_updates.push(PhysicalDisplayUpdate {
                    id: update.id.outer,
                    content: PhysicalDisplayUpdateContent {
                        brightness: Some(brightness_percent),
                    },
                }),
                Err(displays_physical_linux_sys::ApplyError::WriteFile { source, .. })
                    if source.kind() == ErrorKind::PermissionDenied =>
                {
                    let devices = sys
                        .list_by_classes([DeviceClass::Backlight])
                        .map_err(|err| ApplyError::BacklightOperation {
                            display_id: update.path.clone(),
                            message: err.to_string(),
                        })?;
                    let Some(device) = devices
                        .iter()
                        .find(|device| request.id.is_subset(&device.metadata))
                    else {
                        remaining_updates.push(PhysicalDisplayUpdate {
                            id: update.id.outer,
                            content: PhysicalDisplayUpdateContent {
                                brightness: Some(brightness_percent),
                            },
                        });
                        continue;
                    };
                    let target_raw = displays_physical_linux_sys::normalize_brightness_update(
                        &device.state,
                        request
                            .brightness
                            .as_ref()
                            .expect("backlight request has brightness"),
                    );

                    if let Err(err) = logind.set_brightness(
                        device.metadata.class,
                        &device.metadata.id,
                        target_raw,
                    ) {
                        return Err(ApplyError::BacklightOperation {
                            display_id: device.metadata.path.clone(),
                            message: err.to_string(),
                        });
                    }
                }
                Err(err) => {
                    tracing::warn!(
                        "Failed to set backlight brightness for display '{}': {}",
                        update.path,
                        err
                    );
                    remaining_updates.push(PhysicalDisplayUpdate {
                        id: update.id.outer,
                        content: PhysicalDisplayUpdateContent {
                            brightness: Some(brightness_percent),
                        },
                    });
                }
            }
        }

        Ok(remaining_updates)
    }
}

fn backlight_handle_from_device(device: Device) -> Result<DisplayHandle, QueryError> {
    let path = device.metadata.path;
    let name = device.metadata.id;

    Ok(DisplayHandle {
        metadata: PhysicalDisplayMetadata {
            path: path.clone(),
            name,
            serial_number: String::new(),
        },
        state: PhysicalDisplayState {
            brightness: Brightness::new(device.state.brightness_percent),
            scale_factor: 100,
        },
        backend: Backend::Backlight { path },
    })
}
