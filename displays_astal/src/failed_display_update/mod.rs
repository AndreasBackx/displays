use glib::Object;

use crate::{backend::types::FailedDisplayUpdateData, display_identifier::DisplayIdentifier};

pub mod ffi;
mod imp;

glib::wrapper! {
    pub struct FailedDisplayUpdate(ObjectSubclass<imp::FailedDisplayUpdate>);
}

impl FailedDisplayUpdate {
    pub fn from_data(value: FailedDisplayUpdateData) -> Self {
        Object::builder()
            .property("matched-id", DisplayIdentifier::from_data(value.matched_id))
            .property("message", value.message)
            .build()
    }
}
