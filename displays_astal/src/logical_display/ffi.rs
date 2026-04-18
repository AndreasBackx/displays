use glib::{prelude::*, subclass::types::ObjectSubclass, translate::IntoGlib};

pub type DisplaysAstalLogicalDisplay = <super::imp::LogicalDisplay as ObjectSubclass>::Instance;
pub type DisplaysAstalLogicalDisplayUpdateContent =
    <super::imp::update_content::LogicalDisplayUpdateContent as ObjectSubclass>::Instance;

#[unsafe(no_mangle)]
pub extern "C" fn displays_astal_logical_display_get_type() -> glib::ffi::GType {
    super::LogicalDisplay::static_type().into_glib()
}

#[unsafe(no_mangle)]
pub extern "C" fn displays_astal_logical_display_update_content_get_type() -> glib::ffi::GType {
    super::LogicalDisplayUpdateContent::static_type().into_glib()
}
