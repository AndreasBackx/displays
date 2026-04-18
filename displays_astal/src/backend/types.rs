use crate::enums::Orientation;

/// Backend-neutral identifier shape used by the GI layer.

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct DisplayIdentifierData {
    pub name: Option<String>,
    pub serial_number: Option<String>,
}

impl DisplayIdentifierData {
    pub fn is_subset_of(&self, other: &Self) -> bool {
        let left: displays::types::DisplayIdentifier = self.clone().into();
        let right: displays::types::DisplayIdentifier = other.clone().into();
        left.is_subset(&right)
    }
}

impl From<DisplayIdentifierData> for displays::types::DisplayIdentifier {
    fn from(value: DisplayIdentifierData) -> Self {
        Self {
            name: value.name,
            serial_number: value.serial_number,
        }
    }
}

impl From<displays::types::DisplayIdentifier> for DisplayIdentifierData {
    fn from(value: displays::types::DisplayIdentifier) -> Self {
        Self {
            name: value.name,
            serial_number: value.serial_number,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PointData {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SizeData {
    pub width: u32,
    pub height: u32,
}

impl From<PointData> for displays::types::Point {
    fn from(value: PointData) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<displays::types::Point> for PointData {
    fn from(value: displays::types::Point) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<SizeData> for displays::types::Size {
    fn from(value: SizeData) -> Self {
        Self {
            width: value.width,
            height: value.height,
        }
    }
}

impl From<displays::types::Size> for SizeData {
    fn from(value: displays::types::Size) -> Self {
        Self {
            width: value.width,
            height: value.height,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LogicalDisplayData {
    pub is_enabled: bool,
    pub orientation: Orientation,
    pub logical_size: Option<SizeData>,
    pub mode_size: Option<SizeData>,
    pub scale_ratio_milli: Option<u32>,
    pub position: Option<PointData>,
}

impl From<displays::types::LogicalDisplay> for LogicalDisplayData {
    fn from(value: displays::types::LogicalDisplay) -> Self {
        Self {
            is_enabled: value.state.is_enabled,
            orientation: value.state.orientation.into(),
            logical_size: value.state.logical_size.map(Into::into),
            mode_size: value.state.mode_size.map(Into::into),
            scale_ratio_milli: value.state.scale_ratio_milli,
            position: value.state.logical_position.map(Into::into),
        }
    }
}

impl From<&displays::types::LogicalDisplay> for LogicalDisplayData {
    fn from(value: &displays::types::LogicalDisplay) -> Self {
        Self {
            is_enabled: value.state.is_enabled,
            orientation: value.state.orientation.clone().into(),
            logical_size: value.state.logical_size.clone().map(Into::into),
            mode_size: value.state.mode_size.clone().map(Into::into),
            scale_ratio_milli: value.state.scale_ratio_milli,
            position: value.state.logical_position.clone().map(Into::into),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalDisplayData {
    pub brightness: Option<u32>,
}

impl From<displays::types::PhysicalDisplay> for PhysicalDisplayData {
    fn from(value: displays::types::PhysicalDisplay) -> Self {
        Self {
            brightness: value
                .state
                .brightness
                .map(|brightness| brightness.value() as u32),
        }
    }
}

impl From<&displays::types::PhysicalDisplay> for PhysicalDisplayData {
    fn from(value: &displays::types::PhysicalDisplay) -> Self {
        Self {
            brightness: value
                .state
                .brightness
                .as_ref()
                .map(|brightness| brightness.value() as u32),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DisplayData {
    pub id: DisplayIdentifierData,
    pub logical: LogicalDisplayData,
    pub physical: Option<PhysicalDisplayData>,
}

impl From<displays::display::Display> for DisplayData {
    fn from(value: displays::display::Display) -> Self {
        let id = value.id().outer.into();
        let logical = value.logical.into();
        let physical = value.physical.map(Into::into);

        Self {
            id,
            logical,
            physical,
        }
    }
}

impl From<&displays::display::Display> for DisplayData {
    fn from(value: &displays::display::Display) -> Self {
        let id = value.id().outer.into();
        let logical = (&value.logical).into();
        let physical = value.physical.as_ref().map(Into::into);

        Self {
            id,
            logical,
            physical,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DisplayMatchData {
    pub requested_id: DisplayIdentifierData,
    pub matched_id: DisplayIdentifierData,
    pub display: DisplayData,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FailedDisplayUpdateData {
    pub matched_id: DisplayIdentifierData,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct LogicalDisplayUpdateContentData {
    pub is_enabled: Option<bool>,
    pub orientation: Option<Orientation>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub position: Option<PointData>,
}

impl From<LogicalDisplayUpdateContentData>
    for displays::types::LogicalDisplayUpdateContent
{
    fn from(value: LogicalDisplayUpdateContentData) -> Self {
        Self {
            is_enabled: value.is_enabled,
            orientation: value.orientation.map(Into::into),
            width: value.width,
            height: value.height,
            pixel_format: None,
            position: value.position.map(Into::into),
        }
    }
}

impl From<displays::types::LogicalDisplayUpdateContent>
    for LogicalDisplayUpdateContentData
{
    fn from(value: displays::types::LogicalDisplayUpdateContent) -> Self {
        Self {
            is_enabled: value.is_enabled,
            orientation: value.orientation.map(Into::into),
            width: value.width,
            height: value.height,
            position: value.position.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalDisplayUpdateContentData {
    pub brightness: Option<u32>,
}

impl From<PhysicalDisplayUpdateContentData>
    for displays::types::PhysicalDisplayUpdateContent
{
    fn from(value: PhysicalDisplayUpdateContentData) -> Self {
        Self {
            brightness: value.brightness,
        }
    }
}

impl From<displays::types::PhysicalDisplayUpdateContent>
    for PhysicalDisplayUpdateContentData
{
    fn from(value: displays::types::PhysicalDisplayUpdateContent) -> Self {
        Self {
            brightness: value.brightness,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct DisplayUpdateData {
    pub id: DisplayIdentifierData,
    pub logical: Option<LogicalDisplayUpdateContentData>,
    pub physical: Option<PhysicalDisplayUpdateContentData>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DisplayUpdateResultData {
    pub requested_update: DisplayUpdateData,
    pub applied: Vec<DisplayIdentifierData>,
    pub failed: Vec<FailedDisplayUpdateData>,
}

impl From<DisplayUpdateData> for displays::display::DisplayUpdate {
    fn from(value: DisplayUpdateData) -> Self {
        Self {
            id: value.id.into(),
            logical: value.logical.map(Into::into),
            physical: value.physical.map(Into::into),
        }
    }
}

impl From<displays::display::DisplayUpdate> for DisplayUpdateData {
    fn from(value: displays::display::DisplayUpdate) -> Self {
        Self {
            id: value.id.into(),
            logical: value.logical.map(Into::into),
            physical: value.physical.map(Into::into),
        }
    }
}

impl From<crate::enums::Orientation> for displays::types::Orientation {
    fn from(value: crate::enums::Orientation) -> Self {
        match value {
            crate::enums::Orientation::Landscape => Self::Landscape,
            crate::enums::Orientation::Portrait => Self::Portrait,
            crate::enums::Orientation::LandscapeFlipped => Self::LandscapeFlipped,
            crate::enums::Orientation::PortraitFlipped => Self::PortraitFlipped,
        }
    }
}
