use glib::{prelude::*, subclass::types::ObjectSubclass, translate::IntoGlib};

pub type AstalDisplaysFailedDisplayUpdate =
    <super::imp::FailedDisplayUpdate as ObjectSubclass>::Instance;

#[unsafe(no_mangle)]
pub extern "C" fn astal_displays_failed_display_update_get_type() -> glib::ffi::GType {
    super::FailedDisplayUpdate::static_type().into_glib()
}
