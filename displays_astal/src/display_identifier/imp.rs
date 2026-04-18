use std::cell::RefCell;

use glib::{prelude::*, subclass::prelude::*, Properties};

#[derive(Default, Properties)]
#[properties(wrapper_type = super::DisplayIdentifier)]
pub struct DisplayIdentifier {
    #[property(get, set, construct_only)]
    pub name: RefCell<Option<String>>,
    #[property(get, set, construct_only)]
    pub serial_number: RefCell<Option<String>>,
}

#[glib::object_subclass]
impl ObjectSubclass for DisplayIdentifier {
    const NAME: &'static str = "DisplaysAstalDisplayIdentifier";
    type Type = super::DisplayIdentifier;
}

#[glib::derived_properties]
impl ObjectImpl for DisplayIdentifier {}
