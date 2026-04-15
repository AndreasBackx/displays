use std::collections::{BTreeMap, BTreeSet};
use std::io::ErrorKind;
use std::path::PathBuf;

use displays_physical_linux_logind::PhysicalDisplayManagerLinuxLogind;
use displays_physical_linux_sys::{
    normalize_brightness_update, Device, DeviceIdentifier, DeviceMetadata, DeviceState,
    DeviceUpdate, PhysicalDisplayManagerLinuxSys, QueryError,
};

use crate::error::ApplyError;

/// High-level entry point for querying and updating Linux brightness devices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalDisplayManagerLinux {
    sys: PhysicalDisplayManagerLinuxSys,
    logind: PhysicalDisplayManagerLinuxLogind,
}

impl Default for PhysicalDisplayManagerLinux {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicalDisplayManagerLinux {
    /// Creates a manager that reads devices from `/sys/class` and falls back to logind for writes.
    pub fn new() -> Self {
        Self {
            sys: PhysicalDisplayManagerLinuxSys::new(),
            logind: PhysicalDisplayManagerLinuxLogind::new(),
        }
    }

    /// Creates a manager using a custom sysfs root.
    pub fn with_sysfs_root(path: impl Into<PathBuf>) -> Self {
        Self {
            sys: PhysicalDisplayManagerLinuxSys::with_sysfs_root(path),
            logind: PhysicalDisplayManagerLinuxLogind::new(),
        }
    }

    /// Lists all detected brightness-capable Linux devices.
    pub fn list(&self) -> Result<Vec<Device>, QueryError> {
        self.sys.list()
    }

    /// Lists detected Linux brightness devices from the requested classes.
    pub fn list_by_classes(
        &self,
        classes: impl IntoIterator<Item = displays_physical_linux_sys::DeviceClass>,
    ) -> Result<Vec<Device>, QueryError> {
        self.sys.list_by_classes(classes)
    }

    /// Looks up devices matching the provided identifiers.
    pub fn get(
        &self,
        ids: BTreeSet<DeviceIdentifier>,
    ) -> Result<BTreeMap<DeviceIdentifier, Device>, QueryError> {
        self.sys.get(ids)
    }

    /// Applies the requested device updates.
    pub fn apply(
        &self,
        updates: Vec<DeviceUpdate>,
        validate: bool,
    ) -> Result<Vec<DeviceUpdate>, ApplyError> {
        if updates.is_empty() {
            return Ok(Vec::new());
        }

        let devices = self.sys.list()?;
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

            if validate {
                continue;
            }

            for device in matched_devices {
                self.apply_to_device(&device.metadata, &device.state, &brightness_update)?;
            }
        }

        Ok(remaining)
    }

    /// Applies device updates without validation-only mode.
    pub fn update(&self, updates: Vec<DeviceUpdate>) -> Result<Vec<DeviceUpdate>, ApplyError> {
        self.apply(updates, false)
    }

    /// Validates device updates without writing to sysfs or logind.
    pub fn validate(&self, updates: Vec<DeviceUpdate>) -> Result<Vec<DeviceUpdate>, ApplyError> {
        self.apply(updates, true)
    }

    fn apply_to_device(
        &self,
        metadata: &DeviceMetadata,
        state: &DeviceState,
        update: &displays_physical_linux_sys::BrightnessUpdate,
    ) -> Result<(), ApplyError> {
        let target_raw = normalize_brightness_update(state, update);
        match self.sys.set_brightness_raw(metadata, target_raw) {
            Ok(()) => Ok(()),
            Err(displays_physical_linux_sys::ApplyError::WriteFile { source, .. })
                if source.kind() == ErrorKind::PermissionDenied =>
            {
                self.logind
                    .set_brightness(metadata.class, &metadata.id, target_raw)
                    .map_err(Into::into)
            }
            Err(err) => Err(err.into()),
        }
    }
}
