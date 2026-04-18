use glib::{prelude::*, subclass::types::ObjectSubclass, translate::IntoGlib};

pub type DisplaysAstalDisplay = <super::imp::Display as ObjectSubclass>::Instance;

#[unsafe(no_mangle)]
pub extern "C" fn displays_astal_display_get_type() -> glib::ffi::GType {
    super::Display::static_type().into_glib()
}
