use std::sync::OnceLock;

use glib::Object;

use crate::{
    backend, display::Display, display_identifier::DisplayIdentifier, display_match::DisplayMatch,
    display_update::DisplayUpdate,
};

pub mod ffi;
mod imp;

glib::wrapper! {
    pub struct Manager(ObjectSubclass<imp::Manager>);
}

impl Manager {
    pub fn get_default() -> Self {
        static INSTANCE: OnceLock<Manager> = OnceLock::new();
        INSTANCE.get_or_init(|| Object::new()).clone()
    }

    pub fn query(&self) -> Result<Vec<Display>, glib::Error> {
        backend::active()
            .query()
            .map(|items| items.into_iter().map(Display::from_data).collect())
    }

    pub fn get(&self, ids: Vec<DisplayIdentifier>) -> Result<Vec<DisplayMatch>, glib::Error> {
        let ids = ids.into_iter().map(|id| id.to_data()).collect();
        backend::active()
            .get(ids)
            .map(|items| items.into_iter().map(DisplayMatch::from_data).collect())
    }

    pub fn apply(
        &self,
        updates: Vec<DisplayUpdate>,
        validate: bool,
    ) -> Result<Vec<DisplayUpdate>, glib::Error> {
        let updates = updates.into_iter().map(|update| update.to_data()).collect();
        backend::active()
            .apply(updates, validate)
            .map(|items| items.into_iter().map(DisplayUpdate::from_data).collect())
    }

    pub fn update(&self, updates: Vec<DisplayUpdate>) -> Result<Vec<DisplayUpdate>, glib::Error> {
        self.apply(updates, false)
    }

    pub fn validate(&self, updates: Vec<DisplayUpdate>) -> Result<Vec<DisplayUpdate>, glib::Error> {
        self.apply(updates, true)
    }
}
