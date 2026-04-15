use glib::{prelude::*, Object};

use crate::backend::types::PointData;

pub mod ffi;
mod imp;

glib::wrapper! {
    pub struct Point(ObjectSubclass<imp::Point>);
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Object::builder().property("x", x).property("y", y).build()
    }

    pub fn from_data(value: PointData) -> Self {
        Self::new(value.x, value.y)
    }

    pub fn to_data(&self) -> PointData {
        PointData {
            x: self.property::<i32>("x"),
            y: self.property::<i32>("y"),
        }
    }
}
