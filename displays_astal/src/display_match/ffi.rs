use glib::{prelude::*, subclass::types::ObjectSubclass, translate::IntoGlib};

pub type AstalDisplaysDisplayMatch = <super::imp::DisplayMatch as ObjectSubclass>::Instance;

#[unsafe(no_mangle)]
pub extern "C" fn astal_displays_display_match_get_type() -> glib::ffi::GType {
    super::DisplayMatch::static_type().into_glib()
}
