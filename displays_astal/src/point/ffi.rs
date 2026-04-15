use glib::{prelude::*, subclass::types::ObjectSubclass, translate::IntoGlib};

pub type AstalDisplaysPoint = <super::imp::Point as ObjectSubclass>::Instance;

#[unsafe(no_mangle)]
pub extern "C" fn astal_displays_point_get_type() -> glib::ffi::GType {
    super::Point::static_type().into_glib()
}
