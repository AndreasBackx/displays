use glib::Object;

use crate::{
    backend::types::DisplayData, display_identifier::DisplayIdentifier,
    logical_display::LogicalDisplay, physical_display::PhysicalDisplay,
};

pub mod ffi;
mod imp;

glib::wrapper! {
    pub struct Display(ObjectSubclass<imp::Display>);
}

impl Display {
    pub fn from_data(value: DisplayData) -> Self {
        Object::builder()
            .property("id", DisplayIdentifier::from_data(value.id))
            .property("logical", LogicalDisplay::from_data(value.logical))
            .property("physical", value.physical.map(PhysicalDisplay::from_data))
            .build()
    }
}
