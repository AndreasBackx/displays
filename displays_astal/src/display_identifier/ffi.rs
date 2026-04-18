use glib::{prelude::*, subclass::types::ObjectSubclass, translate::IntoGlib};

pub type DisplaysAstalDisplayIdentifier =
    <super::imp::DisplayIdentifier as ObjectSubclass>::Instance;

#[unsafe(no_mangle)]
pub extern "C" fn displays_astal_display_identifier_get_type() -> glib::ffi::GType {
    super::DisplayIdentifier::static_type().into_glib()
}
