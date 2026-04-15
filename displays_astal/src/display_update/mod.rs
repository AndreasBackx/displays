use glib::{prelude::*, Object};

use crate::{
    backend::types::DisplayUpdateData, display_identifier::DisplayIdentifier,
    logical_display::LogicalDisplayUpdateContent, physical_display::PhysicalDisplayUpdateContent,
};

pub mod ffi;
mod imp;

glib::wrapper! {
    pub struct DisplayUpdate(ObjectSubclass<imp::DisplayUpdate>);
}

impl DisplayUpdate {
    pub fn from_data(value: DisplayUpdateData) -> Self {
        Object::builder()
            .property("id", DisplayIdentifier::from_data(value.id))
            .property(
                "logical",
                value.logical.map(LogicalDisplayUpdateContent::from_data),
            )
            .property(
                "physical",
                value.physical.map(PhysicalDisplayUpdateContent::from_data),
            )
            .build()
    }

    pub fn to_data(&self) -> DisplayUpdateData {
        DisplayUpdateData {
            id: self.property::<DisplayIdentifier>("id").to_data(),
            logical: self
                .property::<Option<LogicalDisplayUpdateContent>>("logical")
                .map(|logical: LogicalDisplayUpdateContent| logical.to_data()),
            physical: self
                .property::<Option<PhysicalDisplayUpdateContent>>("physical")
                .map(|physical: PhysicalDisplayUpdateContent| physical.to_data()),
        }
    }
}
