use glib::{prelude::*, subclass::types::ObjectSubclass, translate::IntoGlib};

pub type DisplaysAstalDisplayUpdate = <super::imp::DisplayUpdate as ObjectSubclass>::Instance;

#[unsafe(no_mangle)]
pub extern "C" fn displays_astal_display_update_get_type() -> glib::ffi::GType {
    super::DisplayUpdate::static_type().into_glib()
}
