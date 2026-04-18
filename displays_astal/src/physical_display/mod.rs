use glib::{prelude::*, Object};

use crate::backend::types::{PhysicalDisplayData, PhysicalDisplayUpdateContentData};

pub mod ffi;
mod imp;

glib::wrapper! {
    pub struct PhysicalDisplay(ObjectSubclass<imp::PhysicalDisplay>);
}

glib::wrapper! {
    pub struct PhysicalDisplayUpdateContent(ObjectSubclass<imp::update_content::PhysicalDisplayUpdateContent>);
}

impl PhysicalDisplay {
    pub fn from_data(value: PhysicalDisplayData) -> Self {
        Object::builder()
            .property("has-brightness", value.brightness.is_some())
            .property("brightness", value.brightness.unwrap_or_default())
            .property("scale-factor", value.scale_factor)
            .build()
    }
}

impl PhysicalDisplayUpdateContent {
    pub fn from_data(value: PhysicalDisplayUpdateContentData) -> Self {
        Object::builder()
            .property("has-brightness", value.brightness.is_some())
            .property("brightness", value.brightness.unwrap_or_default())
            .build()
    }

    pub fn to_data(&self) -> PhysicalDisplayUpdateContentData {
        PhysicalDisplayUpdateContentData {
            brightness: self
                .property::<bool>("has-brightness")
                .then(|| self.property::<u32>("brightness")),
        }
    }
}
