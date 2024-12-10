use std::collections::{BTreeMap, BTreeSet};

use tracing::{debug, instrument};

use crate::{
    display::{
        Display, DisplayIdentifier, DisplayIdentifierInner, DisplayUpdate, DisplayUpdateInner,
    },
    windows::{
        logical_display::LogicalDisplayUpdate, logical_manager::LogicalDisplayManagerWindows,
        physical_display::PhysicalDisplayUpdate, physical_manager::PhysicalDisplayManagerWindows,
    },
};

pub struct DisplayManager {}

impl DisplayManager {
    #[instrument(ret)]
    pub fn query() -> anyhow::Result<Vec<Display>> {
        let mut logical_displays: Vec<_> =
            LogicalDisplayManagerWindows::query()?.into_iter().collect();
        // Enabled displays first as we want to return enabled displays ideally.
        logical_displays.sort_by_key(|logical| logical.is_enabled);
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

    #[instrument(ret, skip_all, level = "debug")]
    fn get_inner(
        ids: BTreeSet<DisplayIdentifier>,
    ) -> anyhow::Result<BTreeMap<DisplayIdentifier, (DisplayIdentifierInner, Display)>> {
        let mut displays = Self::query()?;
        // Enabled displays first as it's required for setting brightness.
        displays.sort_by_key(|display| display.logical.is_enabled);
        Ok(
            displays
                .into_iter()
                .filter_map(|displ| {
                    let id = displ.id();
                    ids.iter()
                        .filter(|user_id| user_id.is_subset(&id.outer))
                        .nth(0)
                        .and_then(|user_id| {
                            debug!("{id:?}: {displ:?}");
                            Some((user_id.clone(), (id, displ)))
                        })
                })
                .collect(), // .fold(BTreeMap::new(), |mut items_by_user_id, (user_id, item)| {
                            //     items_by_user_id.entry(user_id).or_insert(vec![]).push(item);
                            //     items_by_user_id
                            // })
        )
    }

    pub fn get(
        ids: BTreeSet<DisplayIdentifier>,
    ) -> anyhow::Result<BTreeMap<DisplayIdentifier, Display>> {
        Self::get_inner(ids).map(|display_by_id| {
            display_by_id
                .into_iter()
                .map(|(id, (_, display))| (id, display))
                .collect()
        })
    }

    fn apply(updates: Vec<DisplayUpdate>, validate: bool) -> anyhow::Result<Vec<DisplayUpdate>> {
        let ids: BTreeSet<_> = updates
            .clone()
            .into_iter()
            .map(|update| update.id)
            .collect();
        let mut id_mapping = Self::get_inner(ids)?;
        debug!("id_mapping: {id_mapping:#?}");
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
        debug!("updates_inner: {updates_inner:#?}");

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
