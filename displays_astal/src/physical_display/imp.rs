use std::cell::Cell;

use glib::{prelude::*, subclass::prelude::*, Properties};

#[derive(Default, Properties)]
#[properties(wrapper_type = super::PhysicalDisplay)]
pub struct PhysicalDisplay {
    #[property(get, set, construct_only)]
    pub has_brightness: Cell<bool>,
    #[property(get, set, construct_only)]
    pub brightness: Cell<u32>,
}

#[glib::object_subclass]
impl ObjectSubclass for PhysicalDisplay {
    const NAME: &'static str = "DisplaysAstalPhysicalDisplay";
    type Type = super::PhysicalDisplay;
}

#[glib::derived_properties]
impl ObjectImpl for PhysicalDisplay {}

pub mod update_content {
    use std::cell::Cell;

    use glib::{prelude::*, subclass::prelude::*, Properties};

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::super::PhysicalDisplayUpdateContent)]
    pub struct PhysicalDisplayUpdateContent {
        #[property(get, set, construct_only)]
        pub has_brightness: Cell<bool>,
        #[property(get, set, construct_only)]
        pub brightness: Cell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PhysicalDisplayUpdateContent {
        const NAME: &'static str = "DisplaysAstalPhysicalDisplayUpdateContent";
        type Type = super::super::PhysicalDisplayUpdateContent;
    }

    #[glib::derived_properties]
    impl ObjectImpl for PhysicalDisplayUpdateContent {}
}
