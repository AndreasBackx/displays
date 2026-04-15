use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use crate::{
    display::{Display, DisplayUpdate},
    display_identifier::{DisplayIdentifier, DisplayIdentifierInner},
    logical_display::{LogicalDisplay, LogicalDisplayMetadata, LogicalDisplayState},
    types::Orientation,
};

#[cfg(target_os = "windows")]
use crate::{
    display::DisplayMetadata,
    physical_display::{PhysicalDisplay, PhysicalDisplayMetadata, PhysicalDisplayState},
};

#[cfg(target_os = "windows")]
use displays_logical_windows::{
    ApplyError as LogicalDisplayApplyError, LogicalDisplayManager as LogicalDisplayManagerWindows,
    LogicalDisplayMetadata as WindowsLogicalDisplayMetadata,
    LogicalDisplayState as WindowsLogicalDisplayState,
    LogicalDisplayUpdate as WindowsLogicalDisplayUpdate,
    LogicalDisplayUpdateContent as WindowsLogicalDisplayUpdateContent,
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
use displays_windows_common::types::{
    Brightness as WindowsBrightness, DisplayIdentifier as WindowsDisplayIdentifier,
    DisplayIdentifierInner as WindowsDisplayIdentifierInner,
};

#[cfg(target_os = "linux")]
use crate::physical_display::{
    PhysicalDisplay, PhysicalDisplayMetadata, PhysicalDisplayState, PhysicalDisplayUpdateContent,
};

#[cfg(target_os = "linux")]
use displays_physical_linux::{
    ApplyError as PhysicalDisplayApplyError,
    PhysicalDisplayIdentifier as LinuxPhysicalDisplayIdentifier,
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
    fn get_inner(
        ids: BTreeSet<DisplayIdentifier>,
    ) -> Result<BTreeMap<DisplayIdentifier, (DisplayIdentifierInner, Display)>, DisplayQueryError>
    {
        let displays = Self::query()?;
        Ok(displays
            .into_iter()
            .filter_map(|display| {
                let id = display.id();
                ids.iter()
                    .find(|user_id| user_id.is_subset(&id.outer))
                    .map(|user_id| (user_id.clone(), (id, display)))
            })
            .collect())
    }

    /// Looks up displays matching the provided user-facing identifiers.
    pub fn get(
        ids: BTreeSet<DisplayIdentifier>,
    ) -> Result<BTreeMap<DisplayIdentifier, Display>, DisplayQueryError> {
        Self::get_inner(ids).map(|display_by_id| {
            display_by_id
                .into_iter()
                .map(|(id, (_, display))| (id, display))
                .collect()
        })
    }

    /// Applies the requested display updates.
    ///
    /// When `validate` is `true`, backends may validate updates without applying
    /// them if the platform supports that behavior.
    #[tracing::instrument(ret, level = "trace")]
    pub fn apply(
        updates: Vec<DisplayUpdate>,
        validate: bool,
    ) -> Result<Vec<DisplayUpdate>, DisplayApplyError> {
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
    pub fn update(updates: Vec<DisplayUpdate>) -> Result<Vec<DisplayUpdate>, DisplayApplyError> {
        Self::apply(updates, false)
    }

    /// Validates the requested display updates when supported by the platform backend.
    pub fn validate(updates: Vec<DisplayUpdate>) -> Result<Vec<DisplayUpdate>, DisplayApplyError> {
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
                    logical: logical_display.metadata.into(),
                    physical: physical_metadata.map(Into::into),
                },
                logical_display.state.into(),
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
) -> Result<Vec<DisplayUpdate>, DisplayApplyError> {
    let ids: BTreeSet<_> = updates
        .clone()
        .into_iter()
        .map(|update| update.id)
        .collect();
    let mut id_mapping = DisplayManager::get_inner(ids)?;
    let updates_inner: Vec<_> = updates
        .into_iter()
        .filter_map(|update| {
            id_mapping
                .remove(&update.id)
                .map(|(id_inner, _display)| (id_inner, update.logical, update.physical))
        })
        .collect();

    let logical_updates: Vec<WindowsLogicalDisplayUpdate> = updates_inner
        .clone()
        .into_iter()
        .filter_map(|(id, logical, _physical)| {
            logical.map(|content| WindowsLogicalDisplayUpdate {
                id: to_windows_display_identifier_inner(id),
                content: to_windows_logical_update_content(content),
            })
        })
        .collect();
    let remaining_logical_updates = LogicalDisplayManagerWindows::apply(logical_updates, validate)?;

    let physical_updates: Vec<WindowsPhysicalDisplayUpdate> = if validate {
        vec![]
    } else {
        updates_inner
            .into_iter()
            .filter_map(|(id, _logical, physical)| {
                physical.map(|content| WindowsPhysicalDisplayUpdate {
                    id: to_windows_display_identifier_inner(id),
                    content: WindowsPhysicalDisplayUpdateContent {
                        brightness: content.brightness,
                    },
                })
            })
            .collect()
    };
    let mut remaining_physical_updates = PhysicalDisplayManagerWindows::apply(physical_updates)?;

    let remaining_updates = remaining_logical_updates
        .into_iter()
        .map(|logical_update| DisplayUpdate {
            id: from_windows_display_identifier(logical_update.id.outer.clone()),
            logical: Some(from_windows_logical_update_content(logical_update.content)),
            physical: remaining_physical_updates
                .iter()
                .position(|physical_update| physical_update.id == logical_update.id)
                .map(|index| {
                    let content = remaining_physical_updates.remove(index).content;
                    crate::physical_display::PhysicalDisplayUpdateContent {
                        brightness: content.brightness,
                    }
                }),
        })
        .collect::<Vec<_>>()
        .into_iter()
        .chain(
            remaining_physical_updates
                .into_iter()
                .map(|physical_update| DisplayUpdate {
                    id: from_windows_display_identifier(physical_update.id.outer),
                    physical: Some(crate::physical_display::PhysicalDisplayUpdateContent {
                        brightness: physical_update.content.brightness,
                    }),
                    logical: None,
                }),
        )
        .collect();
    Ok(remaining_updates)
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
fn from_windows_display_identifier(id: WindowsDisplayIdentifier) -> DisplayIdentifier {
    DisplayIdentifier {
        name: id.name,
        serial_number: id.serial_number,
    }
}

#[cfg(target_os = "windows")]
fn to_windows_logical_update_content(
    content: crate::logical_display::LogicalDisplayUpdateContent,
) -> WindowsLogicalDisplayUpdateContent {
    WindowsLogicalDisplayUpdateContent {
        is_enabled: content.is_enabled,
        orientation: content.orientation.map(to_windows_orientation),
        width: content.width,
        height: content.height,
        pixel_format: content.pixel_format.map(to_windows_pixel_format),
        position: content.position.map(to_windows_point),
    }
}

#[cfg(target_os = "windows")]
fn from_windows_logical_update_content(
    content: WindowsLogicalDisplayUpdateContent,
) -> crate::logical_display::LogicalDisplayUpdateContent {
    crate::logical_display::LogicalDisplayUpdateContent {
        is_enabled: content.is_enabled,
        orientation: content.orientation.map(from_windows_orientation),
        width: content.width,
        height: content.height,
        pixel_format: content.pixel_format.map(from_windows_pixel_format),
        position: content.position.map(from_windows_point),
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
impl From<WindowsLogicalDisplayMetadata> for LogicalDisplayMetadata {
    fn from(value: WindowsLogicalDisplayMetadata) -> Self {
        Self {
            name: value.name,
            path: value.path,
            gdi_device_id: value.gdi_device_id,
        }
    }
}

#[cfg(target_os = "windows")]
impl From<WindowsLogicalDisplayState> for LogicalDisplayState {
    fn from(value: WindowsLogicalDisplayState) -> Self {
        Self {
            is_enabled: value.is_enabled,
            orientation: from_windows_orientation(value.orientation),
            width: value.width,
            height: value.height,
            pixel_format: value.pixel_format.map(from_windows_pixel_format),
            position: value.position.map(from_windows_point),
        }
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

#[cfg(target_os = "windows")]
impl From<WindowsBrightness> for crate::display::Brightness {
    fn from(value: WindowsBrightness) -> Self {
        crate::display::Brightness::new(value.value())
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
                physical: Some(PhysicalDisplay {
                    metadata: PhysicalDisplayMetadata {
                        path: physical.metadata.path,
                        name: physical.metadata.name,
                        serial_number: physical.metadata.serial_number,
                    },
                    state: PhysicalDisplayState {
                        brightness: crate::display::Brightness::new(
                            physical.state.brightness_percent,
                        ),
                        scale_factor: physical.state.scale_factor,
                    },
                }),
            }
        })
        .collect())
}

#[cfg(target_os = "linux")]
fn apply_linux(
    updates: Vec<DisplayUpdate>,
    validate: bool,
) -> Result<Vec<DisplayUpdate>, DisplayApplyError> {
    if updates.iter().any(|update| update.logical.is_some()) {
        return Err(LogicalDisplayApplyError::Unsupported.into());
    }

    let linux_updates = updates
        .into_iter()
        .map(|update| LinuxPhysicalDisplayUpdate {
            id: LinuxPhysicalDisplayIdentifier {
                name: update.id.name,
                serial_number: update.id.serial_number,
            },
            brightness_percent: update
                .physical
                .as_ref()
                .and_then(|physical| physical.brightness),
        })
        .collect();

    PhysicalDisplayManagerLinux::apply(linux_updates, validate)
        .map(|remaining| {
            remaining
                .into_iter()
                .map(|update| DisplayUpdate {
                    id: DisplayIdentifier {
                        name: update.id.name,
                        serial_number: update.id.serial_number,
                    },
                    logical: None,
                    physical: Some(PhysicalDisplayUpdateContent {
                        brightness: update.brightness_percent,
                    }),
                })
                .collect()
        })
        .map_err(Into::into)
}

pub struct QueryError {}
pub struct ValidateUpdateError {}
pub struct CreationError {}
