use std::collections::{BTreeMap, BTreeSet};

use crate::{
    display::{
        Display, DisplayIdentifier, DisplayIdentifierInner, DisplayUpdate, DisplayUpdateInner,
    },
    windows::{
        logical_display::LogicalDisplayUpdate, logical_manager::LogicalDisplayManagerWindows,
        physical_display::PhysicalDisplayUpdate, physical_manager::PhysicalDisplayManagerWindows,
    },
};

pub struct Displays {}

impl Displays {
    pub fn query() -> anyhow::Result<Vec<Display>> {
        let logical_displays = LogicalDisplayManagerWindows::query()?;
        let mut physical_displays = PhysicalDisplayManagerWindows::query()?;

        Ok(logical_displays
            .into_iter()
            .filter_map(|logical_display| {
                physical_displays
                    .iter()
                    .position(|physical_display| {
                        logical_display
                            .target
                            .path
                            .starts_with(&physical_display.path)
                    })
                    .map(|position| physical_displays.remove(position))
                    .map(|physical_display| Display {
                        logical: logical_display,
                        physical: physical_display,
                    })
            })
            .collect())
    }

    fn get_inner(
        ids: BTreeSet<&DisplayIdentifier>,
    ) -> anyhow::Result<BTreeMap<DisplayIdentifier, (DisplayIdentifierInner, Display)>> {
        let displays = Self::query()?;
        Ok(displays
            .into_iter()
            .map(|display| {
                let id = display.id();
                (id.outer.clone(), (id, display))
            })
            .filter(|(id, _)| ids.contains(&id))
            .collect())
    }

    pub fn get(
        ids: BTreeSet<&DisplayIdentifier>,
    ) -> anyhow::Result<BTreeMap<DisplayIdentifier, Display>> {
        Self::get_inner(ids).map(|display_by_id| {
            display_by_id
                .into_iter()
                .map(|(id, (_, display))| (id, display))
                .collect()
        })
    }

    fn apply(updates: Vec<DisplayUpdate>, validate: bool) -> anyhow::Result<Vec<DisplayUpdate>> {
        let ids = updates.iter().map(|update| &update.id).collect();
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

    pub fn update(updates: Vec<DisplayUpdate>) -> anyhow::Result<Vec<DisplayUpdate>> {
        Self::apply(updates, false)
    }

    pub fn validate(updates: Vec<DisplayUpdate>) -> anyhow::Result<Vec<DisplayUpdate>> {
        Self::apply(updates, true)
    }
}

pub struct QueryError {}
pub struct ValidateUpdateError {}

pub struct CreationError {}
