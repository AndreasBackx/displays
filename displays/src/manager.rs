use thiserror::Error;

use crate::{
    display::{Display, DisplayUpdate},
    display_identifier::{DisplayIdentifier, DisplayIdentifierInner},
    logical_display::{LogicalDisplay, LogicalDisplayMetadata, LogicalDisplayState},
    types::Orientation,
};

#[cfg(target_os = "windows")]
use std::collections::BTreeMap;

#[cfg(target_os = "windows")]
use crate::{
    display::DisplayMetadata,
    physical_display::{PhysicalDisplay, PhysicalDisplayMetadata, PhysicalDisplayState},
};

#[cfg(target_os = "windows")]
use displays_logical_windows::{
    ApplyError as LogicalDisplayApplyError, LogicalDisplayManager as LogicalDisplayManagerWindows,
    LogicalDisplayUpdate as WindowsLogicalDisplayUpdate,
    Orientation as WindowsOrientation, PixelFormat as WindowsPixelFormat, Point as WindowsPoint,
    QueryError as LogicalDisplayQueryError,
};

#[cfg(target_os = "windows")]
use displays_physical_windows::{
    ApplyError as PhysicalDisplayApplyError,
    PhysicalDisplayManager as PhysicalDisplayManagerWindows,
    PhysicalDisplayMetadata as WindowsPhysicalDisplayMetadata,
    PhysicalDisplayUpdate as WindowsPhysicalDisplayUpdate,
    PhysicalDisplayUpdateContent as WindowsPhysicalDisplayUpdateContent,
    QueryError as PhysicalDisplayQueryError,
};

#[cfg(target_os = "windows")]
use displays_types::{
    DisplayIdentifier as WindowsDisplayIdentifier, DisplayIdentifierInner as WindowsDisplayIdentifierInner,
};

#[cfg(target_os = "linux")]
use displays_physical_linux::{
    ApplyError as PhysicalDisplayApplyError,
    PhysicalDisplayManager as PhysicalDisplayManagerLinux,
    PhysicalDisplayUpdate as LinuxPhysicalDisplayUpdate, QueryError as PhysicalDisplayQueryError,
};

#[cfg(target_os = "linux")]
#[derive(Error, Debug)]
pub enum LogicalDisplayQueryError {
    #[error("logical display query is not supported on Linux")]
    Unsupported,
}

#[cfg(target_os = "linux")]
#[derive(Error, Debug)]
pub enum LogicalDisplayApplyError {
    #[error("logical display updates are not supported on Linux")]
    Unsupported,
}

/// Errors that can occur while querying display state.
#[derive(Error, Debug)]
pub enum DisplayQueryError {
    #[error("physical querying error")]
    Physical {
        #[from]
        source: PhysicalDisplayQueryError,
    },
    #[error("logical querying error")]
    Logical {
        #[from]
        source: LogicalDisplayQueryError,
    },
}

/// Errors that can occur while applying display updates.
#[derive(Error, Debug)]
pub enum DisplayApplyError {
    #[error("error while first querying displays")]
    Query {
        #[from]
        source: DisplayQueryError,
    },
    #[error("physical applying error: {source}")]
    Physical {
        #[from]
        source: PhysicalDisplayApplyError,
    },
    #[error("logical applying error")]
    Logical {
        #[from]
        source: LogicalDisplayApplyError,
    },
}

/// A concrete display matched by a user-facing identifier.
#[derive(Debug, Clone)]
pub struct DisplayMatch {
    pub requested_id: DisplayIdentifier,
    pub matched_id: DisplayIdentifier,
    pub display: Display,
}

/// A per-display failure encountered while applying an update.
#[derive(Debug, Clone)]
pub struct FailedDisplayUpdate {
    pub matched_id: DisplayIdentifier,
    pub message: String,
}

/// Best-effort result for a single requested display update.
#[derive(Debug, Clone)]
pub struct DisplayUpdateResult {
    pub requested_update: DisplayUpdate,
    pub applied: Vec<DisplayIdentifier>,
    pub failed: Vec<FailedDisplayUpdate>,
}

/// High-level entry point for querying and updating displays.
///
/// On Windows, logical and physical display operations are supported.
/// On Linux, display querying and brightness updates are supported, but logical
/// display operations are currently unsupported.
pub struct DisplayManager;

impl DisplayManager {
    /// Queries the current display state.
    #[tracing::instrument(ret, level = "trace")]
    pub fn query() -> Result<Vec<Display>, DisplayQueryError> {
        #[cfg(target_os = "windows")]
        {
            return query_windows();
        }

        #[cfg(target_os = "linux")]
        {
            return query_linux();
        }
    }

    #[tracing::instrument(ret, skip_all, level = "trace")]
    fn get_inner(ids: Vec<DisplayIdentifier>) -> Result<Vec<DisplayMatch>, DisplayQueryError> {
        let displays = Self::query()?;
        Ok(ids
            .into_iter()
            .flat_map(|requested_id| {
                displays
                    .iter()
                    .filter(|display| requested_id.is_subset(&display.id().outer))
                    .map(|display| DisplayMatch {
                        requested_id: requested_id.clone(),
                        matched_id: display.id().outer,
                        display: display.clone(),
                    })
                    .collect::<Vec<_>>()
            })
            .collect())
    }

    /// Looks up displays matching the provided user-facing identifiers.
    pub fn get(ids: Vec<DisplayIdentifier>) -> Result<Vec<DisplayMatch>, DisplayQueryError> {
        Self::get_inner(ids)
    }

    /// Applies the requested display updates.
    ///
    /// When `validate` is `true`, backends may validate updates without applying
    /// them if the platform supports that behavior.
    #[tracing::instrument(ret, level = "trace")]
    pub fn apply(
        updates: Vec<DisplayUpdate>,
        validate: bool,
    ) -> Result<Vec<DisplayUpdateResult>, DisplayApplyError> {
        #[cfg(target_os = "windows")]
        {
            return apply_windows(updates, validate);
        }

        #[cfg(target_os = "linux")]
        {
            return apply_linux(updates, validate);
        }
    }

    /// Applies the requested display updates without validation-only mode.
    pub fn update(
        updates: Vec<DisplayUpdate>,
    ) -> Result<Vec<DisplayUpdateResult>, DisplayApplyError> {
        Self::apply(updates, false)
    }

    /// Validates the requested display updates when supported by the platform backend.
    pub fn validate(
        updates: Vec<DisplayUpdate>,
    ) -> Result<Vec<DisplayUpdateResult>, DisplayApplyError> {
        Self::apply(updates, true)
    }
}

#[cfg(target_os = "windows")]
fn query_windows() -> Result<Vec<Display>, DisplayQueryError> {
    let mut logical_displays_metadata: Vec<_> =
        LogicalDisplayManagerWindows::query()?.into_iter().collect();
    logical_displays_metadata.sort_by_key(|logical| !logical.state.is_enabled);
    let mut physical_metadatas = PhysicalDisplayManagerWindows::metadata()?;

    let logical_state_by_metadata = logical_displays_metadata
        .into_iter()
        .map(|logical_display| {
            let physical_metadata = physical_metadatas
                .iter()
                .position(|physical_metadata| {
                    logical_display
                        .metadata
                        .path
                        .starts_with(&physical_metadata.path)
                })
                .map(|position| physical_metadatas.remove(position));

            (
                DisplayMetadata {
                    logical: logical_display.metadata,
                    physical: physical_metadata.map(Into::into),
                },
                logical_display.state,
            )
        })
        .collect::<BTreeMap<_, _>>()
        .into_iter()
        .collect::<Vec<(_, _)>>();

    let ids: Vec<_> = logical_state_by_metadata
        .iter()
        .filter(|(metadata, _)| metadata.physical.is_some())
        .map(|(metadata, _)| to_windows_display_identifier_inner(metadata.id()))
        .collect();

    let mut physical_states = PhysicalDisplayManagerWindows::state(ids)?;

    Ok(logical_state_by_metadata
        .into_iter()
        .map(|(metadata, logical_state)| {
            let id = to_windows_display_identifier_inner(metadata.id());

            let physical = metadata.physical.and_then(|physical_metadata| {
                physical_states.remove(&id).map(|physical_state| {
                    (
                        physical_metadata,
                        PhysicalDisplayState {
                            brightness: physical_state.brightness.into(),
                            scale_factor: physical_state.scale_factor,
                        },
                    )
                })
            });

            Display {
                logical: LogicalDisplay {
                    metadata: metadata.logical,
                    state: logical_state,
                },
                physical: physical.map(|(physical_metadata, physical_state)| PhysicalDisplay {
                    metadata: physical_metadata,
                    state: physical_state,
                }),
            }
        })
        .collect())
}

#[cfg(target_os = "windows")]
fn apply_windows(
    updates: Vec<DisplayUpdate>,
    validate: bool,
) -> Result<Vec<DisplayUpdateResult>, DisplayApplyError> {
    let matched_updates = matched_updates(updates)?;
    let mut results = Vec::with_capacity(matched_updates.len());

    for (requested_update, matched_ids) in matched_updates {
        let mut result = DisplayUpdateResult {
            requested_update: requested_update.clone(),
            applied: Vec::new(),
            failed: Vec::new(),
        };

        for matched_id in matched_ids {
            if let Some(logical_content) = requested_update.logical.clone() {
                let logical_update = WindowsLogicalDisplayUpdate {
                    id: to_windows_display_identifier_inner(matched_id.clone()),
                    content: logical_content,
                };

                match LogicalDisplayManagerWindows::apply(vec![logical_update], validate) {
                    Ok(remaining) if remaining.is_empty() => {}
                    Ok(_) => {
                        result.failed.push(FailedDisplayUpdate {
                            matched_id: matched_id.outer,
                            message: "logical update was not applied".to_string(),
                        });
                        continue;
                    }
                    Err(err) => {
                        result.failed.push(FailedDisplayUpdate {
                            matched_id: matched_id.outer,
                            message: err.to_string(),
                        });
                        continue;
                    }
                }
            }

            if validate || requested_update.physical.is_none() {
                if !result.applied.contains(&matched_id) {
                    result.applied.push(matched_id.outer);
                }
                continue;
            }

            let physical_content = requested_update.physical.clone().expect("checked above");
            let physical_update = WindowsPhysicalDisplayUpdate {
                id: to_windows_display_identifier_inner(matched_id.clone()),
                content: WindowsPhysicalDisplayUpdateContent {
                    brightness: physical_content.brightness,
                },
            };

            match PhysicalDisplayManagerWindows::apply(vec![physical_update]) {
                Ok(remaining) if remaining.is_empty() => result.applied.push(matched_id.outer),
                Ok(_) => result.failed.push(FailedDisplayUpdate {
                    matched_id: matched_id.outer,
                    message: "physical update was not applied".to_string(),
                }),
                Err(err) => result.failed.push(FailedDisplayUpdate {
                    matched_id: matched_id.outer,
                    message: err.to_string(),
                }),
            }
        }

        results.push(result);
    }

    Ok(results)
}

#[cfg(target_os = "windows")]
fn to_windows_display_identifier_inner(
    id: DisplayIdentifierInner,
) -> WindowsDisplayIdentifierInner {
    WindowsDisplayIdentifierInner {
        outer: WindowsDisplayIdentifier {
            name: id.outer.name,
            serial_number: id.outer.serial_number,
        },
        path: id.path,
        gdi_device_id: id.gdi_device_id,
    }
}

#[cfg(target_os = "windows")]
fn to_windows_orientation(value: Orientation) -> WindowsOrientation {
    match value {
        Orientation::Landscape => WindowsOrientation::Landscape,
        Orientation::Portrait => WindowsOrientation::Portrait,
        Orientation::LandscapeFlipped => WindowsOrientation::LandscapeFlipped,
        Orientation::PortraitFlipped => WindowsOrientation::PortraitFlipped,
    }
}

#[cfg(target_os = "windows")]
fn from_windows_orientation(value: WindowsOrientation) -> Orientation {
    match value {
        WindowsOrientation::Landscape => Orientation::Landscape,
        WindowsOrientation::Portrait => Orientation::Portrait,
        WindowsOrientation::LandscapeFlipped => Orientation::LandscapeFlipped,
        WindowsOrientation::PortraitFlipped => Orientation::PortraitFlipped,
    }
}

#[cfg(target_os = "windows")]
fn to_windows_pixel_format(value: crate::types::PixelFormat) -> WindowsPixelFormat {
    match value {
        crate::types::PixelFormat::BPP8 => WindowsPixelFormat::BPP8,
        crate::types::PixelFormat::BPP16 => WindowsPixelFormat::BPP16,
        crate::types::PixelFormat::BPP24 => WindowsPixelFormat::BPP24,
        crate::types::PixelFormat::BPP32 => WindowsPixelFormat::BPP32,
        crate::types::PixelFormat::NONGDI => WindowsPixelFormat::NONGDI,
    }
}

#[cfg(target_os = "windows")]
fn from_windows_pixel_format(value: WindowsPixelFormat) -> crate::types::PixelFormat {
    match value {
        WindowsPixelFormat::BPP8 => crate::types::PixelFormat::BPP8,
        WindowsPixelFormat::BPP16 => crate::types::PixelFormat::BPP16,
        WindowsPixelFormat::BPP24 => crate::types::PixelFormat::BPP24,
        WindowsPixelFormat::BPP32 => crate::types::PixelFormat::BPP32,
        WindowsPixelFormat::NONGDI => crate::types::PixelFormat::NONGDI,
    }
}

#[cfg(target_os = "windows")]
fn to_windows_point(value: crate::types::Point) -> WindowsPoint {
    WindowsPoint {
        x: value.x,
        y: value.y,
    }
}

#[cfg(target_os = "windows")]
fn from_windows_point(value: WindowsPoint) -> crate::types::Point {
    crate::types::Point {
        x: value.x,
        y: value.y,
    }
}

#[cfg(target_os = "windows")]
impl From<WindowsPhysicalDisplayMetadata> for PhysicalDisplayMetadata {
    fn from(value: WindowsPhysicalDisplayMetadata) -> Self {
        Self {
            path: value.path,
            name: value.name,
            serial_number: value.serial_number,
        }
    }
}

#[cfg(target_os = "linux")]
fn query_linux() -> Result<Vec<Display>, DisplayQueryError> {
    let physical_displays = PhysicalDisplayManagerLinux::query()?;

    Ok(physical_displays
        .into_iter()
        .map(|physical| {
            let logical_name = physical.metadata.name.clone();
            let logical_path = physical.metadata.path.clone();
            Display {
                logical: LogicalDisplay {
                    metadata: LogicalDisplayMetadata {
                        name: logical_name,
                        path: logical_path,
                        #[cfg(target_os = "windows")]
                        gdi_device_id: None,
                    },
                    state: LogicalDisplayState {
                        is_enabled: true,
                        orientation: Orientation::Landscape,
                        width: None,
                        height: None,
                        pixel_format: None,
                        position: None,
                    },
                },
                physical: Some(physical),
            }
        })
        .collect())
}

#[cfg(target_os = "linux")]
fn apply_linux(
    updates: Vec<DisplayUpdate>,
    validate: bool,
) -> Result<Vec<DisplayUpdateResult>, DisplayApplyError> {
    if updates.iter().any(|update| update.logical.is_some()) {
        return Err(LogicalDisplayApplyError::Unsupported.into());
    }

    let matched_updates = matched_updates(updates)?;
    let mut results = Vec::with_capacity(matched_updates.len());

    for (requested_update, matched_ids) in matched_updates {
        let mut result = DisplayUpdateResult {
            requested_update: requested_update.clone(),
            applied: Vec::new(),
            failed: Vec::new(),
        };

        for matched_id in matched_ids {
            let linux_update = LinuxPhysicalDisplayUpdate {
                id: matched_id.outer.clone(),
                content: requested_update.physical.clone().unwrap_or_default(),
            };

            match PhysicalDisplayManagerLinux::apply(vec![linux_update], validate) {
                Ok(remaining) if remaining.is_empty() => result.applied.push(matched_id.outer),
                Ok(_) => result.failed.push(FailedDisplayUpdate {
                    matched_id: matched_id.outer,
                    message: "physical update was not applied".to_string(),
                }),
                Err(err) => result.failed.push(FailedDisplayUpdate {
                    matched_id: matched_id.outer,
                    message: err.to_string(),
                }),
            }
        }

        results.push(result);
    }

    Ok(results)
}

fn matched_updates(
    updates: Vec<DisplayUpdate>,
) -> Result<Vec<(DisplayUpdate, Vec<DisplayIdentifierInner>)>, DisplayQueryError> {
    let displays = DisplayManager::query()?;
    Ok(updates
        .into_iter()
        .map(|update| {
            let matched_ids = displays
                .iter()
                .filter(|display| update.id.is_subset(&display.id().outer))
                .map(Display::id)
                .collect();
            (update, matched_ids)
        })
        .collect())
}

pub struct QueryError {}
pub struct ValidateUpdateError {}
pub struct CreationError {}
