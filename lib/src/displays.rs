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

pub struct Displays {
    logical_manager: LogicalDisplayManagerWindows,
    physical_manager: PhysicalDisplayManagerWindows,
}

impl Displays {
    pub fn try_new() -> anyhow::Result<Self> {
        Ok(Self {
            logical_manager: LogicalDisplayManagerWindows::try_new()?,
            physical_manager: PhysicalDisplayManagerWindows::try_new()?,
        })
    }

    pub fn query(&self) -> anyhow::Result<Vec<Display>> {
        let logical_displays = self.logical_manager.query()?;
        let mut physical_displays = self.physical_manager.query()?;

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
        &self,
        ids: BTreeSet<&DisplayIdentifier>,
    ) -> anyhow::Result<BTreeMap<DisplayIdentifier, (DisplayIdentifierInner, Display)>> {
        let displays = self.query()?;
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
        &self,
        ids: BTreeSet<&DisplayIdentifier>,
    ) -> anyhow::Result<BTreeMap<DisplayIdentifier, Display>> {
        self.get_inner(ids).map(|display_by_id| {
            display_by_id
                .into_iter()
                .map(|(id, (_, display))| (id, display))
                .collect()
        })
    }

    // fn resolve_id(&self, id: DisplayIdentifier) -> anyhow::Result<DisplayIdentifier> {}

    fn apply(self, updates: Vec<DisplayUpdate>, validate: bool) -> anyhow::Result<()> {
        let ids = updates.iter().map(|update| &update.id).collect();
        let mut id_mapping = self.get_inner(ids)?;
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
        self.logical_manager.apply(logical_updates, validate)?;
        let physical_updates: Vec<PhysicalDisplayUpdate> = updates_inner
            .into_iter()
            .filter_map(|display| display.into())
            .collect();
        self.physical_manager.apply(physical_updates)?;
        Ok(())
    }

    pub fn update(self, updates: Vec<DisplayUpdate>) -> anyhow::Result<()> {
        self.apply(updates, false)
    }

    pub fn validate(self, updates: Vec<DisplayUpdate>) -> anyhow::Result<()> {
        self.apply(updates, true)
    }
}

pub struct QueryError {}
pub struct ValidateUpdateError {}

pub struct CreationError {}
