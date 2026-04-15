use glib::{prelude::*, subclass::types::ObjectSubclass, translate::IntoGlib};

pub type AstalDisplaysDisplayUpdate = <super::imp::DisplayUpdate as ObjectSubclass>::Instance;

#[unsafe(no_mangle)]
pub extern "C" fn astal_displays_display_update_get_type() -> glib::ffi::GType {
    super::DisplayUpdate::static_type().into_glib()
}
