use std::cell::RefCell;

use glib::{prelude::*, subclass::prelude::*, Properties};

use crate::{display::Display, display_identifier::DisplayIdentifier};

#[derive(Default, Properties)]
#[properties(wrapper_type = super::DisplayMatch)]
pub struct DisplayMatch {
    #[property(get, set, construct_only)]
    pub requested_id: RefCell<Option<DisplayIdentifier>>,
    #[property(get, set, construct_only)]
    pub matched_id: RefCell<Option<DisplayIdentifier>>,
    #[property(get, set, construct_only)]
    pub display: RefCell<Option<Display>>,
}

#[glib::object_subclass]
impl ObjectSubclass for DisplayMatch {
    const NAME: &'static str = "AstalDisplaysDisplayMatch";
    type Type = super::DisplayMatch;
}

#[glib::derived_properties]
impl ObjectImpl for DisplayMatch {}
