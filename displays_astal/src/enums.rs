use glib::{prelude::*, translate::IntoGlib};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, glib::Enum)]
#[enum_type(name = "AstalDisplaysOrientation")]
#[repr(i32)]
pub enum Orientation {
    #[default]
    Landscape = 0,
    Portrait = 90,
    LandscapeFlipped = 180,
    PortraitFlipped = 270,
}

impl From<displays::types::Orientation> for Orientation {
    fn from(value: displays::types::Orientation) -> Self {
        match value {
            displays::types::Orientation::Landscape => Self::Landscape,
            displays::types::Orientation::Portrait => Self::Portrait,
            displays::types::Orientation::LandscapeFlipped => Self::LandscapeFlipped,
            displays::types::Orientation::PortraitFlipped => Self::PortraitFlipped,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn astal_displays_orientation_get_type() -> glib::ffi::GType {
    Orientation::static_type().into_glib()
}
