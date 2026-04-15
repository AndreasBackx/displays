use glib::Object;

use crate::{
    backend::types::DisplayMatchData, display::Display, display_identifier::DisplayIdentifier,
};

pub mod ffi;
mod imp;

glib::wrapper! {
    pub struct DisplayMatch(ObjectSubclass<imp::DisplayMatch>);
}

impl DisplayMatch {
    pub fn from_data(value: DisplayMatchData) -> Self {
        Object::builder()
            .property(
                "requested-id",
                DisplayIdentifier::from_data(value.requested_id),
            )
            .property("display", Display::from_data(value.display))
            .build()
    }
}
