use std::sync::OnceLock;

use gio::{AsyncResult, Cancellable};
use glib::Object;

use crate::{
    backend,
    backend::types::{DisplayData, DisplayMatchData, DisplayUpdateData, DisplayUpdateResultData},
    display::Display,
    display_identifier::DisplayIdentifier,
    display_match::DisplayMatch,
    display_update::DisplayUpdate,
    display_update_result::DisplayUpdateResult,
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

    pub(crate) fn query_in_thread() -> Result<Vec<DisplayData>, glib::Error> {
        backend::active().query()
    }

    pub(crate) fn get_in_thread(
        ids: Vec<crate::backend::types::DisplayIdentifierData>,
    ) -> Result<Vec<DisplayMatchData>, glib::Error> {
        backend::active().get(ids)
    }

    pub(crate) fn apply_in_thread(
        updates: Vec<DisplayUpdateData>,
        validate: bool,
    ) -> Result<Vec<DisplayUpdateResultData>, glib::Error> {
        backend::active().apply(updates, validate)
    }

    pub fn query_finish(
        &self,
        result: &impl glib::object::IsA<AsyncResult>,
    ) -> Result<Vec<Display>, glib::Error> {
        let payload = crate::manager::ffi::take_query_result(self, result)?;
        Ok(payload.into_iter().map(Display::from_data).collect())
    }

    pub fn get_finish(
        &self,
        result: &impl glib::object::IsA<AsyncResult>,
    ) -> Result<Vec<DisplayMatch>, glib::Error> {
        let payload = crate::manager::ffi::take_get_result(self, result)?;
        Ok(payload.into_iter().map(DisplayMatch::from_data).collect())
    }

    pub fn apply_finish(
        &self,
        result: &impl glib::object::IsA<AsyncResult>,
    ) -> Result<Vec<DisplayUpdateResult>, glib::Error> {
        let payload = crate::manager::ffi::take_update_result(self, result)?;
        Ok(payload
            .into_iter()
            .map(DisplayUpdateResult::from_data)
            .collect())
    }

    pub fn update_finish(
        &self,
        result: &impl glib::object::IsA<AsyncResult>,
    ) -> Result<Vec<DisplayUpdateResult>, glib::Error> {
        self.apply_finish(result)
    }

    pub fn validate_finish(
        &self,
        result: &impl glib::object::IsA<AsyncResult>,
    ) -> Result<Vec<DisplayUpdateResult>, glib::Error> {
        self.apply_finish(result)
    }

    pub fn query_async<P: glib::object::IsA<Cancellable>, F>(
        &self,
        cancellable: Option<&P>,
        callback: F,
    ) where
        F: FnOnce(&Self, &gio::AsyncResult) + 'static,
    {
        crate::manager::ffi::spawn_query_task(self, cancellable, callback);
    }

    pub fn get_async<P: glib::object::IsA<Cancellable>, F>(
        &self,
        ids: Vec<DisplayIdentifier>,
        cancellable: Option<&P>,
        callback: F,
    ) where
        F: FnOnce(&Self, &gio::AsyncResult) + 'static,
    {
        let ids = ids.into_iter().map(|id| id.to_data()).collect();
        crate::manager::ffi::spawn_get_task(self, ids, cancellable, callback);
    }

    pub fn apply_async<P: glib::object::IsA<Cancellable>, F>(
        &self,
        updates: Vec<DisplayUpdate>,
        validate: bool,
        cancellable: Option<&P>,
        callback: F,
    ) where
        F: FnOnce(&Self, &gio::AsyncResult) + 'static,
    {
        let updates = updates.into_iter().map(|update| update.to_data()).collect();
        crate::manager::ffi::spawn_apply_task(self, updates, validate, cancellable, callback);
    }

    pub fn update_async<P: glib::object::IsA<Cancellable>, F>(
        &self,
        updates: Vec<DisplayUpdate>,
        cancellable: Option<&P>,
        callback: F,
    ) where
        F: FnOnce(&Self, &gio::AsyncResult) + 'static,
    {
        self.apply_async(updates, false, cancellable, callback)
    }

    pub fn validate_async<P: glib::object::IsA<Cancellable>, F>(
        &self,
        updates: Vec<DisplayUpdate>,
        cancellable: Option<&P>,
        callback: F,
    ) where
        F: FnOnce(&Self, &gio::AsyncResult) + 'static,
    {
        self.apply_async(updates, true, cancellable, callback)
    }
}
