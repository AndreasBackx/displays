use glib::{prelude::*, subclass::types::ObjectSubclass, translate::IntoGlib};

pub type AstalDisplaysDisplay = <super::imp::Display as ObjectSubclass>::Instance;

#[unsafe(no_mangle)]
pub extern "C" fn astal_displays_display_get_type() -> glib::ffi::GType {
    super::Display::static_type().into_glib()
}
