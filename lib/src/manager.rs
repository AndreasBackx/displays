use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;
use tracing::{debug, instrument, Span};

use crate::{
    display::{Display, DisplayMetadata, DisplayUpdate, DisplayUpdateInner},
    display_identifier::{DisplayIdentifier, DisplayIdentifierInner},
    logical_display::LogicalDisplayUpdate,
    physical_display::PhysicalDisplayUpdate,
    windows::{
        logical_display::LogicalDisplayWindows,
        logical_manager::{
            LogicalDisplayApplyError, LogicalDisplayManagerWindows, LogicalDisplayQueryError,
        },
        physical_display::PhysicalDisplayWindows,
        physical_manager::{
            PhysicalDisplayApplyError, PhysicalDisplayManagerWindows, PhysicalDisplayQueryError,
        },
    },
};

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

pub struct DisplayManager {}

impl DisplayManager {
    #[instrument(ret, level = "trace")]
    pub fn query() -> Result<Vec<Display>, DisplayQueryError> {
        let mut logical_displays_metadata: Vec<_> = LogicalDisplayManagerWindows::metadata()?
            .into_iter()
            .collect();
        // Enabled displays first as we want to return enabled displays ideally.
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
                        physical: physical_metadata,
                    },
                    logical_display.state,
                )
            })
            .collect::<BTreeMap<_, _>>()
            .into_iter()
            .collect::<Vec<(_, _)>>();

        let ids: Vec<_> = logical_state_by_metadata
            .iter()
            // We only want to get the states for the metadata that we've been able to map.
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
                        .map(|physical_state| (physical_metadata, physical_state))
                });

                Display {
                    logical: LogicalDisplayWindows {
                        metadata: metadata.logical,
                        state: logical_state,
                    },
                    physical: physical.map(|(physical_metadata, physical_state)| {
                        PhysicalDisplayWindows {
                            metadata: physical_metadata,
                            state: physical_state,
                        }
                    }),
                }
            })
            .collect())
    }

    #[instrument(ret, skip_all, level = "trace")]
    fn get_inner(
        ids: BTreeSet<DisplayIdentifier>,
    ) -> Result<BTreeMap<DisplayIdentifier, (DisplayIdentifierInner, Display)>, DisplayQueryError>
    {
        let displays = Self::query()?;
        Ok(displays
            .into_iter()
            .filter_map(|displ| {
                let id = displ.id();
                ids.iter()
                    .filter(|user_id| user_id.is_subset(&id.outer))
                    .nth(0)
                    .and_then(|user_id| Some((user_id.clone(), (id, displ))))
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
        let ids: BTreeSet<_> = updates
            .clone()
            .into_iter()
            .map(|update| update.id)
            .collect();
        let mut id_mapping = Self::get_inner(ids)?;
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
        let remaining_logical_updates =
            LogicalDisplayManagerWindows::apply(logical_updates, validate)?;
        let physical_updates: Vec<PhysicalDisplayUpdate> = updates_inner
            .into_iter()
            .filter_map(|display| display.into())
            .collect();
        let mut remaining_physical_updates =
            PhysicalDisplayManagerWindows::apply(physical_updates)?;

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
            // Collecting is done tomake sure `remaining_physical_updates` has had all of its
            // matching items removed.
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

    pub fn update(updates: Vec<DisplayUpdate>) -> Result<Vec<DisplayUpdate>, DisplayApplyError> {
        Self::apply(updates, false)
    }

    pub fn validate(updates: Vec<DisplayUpdate>) -> Result<Vec<DisplayUpdate>, DisplayApplyError> {
        Self::apply(updates, true)
    }
}

pub struct QueryError {}
pub struct ValidateUpdateError {}

pub struct CreationError {}
