use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{ApplyError, QueryError};
use crate::types::{
    BrightnessUpdate, Device, DeviceClass, DeviceIdentifier, DeviceMetadata, DeviceState,
    DeviceUpdate,
};

/// High-level entry point for querying and updating Linux brightness devices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalDisplayManagerLinuxSys {
    sysfs_root: PathBuf,
}

impl Default for PhysicalDisplayManagerLinuxSys {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicalDisplayManagerLinuxSys {
    /// Creates a manager that reads devices from `/sys/class`.
    ///
    /// This is the normal constructor for real Linux systems.
    pub fn new() -> Self {
        Self {
            sysfs_root: PathBuf::from("/sys/class"),
        }
    }

    /// Creates a manager using a custom sysfs root.
    ///
    /// This is primarily useful for tests, fixtures, and integration scenarios
    /// where the sysfs tree is provided from somewhere other than `/sys/class`.
    pub fn with_sysfs_root(path: impl Into<PathBuf>) -> Self {
        Self {
            sysfs_root: path.into(),
        }
    }

    /// Lists all detected backlight and LED brightness devices.
    ///
    /// Devices are read from both `/sys/class/backlight` and `/sys/class/leds`
    /// beneath the configured sysfs root.
    pub fn list(&self) -> Result<Vec<Device>, QueryError> {
        self.list_by_classes([DeviceClass::Backlight, DeviceClass::Leds])
    }

    /// Lists detected brightness devices from the requested classes.
    pub fn list_by_classes(
        &self,
        classes: impl IntoIterator<Item = DeviceClass>,
    ) -> Result<Vec<Device>, QueryError> {
        let mut devices = Vec::new();
        for class in classes {
            devices.extend(self.list_class(class)?);
        }
        devices.sort_by(|left, right| left.metadata.cmp(&right.metadata));
        Ok(devices)
    }

    /// Looks up devices matching the provided user-facing identifiers.
    ///
    /// Matching is subset-based: every populated field on a [`DeviceIdentifier`]
    /// must match the corresponding field on the device metadata.
    /// When more than one device matches an identifier, the first sorted match is
    /// returned.
    pub fn get(
        &self,
        ids: BTreeSet<DeviceIdentifier>,
    ) -> Result<BTreeMap<DeviceIdentifier, Device>, QueryError> {
        let devices = self.list()?;
        Ok(ids
            .into_iter()
            .filter_map(|id| {
                devices
                    .iter()
                    .find(|device| id.is_subset(&device.metadata))
                    .cloned()
                    .map(|device| (id, device))
            })
            .collect())
    }

    /// Applies the requested device updates.
    ///
    /// Matching is subset-based and can target one or more devices per update.
    ///
    /// When `validate` is `true`, matching and value normalization are performed
    /// without writing to sysfs.
    ///
    /// Any update that does not match at least one device is returned unchanged
    /// in the result.
    pub fn apply(
        &self,
        updates: Vec<DeviceUpdate>,
        validate: bool,
    ) -> Result<Vec<DeviceUpdate>, ApplyError> {
        if updates.is_empty() {
            return Ok(Vec::new());
        }

        let devices = self.list()?;
        let mut remaining = Vec::new();

        for update in updates {
            let matched_devices: Vec<_> = devices
                .iter()
                .filter(|device| update.id.is_subset(&device.metadata))
                .cloned()
                .collect();

            if matched_devices.is_empty() {
                remaining.push(update);
                continue;
            }

            let Some(brightness_update) = update.brightness.clone() else {
                continue;
            };

            for device in matched_devices {
                let target_raw = normalize_brightness_update(&device.state, &brightness_update);
                if validate {
                    continue;
                }

                self.set_brightness_raw(&device.metadata, target_raw)?;
            }
        }

        Ok(remaining)
    }

    /// Applies device updates without validation-only mode.
    ///
    /// This is equivalent to calling [`Self::apply`] with `validate = false`.
    pub fn update(&self, updates: Vec<DeviceUpdate>) -> Result<Vec<DeviceUpdate>, ApplyError> {
        self.apply(updates, false)
    }

    /// Validates device updates without writing to sysfs.
    ///
    /// This is equivalent to calling [`Self::apply`] with `validate = true`.
    pub fn validate(&self, updates: Vec<DeviceUpdate>) -> Result<Vec<DeviceUpdate>, ApplyError> {
        self.apply(updates, true)
    }

    fn list_class(&self, class: DeviceClass) -> Result<Vec<Device>, QueryError> {
        let class_path = self.sysfs_root.join(class.directory_name());
        let entries =
            fs::read_dir(&class_path).map_err(|source| QueryError::ReadClassDirectory {
                path: class_path.clone(),
                source,
            })?;

        let mut devices = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|source| QueryError::ReadClassDirectory {
                path: class_path.clone(),
                source,
            })?;
            let device_path = entry.path();
            let file_type =
                entry
                    .file_type()
                    .map_err(|source| QueryError::ReadDeviceDirectory {
                        path: device_path.clone(),
                        source,
                    })?;
            if !file_type.is_dir() && !file_type.is_symlink() {
                continue;
            }

            let id = entry.file_name().to_string_lossy().into_owned();
            let state = read_state(&device_path)?;
            devices.push(Device {
                metadata: DeviceMetadata {
                    class,
                    id,
                    path: device_path.to_string_lossy().into_owned(),
                },
                state,
            });
        }

        Ok(devices)
    }

    /// Writes a raw brightness value directly to the sysfs `brightness` file.
    pub fn set_brightness_raw(
        &self,
        metadata: &DeviceMetadata,
        value: u32,
    ) -> Result<(), ApplyError> {
        let brightness_path = Path::new(&metadata.path).join("brightness");
        fs::write(&brightness_path, value.to_string()).map_err(|source| ApplyError::WriteFile {
            path: brightness_path,
            source,
        })
    }
}

fn read_state(device_path: &Path) -> Result<DeviceState, QueryError> {
    let brightness_raw = read_u32_file(&device_path.join("brightness"))?;
    let max_brightness_raw = read_u32_file(&device_path.join("max_brightness"))?;
    let actual_brightness_raw = read_optional_u32_file(&device_path.join("actual_brightness"))?;

    Ok(DeviceState {
        brightness_raw,
        max_brightness_raw,
        actual_brightness_raw,
        brightness_percent: percent_from_raw(brightness_raw, max_brightness_raw),
    })
}

fn read_optional_u32_file(path: &Path) -> Result<Option<u32>, QueryError> {
    if !path.exists() {
        return Ok(None);
    }
    read_u32_file(path).map(Some)
}

fn read_u32_file(path: &Path) -> Result<u32, QueryError> {
    if !path.exists() {
        return Err(QueryError::MissingFile {
            path: path.to_path_buf(),
        });
    }

    let content = fs::read_to_string(path).map_err(|source| QueryError::ReadFile {
        path: path.to_path_buf(),
        source,
    })?;
    let trimmed = content.trim();
    trimmed.parse::<u32>().map_err(|_| QueryError::ParseFile {
        path: path.to_path_buf(),
        content: trimmed.to_string(),
    })
}

/// Normalizes a brightness update into the raw value expected by Linux sysfs.
pub fn normalize_brightness_update(state: &DeviceState, update: &BrightnessUpdate) -> u32 {
    let max = state.max_brightness_raw;
    if max == 0 {
        return 0;
    }

    match *update {
        BrightnessUpdate::Raw(value) => value.min(max),
        BrightnessUpdate::Percent(percent) => raw_from_percent(percent, max),
        BrightnessUpdate::RawDelta(delta) => {
            clamp_i64(state.brightness_raw as i64 + delta as i64, max)
        }
        BrightnessUpdate::PercentDelta(delta) => {
            let current_percent = state.brightness_raw as f64 / max as f64 * 100.0;
            let target_percent = (current_percent + delta as f64).clamp(0.0, 100.0);
            raw_from_percent_f64(target_percent, max)
        }
    }
}

fn clamp_i64(value: i64, max: u32) -> u32 {
    value.clamp(0, max as i64) as u32
}

fn percent_from_raw(value: u32, max: u32) -> u8 {
    if max == 0 {
        return 0;
    }
    ((value as f64 / max as f64) * 100.0)
        .round()
        .clamp(0.0, 100.0) as u8
}

fn raw_from_percent(percent: u8, max: u32) -> u32 {
    let normalized = percent.min(100) as f64 / 100.0;
    (normalized * max as f64).round() as u32
}

fn raw_from_percent_f64(percent: f64, max: u32) -> u32 {
    let normalized = percent.clamp(0.0, 100.0) / 100.0;
    (normalized * max as f64).round() as u32
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::Path;

    use tempfile::TempDir;

    use super::PhysicalDisplayManagerLinuxSys;
    use crate::{BrightnessUpdate, DeviceClass, DeviceIdentifier, DeviceUpdate};

    #[test]
    fn list_enumerates_backlights_and_leds() {
        let fixture = Fixture::new();
        fixture.add_device(
            DeviceClass::Backlight,
            "intel_backlight",
            300,
            1200,
            Some(280),
        );
        fixture.add_device(DeviceClass::Leds, "asus::kbd_backlight", 1, 3, None);

        let manager = fixture.manager();
        let devices = manager.list().unwrap();

        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].metadata.class, DeviceClass::Backlight);
        assert_eq!(devices[0].metadata.id, "intel_backlight");
        assert_eq!(devices[0].state.brightness_percent, 25);
        assert_eq!(devices[0].state.actual_brightness_raw, Some(280));
        assert_eq!(devices[1].metadata.class, DeviceClass::Leds);
        assert_eq!(devices[1].metadata.id, "asus::kbd_backlight");
        assert_eq!(devices[1].state.brightness_percent, 33);
    }

    #[test]
    fn list_follows_sysfs_class_symlinks() {
        let fixture = Fixture::new();
        fixture.add_symlinked_device(DeviceClass::Backlight, "intel_backlight", 300, 1200, None);

        let devices = fixture.manager().list().unwrap();

        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].metadata.class, DeviceClass::Backlight);
        assert_eq!(devices[0].metadata.id, "intel_backlight");
        assert_eq!(devices[0].state.brightness_percent, 25);
    }

    #[test]
    fn get_matches_subset_identifiers() {
        let fixture = Fixture::new();
        fixture.add_device(DeviceClass::Backlight, "intel_backlight", 300, 1200, None);
        fixture.add_device(DeviceClass::Leds, "asus::kbd_backlight", 1, 3, None);

        let manager = fixture.manager();
        let mut ids = BTreeSet::new();
        ids.insert(DeviceIdentifier {
            class: Some(DeviceClass::Backlight),
            id: Some("intel_backlight".to_string()),
            path: None,
        });
        ids.insert(DeviceIdentifier {
            class: None,
            id: Some("asus::kbd_backlight".to_string()),
            path: None,
        });

        let devices = manager.get(ids).unwrap();
        assert_eq!(devices.len(), 2);
        assert!(devices
            .values()
            .any(|device| device.metadata.id == "intel_backlight"));
        assert!(devices
            .values()
            .any(|device| device.metadata.id == "asus::kbd_backlight"));
    }

    #[test]
    fn update_writes_raw_and_percent_values() {
        let fixture = Fixture::new();
        fixture.add_device(DeviceClass::Backlight, "intel_backlight", 100, 400, None);

        let manager = fixture.manager();
        let remaining = manager
            .update(vec![DeviceUpdate {
                id: DeviceIdentifier {
                    class: Some(DeviceClass::Backlight),
                    id: Some("intel_backlight".to_string()),
                    path: None,
                },
                brightness: Some(BrightnessUpdate::Percent(50)),
            }])
            .unwrap();

        assert!(remaining.is_empty());
        assert_eq!(
            fixture.read_brightness(DeviceClass::Backlight, "intel_backlight"),
            200
        );

        manager
            .update(vec![DeviceUpdate {
                id: DeviceIdentifier {
                    class: None,
                    id: Some("intel_backlight".to_string()),
                    path: None,
                },
                brightness: Some(BrightnessUpdate::Raw(123)),
            }])
            .unwrap();

        assert_eq!(
            fixture.read_brightness(DeviceClass::Backlight, "intel_backlight"),
            123
        );
    }

    #[test]
    fn update_supports_deltas_and_clamps_to_valid_range() {
        let fixture = Fixture::new();
        fixture.add_device(DeviceClass::Backlight, "intel_backlight", 100, 400, None);

        let manager = fixture.manager();
        manager
            .update(vec![DeviceUpdate {
                id: DeviceIdentifier {
                    class: Some(DeviceClass::Backlight),
                    id: Some("intel_backlight".to_string()),
                    path: None,
                },
                brightness: Some(BrightnessUpdate::RawDelta(50)),
            }])
            .unwrap();
        assert_eq!(
            fixture.read_brightness(DeviceClass::Backlight, "intel_backlight"),
            150
        );

        manager
            .update(vec![DeviceUpdate {
                id: DeviceIdentifier {
                    class: Some(DeviceClass::Backlight),
                    id: Some("intel_backlight".to_string()),
                    path: None,
                },
                brightness: Some(BrightnessUpdate::PercentDelta(50)),
            }])
            .unwrap();
        assert_eq!(
            fixture.read_brightness(DeviceClass::Backlight, "intel_backlight"),
            350
        );

        manager
            .update(vec![DeviceUpdate {
                id: DeviceIdentifier {
                    class: Some(DeviceClass::Backlight),
                    id: Some("intel_backlight".to_string()),
                    path: None,
                },
                brightness: Some(BrightnessUpdate::RawDelta(-500)),
            }])
            .unwrap();
        assert_eq!(
            fixture.read_brightness(DeviceClass::Backlight, "intel_backlight"),
            0
        );
    }

    #[test]
    fn update_applies_to_multiple_matching_devices() {
        let fixture = Fixture::new();
        fixture.add_device(DeviceClass::Leds, "kbd1", 1, 3, None);
        fixture.add_device(DeviceClass::Leds, "kbd2", 2, 3, None);

        let manager = fixture.manager();
        let remaining = manager
            .update(vec![DeviceUpdate {
                id: DeviceIdentifier {
                    class: Some(DeviceClass::Leds),
                    id: None,
                    path: None,
                },
                brightness: Some(BrightnessUpdate::Percent(100)),
            }])
            .unwrap();

        assert!(remaining.is_empty());
        assert_eq!(fixture.read_brightness(DeviceClass::Leds, "kbd1"), 3);
        assert_eq!(fixture.read_brightness(DeviceClass::Leds, "kbd2"), 3);
    }

    #[test]
    fn validate_checks_matches_without_writing() {
        let fixture = Fixture::new();
        fixture.add_device(DeviceClass::Backlight, "intel_backlight", 100, 400, None);

        let manager = fixture.manager();
        let remaining = manager
            .validate(vec![DeviceUpdate {
                id: DeviceIdentifier {
                    class: Some(DeviceClass::Backlight),
                    id: Some("intel_backlight".to_string()),
                    path: None,
                },
                brightness: Some(BrightnessUpdate::Percent(80)),
            }])
            .unwrap();

        assert!(remaining.is_empty());
        assert_eq!(
            fixture.read_brightness(DeviceClass::Backlight, "intel_backlight"),
            100
        );
    }

    #[test]
    fn unmatched_updates_are_returned() {
        let fixture = Fixture::new();
        fixture.add_device(DeviceClass::Backlight, "intel_backlight", 100, 400, None);

        let manager = fixture.manager();
        let remaining = manager
            .update(vec![DeviceUpdate {
                id: DeviceIdentifier {
                    class: Some(DeviceClass::Backlight),
                    id: Some("missing".to_string()),
                    path: None,
                },
                brightness: Some(BrightnessUpdate::Raw(20)),
            }])
            .unwrap();

        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id.id.as_deref(), Some("missing"));
    }

    struct Fixture {
        tempdir: TempDir,
    }

    impl Fixture {
        fn new() -> Self {
            let tempdir = TempDir::new().unwrap();
            fs::create_dir_all(tempdir.path().join("backlight")).unwrap();
            fs::create_dir_all(tempdir.path().join("leds")).unwrap();
            Self { tempdir }
        }

        fn manager(&self) -> PhysicalDisplayManagerLinuxSys {
            PhysicalDisplayManagerLinuxSys::with_sysfs_root(self.tempdir.path())
        }

        fn add_device(
            &self,
            class: DeviceClass,
            id: &str,
            brightness: u32,
            max_brightness: u32,
            actual_brightness: Option<u32>,
        ) {
            let device_path = self.tempdir.path().join(class.directory_name()).join(id);
            fs::create_dir_all(&device_path).unwrap();
            fs::write(device_path.join("brightness"), brightness.to_string()).unwrap();
            fs::write(
                device_path.join("max_brightness"),
                max_brightness.to_string(),
            )
            .unwrap();
            if let Some(actual_brightness) = actual_brightness {
                fs::write(
                    device_path.join("actual_brightness"),
                    actual_brightness.to_string(),
                )
                .unwrap();
            }
        }

        fn add_symlinked_device(
            &self,
            class: DeviceClass,
            id: &str,
            brightness: u32,
            max_brightness: u32,
            actual_brightness: Option<u32>,
        ) {
            let target_path = self.tempdir.path().join("devices").join(id);
            fs::create_dir_all(&target_path).unwrap();
            fs::write(target_path.join("brightness"), brightness.to_string()).unwrap();
            fs::write(
                target_path.join("max_brightness"),
                max_brightness.to_string(),
            )
            .unwrap();
            if let Some(actual_brightness) = actual_brightness {
                fs::write(
                    target_path.join("actual_brightness"),
                    actual_brightness.to_string(),
                )
                .unwrap();
            }

            let class_entry_path = self.tempdir.path().join(class.directory_name()).join(id);
            std::os::unix::fs::symlink(&target_path, &class_entry_path).unwrap();
        }

        fn read_brightness(&self, class: DeviceClass, id: &str) -> u32 {
            let path = self
                .tempdir
                .path()
                .join(class.directory_name())
                .join(id)
                .join("brightness");
            read_u32(&path)
        }
    }

    fn read_u32(path: &Path) -> u32 {
        fs::read_to_string(path)
            .unwrap()
            .trim()
            .parse::<u32>()
            .unwrap()
    }
}
