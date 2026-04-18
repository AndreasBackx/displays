use std::{
    cell::{Cell, RefCell},
    sync::OnceLock,
};

use glib::{prelude::*, subclass::prelude::*};

use crate::{enums::Orientation, point::Point, size::Size};

#[derive(Default)]
pub struct LogicalDisplay {
    pub is_enabled: Cell<bool>,
    pub orientation: Cell<Orientation>,
    pub logical_size: RefCell<Option<Size>>,
    pub mode_size: RefCell<Option<Size>>,
    pub has_scale_ratio_milli: Cell<bool>,
    pub scale_ratio_milli: Cell<u32>,
    pub position: RefCell<Option<Point>>,
}

#[glib::object_subclass]
impl ObjectSubclass for LogicalDisplay {
    const NAME: &'static str = "DisplaysAstalLogicalDisplay";
    type Type = super::LogicalDisplay;
}

impl ObjectImpl for LogicalDisplay {
    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
        PROPERTIES.get_or_init(|| {
            let flags = glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY;
            vec![
                glib::ParamSpecBoolean::builder("is-enabled")
                    .flags(flags)
                    .build(),
                glib::ParamSpecEnum::builder::<Orientation>("orientation")
                    .flags(flags)
                    .build(),
                glib::ParamSpecObject::builder::<Size>("logical-size")
                    .flags(flags)
                    .build(),
                glib::ParamSpecObject::builder::<Size>("mode-size")
                    .flags(flags)
                    .build(),
                glib::ParamSpecBoolean::builder("has-scale-ratio-milli")
                    .flags(flags)
                    .build(),
                glib::ParamSpecUInt::builder("scale-ratio-milli")
                    .flags(flags)
                    .build(),
                glib::ParamSpecObject::builder::<Point>("position")
                    .flags(flags)
                    .build(),
            ]
        })
    }

    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        match pspec.name() {
            "is-enabled" => self.is_enabled.set(value.get().unwrap()),
            "orientation" => self.orientation.set(value.get().unwrap()),
            "logical-size" => {
                self.logical_size.replace(value.get().unwrap());
            }
            "mode-size" => {
                self.mode_size.replace(value.get().unwrap());
            }
            "has-scale-ratio-milli" => self.has_scale_ratio_milli.set(value.get().unwrap()),
            "scale-ratio-milli" => self.scale_ratio_milli.set(value.get().unwrap()),
            "position" => {
                self.position.replace(value.get().unwrap());
            }
            name => unreachable!("unknown property {name}"),
        }
    }

    fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "is-enabled" => self.is_enabled.get().to_value(),
            "orientation" => self.orientation.get().to_value(),
            "logical-size" => self.logical_size.borrow().to_value(),
            "mode-size" => self.mode_size.borrow().to_value(),
            "has-scale-ratio-milli" => self.has_scale_ratio_milli.get().to_value(),
            "scale-ratio-milli" => self.scale_ratio_milli.get().to_value(),
            "position" => self.position.borrow().to_value(),
            name => unreachable!("unknown property {name}"),
        }
    }
}

pub mod update_content {
    use std::{
        cell::{Cell, RefCell},
        sync::OnceLock,
    };

    use glib::{prelude::*, subclass::prelude::*};

    use crate::{enums::Orientation, point::Point};

    #[derive(Default)]
    pub struct LogicalDisplayUpdateContent {
        pub has_is_enabled: Cell<bool>,
        pub is_enabled: Cell<bool>,
        pub has_orientation: Cell<bool>,
        pub orientation: Cell<Orientation>,
        pub has_width: Cell<bool>,
        pub width: Cell<u32>,
        pub has_height: Cell<bool>,
        pub height: Cell<u32>,
        pub position: RefCell<Option<Point>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LogicalDisplayUpdateContent {
        const NAME: &'static str = "DisplaysAstalLogicalDisplayUpdateContent";
        type Type = super::super::LogicalDisplayUpdateContent;
    }

    impl ObjectImpl for LogicalDisplayUpdateContent {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                let flags = glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY;
                vec![
                    glib::ParamSpecBoolean::builder("has-is-enabled")
                        .flags(flags)
                        .build(),
                    glib::ParamSpecBoolean::builder("is-enabled")
                        .flags(flags)
                        .build(),
                    glib::ParamSpecBoolean::builder("has-orientation")
                        .flags(flags)
                        .build(),
                    glib::ParamSpecEnum::builder::<Orientation>("orientation")
                        .flags(flags)
                        .build(),
                    glib::ParamSpecBoolean::builder("has-width")
                        .flags(flags)
                        .build(),
                    glib::ParamSpecUInt::builder("width").flags(flags).build(),
                    glib::ParamSpecBoolean::builder("has-height")
                        .flags(flags)
                        .build(),
                    glib::ParamSpecUInt::builder("height").flags(flags).build(),
                    glib::ParamSpecObject::builder::<Point>("position")
                        .flags(flags)
                        .build(),
                ]
            })
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "has-is-enabled" => self.has_is_enabled.set(value.get().unwrap()),
                "is-enabled" => self.is_enabled.set(value.get().unwrap()),
                "has-orientation" => self.has_orientation.set(value.get().unwrap()),
                "orientation" => self.orientation.set(value.get().unwrap()),
                "has-width" => self.has_width.set(value.get().unwrap()),
                "width" => self.width.set(value.get().unwrap()),
                "has-height" => self.has_height.set(value.get().unwrap()),
                "height" => self.height.set(value.get().unwrap()),
                "position" => {
                    self.position.replace(value.get().unwrap());
                }
                name => unreachable!("unknown property {name}"),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "has-is-enabled" => self.has_is_enabled.get().to_value(),
                "is-enabled" => self.is_enabled.get().to_value(),
                "has-orientation" => self.has_orientation.get().to_value(),
                "orientation" => self.orientation.get().to_value(),
                "has-width" => self.has_width.get().to_value(),
                "width" => self.width.get().to_value(),
                "has-height" => self.has_height.get().to_value(),
                "height" => self.height.get().to_value(),
                "position" => self.position.borrow().to_value(),
                name => unreachable!("unknown property {name}"),
            }
        }
    }
}
