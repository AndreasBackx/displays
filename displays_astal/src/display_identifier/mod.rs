use glib::{prelude::*, Object};

use crate::backend::types::DisplayIdentifierData;

pub mod ffi;
mod imp;

glib::wrapper! {
    pub struct DisplayIdentifier(ObjectSubclass<imp::DisplayIdentifier>);
}

impl DisplayIdentifier {
    pub fn new(name: Option<&str>, serial_number: Option<&str>) -> Self {
        Object::builder()
            .property("name", name.map(str::to_string))
            .property("serial-number", serial_number.map(str::to_string))
            .build()
    }

    pub fn from_data(value: DisplayIdentifierData) -> Self {
        Self::new(value.name.as_deref(), value.serial_number.as_deref())
    }

    pub fn to_data(&self) -> DisplayIdentifierData {
        DisplayIdentifierData {
            name: self.property::<Option<String>>("name"),
            serial_number: self.property::<Option<String>>("serial-number"),
        }
    }
}
