use std::cell::RefCell;

use glib::{prelude::*, subclass::prelude::*, Properties};

use crate::{
    display_identifier::DisplayIdentifier, logical_display::LogicalDisplayUpdateContent,
    physical_display::PhysicalDisplayUpdateContent,
};

#[derive(Default, Properties)]
#[properties(wrapper_type = super::DisplayUpdate)]
pub struct DisplayUpdate {
    #[property(get, set, construct_only)]
    pub id: RefCell<Option<DisplayIdentifier>>,
    #[property(get, set, construct_only)]
    pub logical: RefCell<Option<LogicalDisplayUpdateContent>>,
    #[property(get, set, construct_only)]
    pub physical: RefCell<Option<PhysicalDisplayUpdateContent>>,
}

#[glib::object_subclass]
impl ObjectSubclass for DisplayUpdate {
    const NAME: &'static str = "DisplaysAstalDisplayUpdate";
    type Type = super::DisplayUpdate;
}

#[glib::derived_properties]
impl ObjectImpl for DisplayUpdate {}
