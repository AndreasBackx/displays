use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use crate::{
    display::{Display, DisplayUpdate, DisplayUpdateInner},
    display_identifier::{DisplayIdentifier, DisplayIdentifierInner},
    logical_display::{LogicalDisplay, LogicalDisplayMetadata, LogicalDisplayState},
    physical_display::PhysicalDisplayUpdate,
    types::Orientation,
};

#[cfg(target_os = "windows")]
use crate::{
    display::DisplayMetadata, logical_display::LogicalDisplayUpdate,
    physical_display::PhysicalDisplay,
};

#[cfg(target_os = "windows")]
use crate::windows::{
    logical_manager::{
        LogicalDisplayApplyError, LogicalDisplayManagerWindows, LogicalDisplayQueryError,
    },
    physical_manager::{
        PhysicalDisplayApplyError, PhysicalDisplayManagerWindows, PhysicalDisplayQueryError,
    },
};

#[cfg(target_os = "linux")]
use crate::linux::physical_manager::{
    PhysicalDisplayApplyError, PhysicalDisplayManagerLinux, PhysicalDisplayQueryError,
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

pub struct DisplayManager;

impl DisplayManager {
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

    pub fn update(updates: Vec<DisplayUpdate>) -> Result<Vec<DisplayUpdate>, DisplayApplyError> {
        Self::apply(updates, false)
    }

    pub fn validate(updates: Vec<DisplayUpdate>) -> Result<Vec<DisplayUpdate>, DisplayApplyError> {
        Self::apply(updates, true)
    }
}

#[cfg(target_os = "windows")]
fn query_windows() -> Result<Vec<Display>, DisplayQueryError> {
    let mut logical_displays_metadata: Vec<_> = LogicalDisplayManagerWindows::metadata()?
        .into_iter()
        .collect();
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
        .map(|(metadata, _)| metadata.id())
        .collect();

    let mut physical_states = PhysicalDisplayManagerWindows::state(ids)?;

    Ok(logical_state_by_metadata
        .into_iter()
        .map(|(metadata, logical_state)| {
            let id = metadata.id();

            let physical = metadata.physical.and_then(|physical_metadata| {
                physical_states
                    .remove(&id)
                    .map(|physical_state| (physical_metadata, physical_state.into()))
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
                .map(|(id_inner, _display)| DisplayUpdateInner {
                    id: id_inner,
                    logical: update.logical,
                    physical: update.physical,
                })
        })
        .collect();

    let logical_updates: Vec<LogicalDisplayUpdate> = updates_inner
        .clone()
        .into_iter()
        .filter_map(|display| display.into())
        .collect();
    let remaining_logical_updates = LogicalDisplayManagerWindows::apply(logical_updates, validate)?;

    let physical_updates: Vec<PhysicalDisplayUpdate> = if validate {
        vec![]
    } else {
        updates_inner
            .into_iter()
            .filter_map(|display| display.into())
            .collect()
    };
    let mut remaining_physical_updates = PhysicalDisplayManagerWindows::apply(physical_updates)?;

    let remaining_updates = remaining_logical_updates
        .into_iter()
        .map(|logical_update| DisplayUpdate {
            id: logical_update.id.outer.clone(),
            logical: Some(logical_update.content),
            physical: remaining_physical_updates
                .iter()
                .position(|physical_update| physical_update.id == logical_update.id)
                .map(|index| remaining_physical_updates.remove(index).content),
        })
        .collect::<Vec<_>>()
        .into_iter()
        .chain(
            remaining_physical_updates
                .into_iter()
                .map(|physical_update| DisplayUpdate {
                    id: physical_update.id.outer,
                    physical: Some(physical_update.content),
                    logical: None,
                }),
        )
        .collect();
    Ok(remaining_updates)
}

#[cfg(target_os = "linux")]
fn query_linux() -> Result<Vec<Display>, DisplayQueryError> {
    let physical_displays = PhysicalDisplayManagerLinux::query()?;

    Ok(physical_displays
        .into_iter()
        .map(|physical| Display {
            logical: LogicalDisplay {
                metadata: LogicalDisplayMetadata {
                    name: physical.metadata.name.clone(),
                    path: physical.metadata.path.clone(),
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

    let displays = DisplayManager::query()?;
    let mut updates_inner = Vec::new();
    let mut unmatched_updates = Vec::new();

    for update in updates {
        let matched_ids: Vec<_> = displays
            .iter()
            .map(|display| display.id())
            .filter(|id| update.id.is_subset(&id.outer))
            .collect();

        if matched_ids.is_empty() {
            unmatched_updates.push(update);
            continue;
        }

        for id in matched_ids {
            updates_inner.push(DisplayUpdateInner {
                id,
                logical: None,
                physical: update.physical.clone(),
            });
        }
    }

    let physical_updates: Vec<PhysicalDisplayUpdate> = if validate {
        vec![]
    } else {
        updates_inner
            .into_iter()
            .filter_map(|display| display.into())
            .collect()
    };

    let remaining_physical_updates = PhysicalDisplayManagerLinux::apply(physical_updates)?;
    Ok(remaining_physical_updates
        .into_iter()
        .map(|physical_update| DisplayUpdate {
            id: physical_update.id.outer,
            physical: Some(physical_update.content),
            logical: None,
        })
        .chain(unmatched_updates)
        .collect())
}

pub struct QueryError {}
pub struct ValidateUpdateError {}
pub struct CreationError {}
