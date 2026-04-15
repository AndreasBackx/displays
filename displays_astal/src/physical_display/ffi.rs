use glib::{prelude::*, subclass::types::ObjectSubclass, translate::IntoGlib};

pub type AstalDisplaysPhysicalDisplay = <super::imp::PhysicalDisplay as ObjectSubclass>::Instance;
pub type AstalDisplaysPhysicalDisplayUpdateContent =
    <super::imp::update_content::PhysicalDisplayUpdateContent as ObjectSubclass>::Instance;

#[unsafe(no_mangle)]
pub extern "C" fn astal_displays_physical_display_get_type() -> glib::ffi::GType {
    super::PhysicalDisplay::static_type().into_glib()
}

#[unsafe(no_mangle)]
pub extern "C" fn astal_displays_physical_display_update_content_get_type() -> glib::ffi::GType {
    super::PhysicalDisplayUpdateContent::static_type().into_glib()
}
