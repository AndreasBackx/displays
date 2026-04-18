use glib::{prelude::*, subclass::types::ObjectSubclass, translate::IntoGlib};

pub type DisplaysAstalPoint = <super::imp::Point as ObjectSubclass>::Instance;

#[unsafe(no_mangle)]
pub extern "C" fn displays_astal_point_get_type() -> glib::ffi::GType {
    super::Point::static_type().into_glib()
}
