use std::cell::RefCell;

use glib::{prelude::*, subclass::prelude::*, Properties};

use crate::{
    display_identifier::DisplayIdentifier, logical_display::LogicalDisplay,
    physical_display::PhysicalDisplay,
};

#[derive(Default, Properties)]
#[properties(wrapper_type = super::Display)]
pub struct Display {
    #[property(get, set, construct_only)]
    pub id: RefCell<Option<DisplayIdentifier>>,
    #[property(get, set, construct_only)]
    pub logical: RefCell<Option<LogicalDisplay>>,
    #[property(get, set, construct_only)]
    pub physical: RefCell<Option<PhysicalDisplay>>,
}

#[glib::object_subclass]
impl ObjectSubclass for Display {
    const NAME: &'static str = "AstalDisplaysDisplay";
    type Type = super::Display;
}

#[glib::derived_properties]
impl ObjectImpl for Display {}
