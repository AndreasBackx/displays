use std::path::PathBuf;

/// A Linux brightness-capable device and its current state.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Device {
    /// Stable metadata identifying the device.
    pub metadata: DeviceMetadata,
    /// The current brightness state reported by sysfs.
    pub state: DeviceState,
}

/// Stable metadata describing a Linux brightness device.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeviceMetadata {
    /// The sysfs class the device was enumerated from.
    pub class: DeviceClass,
    /// The sysfs directory name of the device.
    pub id: String,
    /// The full sysfs path to the device directory.
    pub path: String,
}

/// The current state of a Linux brightness device.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeviceState {
    /// The raw value currently stored in `brightness`.
    pub brightness_raw: u32,
    /// The raw maximum value stored in `max_brightness`.
    pub max_brightness_raw: u32,
    /// The raw value stored in `actual_brightness` when the file exists.
    pub actual_brightness_raw: Option<u32>,
    /// The current brightness normalized to the inclusive range `0..=100`.
    pub brightness_percent: u8,
}

/// Supported Linux sysfs device classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DeviceClass {
    /// A device found under `/sys/class/backlight`.
    Backlight,
    /// A device found under `/sys/class/leds`.
    Leds,
}

impl DeviceClass {
    pub(crate) const fn directory_name(self) -> &'static str {
        match self {
            DeviceClass::Backlight => "backlight",
            DeviceClass::Leds => "leds",
        }
    }
}

/// A user-facing identifier used to match one or more Linux brightness devices.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeviceIdentifier {
    /// Restricts matching to a specific sysfs class.
    pub class: Option<DeviceClass>,
    /// Restricts matching to a specific device directory name.
    pub id: Option<String>,
    /// Restricts matching to an exact sysfs device path.
    pub path: Option<String>,
}

impl DeviceIdentifier {
    pub(crate) fn is_subset(&self, metadata: &DeviceMetadata) -> bool {
        if let Some(class) = self.class {
            if class != metadata.class {
                return false;
            }
        }

        if let Some(ref id) = self.id {
            if id != &metadata.id {
                return false;
            }
        }

        if let Some(ref path) = self.path {
            if path != &metadata.path {
                return false;
            }
        }

        true
    }
}

/// A request to update one or more Linux brightness devices.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DeviceUpdate {
    /// The user-facing identifier used to match one or more devices.
    pub id: DeviceIdentifier,
    /// The brightness operation to apply when the identifier matches.
    pub brightness: Option<BrightnessUpdate>,
}

/// Supported brightness update operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrightnessUpdate {
    /// Sets `brightness` to an absolute raw value.
    Raw(u32),
    /// Sets `brightness` to an absolute percentage of `max_brightness`.
    Percent(u8),
    /// Adds or subtracts a raw delta from the current `brightness` value.
    RawDelta(i32),
    /// Adds or subtracts a percentage delta from the current brightness percentage.
    PercentDelta(i32),
}

impl Device {
    /// Returns the device's sysfs path as a [`PathBuf`].
    pub fn path(&self) -> PathBuf {
        PathBuf::from(&self.metadata.path)
    }
}
