use std::cell::RefCell;

use gio::ListStore;
use glib::{prelude::*, subclass::prelude::*, Properties};

use crate::display_update::DisplayUpdate;

#[derive(Properties)]
#[properties(wrapper_type = super::DisplayUpdateResult)]
pub struct DisplayUpdateResult {
    #[property(get, set, construct_only)]
    pub requested_update: RefCell<Option<DisplayUpdate>>,
    #[property(get, set, construct_only)]
    pub applied: RefCell<ListStore>,
    #[property(get, set, construct_only)]
    pub failed: RefCell<ListStore>,
}

impl Default for DisplayUpdateResult {
    fn default() -> Self {
        Self {
            requested_update: RefCell::default(),
            applied: RefCell::new(ListStore::new::<glib::Object>()),
            failed: RefCell::new(ListStore::new::<glib::Object>()),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for DisplayUpdateResult {
    const NAME: &'static str = "AstalDisplaysDisplayUpdateResult";
    type Type = super::DisplayUpdateResult;
}

#[glib::derived_properties]
impl ObjectImpl for DisplayUpdateResult {}
