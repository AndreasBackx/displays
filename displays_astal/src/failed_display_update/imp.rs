use std::cell::RefCell;

use glib::{prelude::*, subclass::prelude::*, Properties};

use crate::display_identifier::DisplayIdentifier;

#[derive(Default, Properties)]
#[properties(wrapper_type = super::FailedDisplayUpdate)]
pub struct FailedDisplayUpdate {
    #[property(get, set, construct_only)]
    pub matched_id: RefCell<Option<DisplayIdentifier>>,
    #[property(get, set, construct_only)]
    pub message: RefCell<String>,
}

#[glib::object_subclass]
impl ObjectSubclass for FailedDisplayUpdate {
    const NAME: &'static str = "AstalDisplaysFailedDisplayUpdate";
    type Type = super::FailedDisplayUpdate;
}

#[glib::derived_properties]
impl ObjectImpl for FailedDisplayUpdate {}
