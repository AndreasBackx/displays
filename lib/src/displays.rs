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
        let physical_displays = self.physical_manager.query()?;
        Ok(vec![])
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
