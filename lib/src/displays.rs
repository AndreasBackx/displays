use crate::{
    display::{Display, DisplayUpdate},
    logical::windows::{display::LogicalDisplayUpdate, manager::LogicalDisplayManagerWindows},
    physical::windows::{display::PhysicalDisplayUpdate, manager::PhysicalDisplayManagerWindows},
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

    fn apply(self, updates: Vec<DisplayUpdate>, validate: bool) -> anyhow::Result<()> {
        let logical_updates: Vec<LogicalDisplayUpdate> = updates
            .clone()
            .into_iter()
            .filter_map(|display| display.into())
            .collect();
        self.logical_manager.apply(logical_updates, validate)?;
        let physical_updates: Vec<PhysicalDisplayUpdate> = updates
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
