use glib::{prelude::*, subclass::types::ObjectSubclass, translate::IntoGlib};

pub type AstalDisplaysDisplayUpdateResult =
    <super::imp::DisplayUpdateResult as ObjectSubclass>::Instance;

#[unsafe(no_mangle)]
pub extern "C" fn astal_displays_display_update_result_get_type() -> glib::ffi::GType {
    super::DisplayUpdateResult::static_type().into_glib()
}
