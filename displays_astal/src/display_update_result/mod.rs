use gio::ListStore;
use glib::Object;

use crate::{
    backend::types::DisplayUpdateResultData, display_identifier::DisplayIdentifier,
    display_update::DisplayUpdate, failed_display_update::FailedDisplayUpdate,
};

pub mod ffi;
mod imp;

glib::wrapper! {
    pub struct DisplayUpdateResult(ObjectSubclass<imp::DisplayUpdateResult>);
}

impl DisplayUpdateResult {
    pub fn from_data(value: DisplayUpdateResultData) -> Self {
        let applied = ListStore::new::<DisplayIdentifier>();
        for item in value.applied {
            applied.append(&DisplayIdentifier::from_data(item));
        }

        let failed = ListStore::new::<FailedDisplayUpdate>();
        for item in value.failed {
            failed.append(&FailedDisplayUpdate::from_data(item));
        }

        Object::builder()
            .property(
                "requested-update",
                DisplayUpdate::from_data(value.requested_update),
            )
            .property("applied", applied)
            .property("failed", failed)
            .build()
    }
}
