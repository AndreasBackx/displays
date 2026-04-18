use glib::{prelude::*, Object};

use crate::{
    backend::types::{LogicalDisplayData, LogicalDisplayUpdateContentData},
    enums::Orientation,
    point::Point,
    size::Size,
};

pub mod ffi;
mod imp;

glib::wrapper! {
    pub struct LogicalDisplay(ObjectSubclass<imp::LogicalDisplay>);
}

glib::wrapper! {
    pub struct LogicalDisplayUpdateContent(ObjectSubclass<imp::update_content::LogicalDisplayUpdateContent>);
}

impl LogicalDisplay {
    pub fn from_data(value: LogicalDisplayData) -> Self {
        Object::builder()
            .property("is-enabled", value.is_enabled)
            .property("orientation", value.orientation)
            .property("logical-size", value.logical_size.map(Size::from_data))
            .property("mode-size", value.mode_size.map(Size::from_data))
            .property(
                "scale-ratio-milli",
                value.scale_ratio_milli.unwrap_or_default(),
            )
            .property("has-scale-ratio-milli", value.scale_ratio_milli.is_some())
            .property("position", value.position.map(Point::from_data))
            .build()
    }
}

impl LogicalDisplayUpdateContent {
    pub fn from_data(value: LogicalDisplayUpdateContentData) -> Self {
        Object::builder()
            .property("has-is-enabled", value.is_enabled.is_some())
            .property("is-enabled", value.is_enabled.unwrap_or(false))
            .property("has-orientation", value.orientation.is_some())
            .property("orientation", value.orientation.unwrap_or_default())
            .property("has-width", value.width.is_some())
            .property("width", value.width.unwrap_or_default())
            .property("has-height", value.height.is_some())
            .property("height", value.height.unwrap_or_default())
            .property("position", value.position.map(Point::from_data))
            .build()
    }

    pub fn to_data(&self) -> LogicalDisplayUpdateContentData {
        LogicalDisplayUpdateContentData {
            is_enabled: self
                .property::<bool>("has-is-enabled")
                .then(|| self.property::<bool>("is-enabled")),
            orientation: self
                .property::<bool>("has-orientation")
                .then(|| self.property::<Orientation>("orientation")),
            width: self
                .property::<bool>("has-width")
                .then(|| self.property::<u32>("width")),
            height: self
                .property::<bool>("has-height")
                .then(|| self.property::<u32>("height")),
            position: self
                .property::<Option<Point>>("position")
                .map(|point| point.to_data()),
        }
    }
}
