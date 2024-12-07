use crate::{
    display::Display,
    logical_display::LogicalDisplayManagerWindows,
    physical_display::{self, PhysicalDisplayManagerWindows},
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

    pub fn validate(displays: Vec<Displays>) -> Result<(), ValidateUpdateError> {
        Ok(())
    }

    pub fn update(displays: Vec<Displays>) -> Result<(), ValidateUpdateError> {
        Ok(())
    }
}

pub struct QueryError {}
pub struct ValidateUpdateError {}

pub struct CreationError {}
