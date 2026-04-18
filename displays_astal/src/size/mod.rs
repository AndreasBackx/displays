use glib::{prelude::*, Object};

use crate::backend::types::SizeData;

pub mod ffi;
mod imp;

glib::wrapper! {
    pub struct Size(ObjectSubclass<imp::Size>);
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Object::builder()
            .property("width", width)
            .property("height", height)
            .build()
    }

    pub fn from_data(value: SizeData) -> Self {
        Self::new(value.width, value.height)
    }

    pub fn to_data(&self) -> SizeData {
        SizeData {
            width: self.property::<u32>("width"),
            height: self.property::<u32>("height"),
        }
    }
}
