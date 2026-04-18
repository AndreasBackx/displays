use std::cell::Cell;

use glib::{prelude::*, subclass::prelude::*, Properties};

#[derive(Default, Properties)]
#[properties(wrapper_type = super::Size)]
pub struct Size {
    #[property(get, set, construct_only)]
    pub width: Cell<u32>,
    #[property(get, set, construct_only)]
    pub height: Cell<u32>,
}

#[glib::object_subclass]
impl ObjectSubclass for Size {
    const NAME: &'static str = "DisplaysAstalSize";
    type Type = super::Size;
}

#[glib::derived_properties]
impl ObjectImpl for Size {}
