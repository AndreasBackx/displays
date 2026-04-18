use std::cell::Cell;

use glib::{prelude::*, subclass::prelude::*, Properties};

#[derive(Default, Properties)]
#[properties(wrapper_type = super::Point)]
pub struct Point {
    #[property(get, set, construct_only)]
    pub x: Cell<i32>,
    #[property(get, set, construct_only)]
    pub y: Cell<i32>,
}

#[glib::object_subclass]
impl ObjectSubclass for Point {
    const NAME: &'static str = "DisplaysAstalPoint";
    type Type = super::Point;
}

#[glib::derived_properties]
impl ObjectImpl for Point {}
