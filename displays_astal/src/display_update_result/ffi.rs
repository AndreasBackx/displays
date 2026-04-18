use glib::{prelude::*, subclass::types::ObjectSubclass, translate::IntoGlib};

pub type DisplaysAstalDisplayUpdateResult =
    <super::imp::DisplayUpdateResult as ObjectSubclass>::Instance;

#[unsafe(no_mangle)]
pub extern "C" fn displays_astal_display_update_result_get_type() -> glib::ffi::GType {
    super::DisplayUpdateResult::static_type().into_glib()
}
