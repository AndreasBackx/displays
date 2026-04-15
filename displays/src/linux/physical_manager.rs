use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::sync::mpsc;
use std::time::Duration;

use ddc_hi::{Ddc, Display as DdcDisplay, FeatureCode};
use displays_physical_linux::{
    BrightnessUpdate as LinuxBrightnessUpdate, Device, DeviceClass, DeviceIdentifier, DeviceUpdate,
    PhysicalDisplayManagerLinux as LinuxPhysicalDisplayManager,
    QueryError as LinuxPhysicalQueryError,
};
use thiserror::Error;

use crate::{
    display::{Brightness, DisplayUpdate},
    display_identifier::{DisplayIdentifier, DisplayIdentifierInner},
    physical_display::{
        PhysicalDisplay, PhysicalDisplayMetadata, PhysicalDisplayState,
        PhysicalDisplayUpdateContent,
    },
};

#[derive(Error, Debug)]
pub enum PhysicalDisplayQueryError {
    #[error("failed to enumerate DDC displays")]
    Enumerate,
    #[error("missing i2c access for display '{display_id}'")]
    MissingI2cAccess { display_id: String },
    #[error("insufficient permissions for display '{display_id}'")]
    PermissionDenied { display_id: String },
    #[error("display '{display_id}' does not expose brightness via VCP 0x10: {message}")]
    UnsupportedMonitor { display_id: String, message: String },
    #[error("ddc operation failed for display '{display_id}': {message}")]
    DdcOperation { display_id: String, message: String },
    #[error("failed to query Linux backlight devices: {message}")]
    BacklightQuery { message: String },
}

#[derive(Error, Debug)]
pub enum PhysicalDisplayApplyError {
    #[error(transparent)]
    Query {
        #[from]
        source: PhysicalDisplayQueryError,
    },
    #[error("display '{display_id}' does not expose brightness via VCP 0x10: {message}")]
    UnsupportedMonitor { display_id: String, message: String },
    #[error("insufficient permissions for display '{display_id}'")]
    PermissionDenied { display_id: String },
    #[error("missing i2c access for display '{display_id}'")]
    MissingI2cAccess { display_id: String },
    #[error("failed to set brightness for display '{display_id}': {message}")]
    DdcOperation { display_id: String, message: String },
    #[error("failed to set backlight brightness for display '{display_id}': {message}")]
    BacklightOperation { display_id: String, message: String },
}

pub struct PhysicalDisplayManagerLinux;

const PER_MONITOR_APPLY_TIMEOUT: Duration = Duration::from_millis(3500);

#[derive(Clone)]
struct LinuxDisplayHandle {
    metadata: PhysicalDisplayMetadata,
    state: PhysicalDisplayState,
    backend: LinuxPhysicalBackend,
}

#[derive(Clone)]
enum LinuxPhysicalBackend {
    Ddc { display_index: usize },
    Backlight { path: String },
}

#[derive(Clone)]
struct DdcApplyUpdate {
    id: DisplayIdentifierInner,
    brightness: Option<u32>,
    display_index: usize,
}

#[derive(Clone)]
struct BacklightApplyUpdate {
    id: DisplayIdentifierInner,
    brightness: Option<u32>,
    path: String,
}

impl LinuxDisplayHandle {
    fn id(&self) -> DisplayIdentifierInner {
        id_from_metadata(&self.metadata)
    }
}

impl PhysicalDisplayManagerLinux {
    pub fn query() -> Result<Vec<PhysicalDisplay>, PhysicalDisplayQueryError> {
        Ok(Self::query_handles()?
            .into_iter()
            .map(|handle| PhysicalDisplay {
                metadata: handle.metadata,
                state: handle.state,
            })
            .collect())
    }

    pub(crate) fn apply_display_updates(
        updates: Vec<DisplayUpdate>,
        validate: bool,
    ) -> Result<Vec<DisplayUpdate>, PhysicalDisplayApplyError> {
        if updates.is_empty() {
            return Ok(Vec::new());
        }

        let handles = Self::query_handles()?;
        let mut remaining_updates = Vec::new();
        let mut ddc_updates = Vec::new();
        let mut backlight_updates = Vec::new();

        for update in updates {
            let matched_handles: Vec<_> = handles
                .iter()
                .filter(|handle| update.id.is_subset(&handle.id().outer))
                .collect();

            if matched_handles.is_empty() {
                remaining_updates.push(update);
                continue;
            }

            if validate {
                continue;
            }

            for handle in matched_handles {
                let brightness = update
                    .physical
                    .as_ref()
                    .and_then(|physical| physical.brightness);

                match &handle.backend {
                    LinuxPhysicalBackend::Ddc { display_index } => {
                        ddc_updates.push(DdcApplyUpdate {
                            id: handle.id(),
                            brightness,
                            display_index: *display_index,
                        });
                    }
                    LinuxPhysicalBackend::Backlight { path } => {
                        backlight_updates.push(BacklightApplyUpdate {
                            id: handle.id(),
                            brightness,
                            path: path.clone(),
                        });
                    }
                }
            }
        }

        remaining_updates.extend(Self::apply_ddc_updates(ddc_updates));
        remaining_updates.extend(Self::apply_backlight_updates(backlight_updates)?);
        Ok(remaining_updates)
    }

    fn query_handles() -> Result<Vec<LinuxDisplayHandle>, PhysicalDisplayQueryError> {
        let mut handles = Self::enumerate_ddc_handles()?;
        handles.extend(Self::enumerate_backlight_handles()?);
        handles.sort_by(|left, right| left.metadata.cmp(&right.metadata));
        Ok(handles)
    }

    fn enumerate_ddc_handles() -> Result<Vec<LinuxDisplayHandle>, PhysicalDisplayQueryError> {
        let mut handles = Vec::new();

        for (display_index, mut display) in DdcDisplay::enumerate().into_iter().enumerate() {
            let info = display.info.clone();
            let display_id = info.id.clone();
            let brightness = match display.handle.get_vcp_feature(FeatureCode::from(0x10)) {
                Ok(vcp) => {
                    let maximum = vcp.maximum();
                    if maximum == 0 {
                        tracing::warn!(
                            "Skipping display '{}' because brightness max was reported as 0",
                            display_id
                        );
                        continue;
                    }
                    ((vcp.value() as f64 / maximum as f64) * 100.0).round() as u8
                }
                Err(err) => {
                    let message = err.to_string();
                    if is_io_error(&message) {
                        tracing::warn!(
                            "Assuming 0% brightness for display '{}' due to I/O error: {}",
                            display_id,
                            message
                        );
                        0
                    } else {
                        tracing::warn!(
                            "Skipping display '{}' due to query error: {}",
                            display_id,
                            message
                        );
                        continue;
                    }
                }
            };

            let metadata = metadata_from_info(&info);

            handles.push(LinuxDisplayHandle {
                metadata,
                state: PhysicalDisplayState {
                    brightness: Brightness::new(brightness.min(100)),
                    scale_factor: 100,
                },
                backend: LinuxPhysicalBackend::Ddc { display_index },
            });
        }

        Ok(handles)
    }

    fn enumerate_backlight_handles() -> Result<Vec<LinuxDisplayHandle>, PhysicalDisplayQueryError> {
        let manager = LinuxPhysicalDisplayManager::new();
        // Only surface `/sys/class/backlight` devices through `displays` for now.
        // If we later decide LED brightness belongs here too, widen this class list
        // and add an explicit conversion path instead of silently folding LEDs into
        // monitor-like metadata.
        match manager.list_by_classes([DeviceClass::Backlight]) {
            Ok(devices) => devices
                .into_iter()
                .map(backlight_handle_from_device)
                .collect(),
            Err(LinuxPhysicalQueryError::ReadClassDirectory { source, .. })
                if source.kind() == ErrorKind::NotFound =>
            {
                Ok(Vec::new())
            }
            Err(err) => Err(PhysicalDisplayQueryError::BacklightQuery {
                message: err.to_string(),
            }),
        }
    }

    #[cfg(test)]
    fn enumerate_backlight_handles_with_manager(
        manager: &displays_physical_linux_sys::PhysicalDisplayManagerLinuxSys,
    ) -> Result<Vec<LinuxDisplayHandle>, PhysicalDisplayQueryError> {
        match manager.list_by_classes([DeviceClass::Backlight]) {
            Ok(devices) => devices
                .into_iter()
                .map(backlight_handle_from_device)
                .collect(),
            Err(displays_physical_linux_sys::QueryError::ReadClassDirectory { source, .. })
                if source.kind() == ErrorKind::NotFound =>
            {
                Ok(Vec::new())
            }
            Err(err) => Err(PhysicalDisplayQueryError::BacklightQuery {
                message: err.to_string(),
            }),
        }
    }

    fn apply_ddc_updates(updates: Vec<DdcApplyUpdate>) -> Vec<DisplayUpdate> {
        if updates.is_empty() {
            return Vec::new();
        }

        let mut remaining_updates = Vec::new();
        let mut display_by_index: BTreeMap<usize, DdcDisplay> =
            DdcDisplay::enumerate().into_iter().enumerate().collect();

        for update in updates {
            let Some(brightness) = update.brightness else {
                continue;
            };
            let outer_id = update.id.outer;

            let Some(display) = display_by_index.remove(&update.display_index) else {
                remaining_updates.push(display_update_with_brightness(outer_id, brightness));
                continue;
            };

            let display_id = display.info.id.clone();
            if let Err(err) = Self::set_brightness_with_timeout(display, brightness) {
                tracing::warn!(
                    "Failed to set brightness for display '{}': {}",
                    display_id,
                    err
                );
                remaining_updates.push(display_update_with_brightness(outer_id, brightness));
            }
        }

        remaining_updates
    }

    fn apply_backlight_updates(
        updates: Vec<BacklightApplyUpdate>,
    ) -> Result<Vec<DisplayUpdate>, PhysicalDisplayApplyError> {
        if updates.is_empty() {
            return Ok(Vec::new());
        }

        let manager = LinuxPhysicalDisplayManager::new();
        let mut remaining_updates = Vec::new();

        for update in updates {
            let Some(brightness) = update.brightness else {
                continue;
            };

            let request = DeviceUpdate {
                id: DeviceIdentifier {
                    class: Some(DeviceClass::Backlight),
                    id: None,
                    path: Some(update.path.clone()),
                },
                brightness: Some(LinuxBrightnessUpdate::Percent(brightness.min(100) as u8)),
            };

            match manager.update(vec![request]) {
                Ok(remaining) if remaining.is_empty() => {}
                Ok(_) => remaining_updates
                    .push(display_update_with_brightness(update.id.outer, brightness)),
                Err(err) => {
                    tracing::warn!(
                        "Failed to set backlight brightness for display '{}': {}",
                        update.path,
                        err
                    );
                    remaining_updates
                        .push(display_update_with_brightness(update.id.outer, brightness));
                }
            }
        }

        Ok(remaining_updates)
    }

    fn set_brightness(
        display: &mut DdcDisplay,
        brightness: u32,
    ) -> Result<(), PhysicalDisplayApplyError> {
        let ddc_id = display.info.id.clone();
        let normalized = brightness.min(100);
        let target_value = if normalized == 0 {
            0
        } else {
            let vcp = display
                .handle
                .get_vcp_feature(FeatureCode::from(0x10))
                .map_err(|err| classify_apply_error(ddc_id.clone(), err.to_string()))?;

            let max = vcp.maximum();
            if max == 0 {
                return Err(PhysicalDisplayApplyError::UnsupportedMonitor {
                    display_id: ddc_id,
                    message: "reported brightness max value is 0".to_string(),
                });
            }

            let percent = normalized as f64 / 100.0;
            (percent * max as f64).round() as u16
        };

        display
            .handle
            .set_vcp_feature(FeatureCode::from(0x10), target_value)
            .map_err(|err| classify_apply_error(display.info.id.clone(), err.to_string()))
    }

    fn set_brightness_with_timeout(
        display: DdcDisplay,
        brightness: u32,
    ) -> Result<(), PhysicalDisplayApplyError> {
        let display_id = display.info.id.clone();
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let mut display = display;
            let result = Self::set_brightness(&mut display, brightness);
            let _ = sender.send(result);
        });

        match receiver.recv_timeout(PER_MONITOR_APPLY_TIMEOUT) {
            Ok(result) => result,
            Err(mpsc::RecvTimeoutError::Timeout) => Err(PhysicalDisplayApplyError::DdcOperation {
                display_id,
                message: format!("timed out after {:?}", PER_MONITOR_APPLY_TIMEOUT),
            }),
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                Err(PhysicalDisplayApplyError::DdcOperation {
                    display_id,
                    message: "apply worker disconnected unexpectedly".to_string(),
                })
            }
        }
    }
}

fn backlight_handle_from_device(
    device: Device,
) -> Result<LinuxDisplayHandle, PhysicalDisplayQueryError> {
    let path = device.metadata.path;
    let name = device.metadata.id;

    let metadata = PhysicalDisplayMetadata {
        path: path.clone(),
        name,
        serial_number: String::new(),
    };

    Ok(LinuxDisplayHandle {
        metadata,
        state: PhysicalDisplayState {
            brightness: Brightness::new(device.state.brightness_percent),
            scale_factor: 100,
        },
        backend: LinuxPhysicalBackend::Backlight { path },
    })
}

fn display_update_with_brightness(id: DisplayIdentifier, brightness: u32) -> DisplayUpdate {
    DisplayUpdate {
        id,
        logical: None,
        physical: Some(PhysicalDisplayUpdateContent {
            brightness: Some(brightness),
        }),
    }
}

fn id_from_metadata(metadata: &PhysicalDisplayMetadata) -> DisplayIdentifierInner {
    DisplayIdentifierInner {
        outer: DisplayIdentifier {
            name: Some(metadata.name.clone()),
            serial_number: Some(metadata.serial_number.clone()),
        },
        path: Some(metadata.path.clone()),
        gdi_device_id: None,
    }
}

fn metadata_from_info(info: &ddc_hi::DisplayInfo) -> PhysicalDisplayMetadata {
    let name = info
        .model_name
        .clone()
        .unwrap_or_else(|| format!("Display {}", info.id));

    let serial_number = info
        .serial_number
        .clone()
        .or_else(|| info.serial.map(|serial| serial.to_string()))
        .unwrap_or_else(|| format!("fallback-{}", stable_fallback_id(&info.id)));

    PhysicalDisplayMetadata {
        path: info.id.clone(),
        name,
        serial_number,
    }
}

fn stable_fallback_id(value: &str) -> String {
    let mut hash: u64 = 1469598103934665603;
    for byte in value.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("{hash:016x}")
}

fn classify_apply_error(display_id: String, message: String) -> PhysicalDisplayApplyError {
    let lowercase = message.to_lowercase();
    if lowercase.contains("permission denied") {
        return PhysicalDisplayApplyError::PermissionDenied { display_id };
    }
    if lowercase.contains("/dev/i2c") || lowercase.contains("no such file") {
        return PhysicalDisplayApplyError::MissingI2cAccess { display_id };
    }
    if lowercase.contains("unsupported") || lowercase.contains("vcp") {
        return PhysicalDisplayApplyError::UnsupportedMonitor {
            display_id,
            message,
        };
    }
    PhysicalDisplayApplyError::DdcOperation {
        display_id,
        message,
    }
}

fn is_io_error(message: &str) -> bool {
    let lowercase = message.to_lowercase();
    lowercase.contains("input/output error") || lowercase.contains("os error 5")
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::{BacklightFixture, PhysicalDisplayManagerLinux};
    use crate::{
        display::DisplayUpdate, display_identifier::DisplayIdentifier,
        physical_display::PhysicalDisplayUpdateContent,
    };

    #[test]
    fn enumerate_backlights_ignores_missing_class_directory() {
        let tempdir = TempDir::new().unwrap();
        let manager = displays_physical_linux_sys::PhysicalDisplayManagerLinuxSys::with_sysfs_root(
            tempdir.path(),
        );

        let displays =
            PhysicalDisplayManagerLinux::enumerate_backlight_handles_with_manager(&manager)
                .unwrap();

        assert!(displays.is_empty());
    }

    #[test]
    fn enumerate_backlights_only_surfaces_backlight_devices() {
        let fixture = BacklightFixture::new();
        fixture.add_backlight("intel_backlight", 300, 1200);
        fixture.add_led("asus::kbd_backlight", 1, 3);

        let displays = PhysicalDisplayManagerLinux::enumerate_backlight_handles_with_manager(
            &fixture.manager(),
        )
        .unwrap();

        assert_eq!(displays.len(), 1);
        assert_eq!(displays[0].metadata.name, "intel_backlight");
        assert_eq!(displays[0].metadata.serial_number, "");
        assert_eq!(displays[0].state.brightness.value(), 25);
    }

    #[test]
    fn apply_backlight_updates_writes_percent_brightness() {
        let fixture = BacklightFixture::new();
        fixture.add_backlight("intel_backlight", 100, 400);

        let update = DisplayUpdate {
            id: DisplayIdentifier {
                name: Some("intel_backlight".to_string()),
                serial_number: None,
            },
            logical: None,
            physical: Some(PhysicalDisplayUpdateContent {
                brightness: Some(50),
            }),
        };

        let handles = PhysicalDisplayManagerLinux::enumerate_backlight_handles_with_manager(
            &fixture.manager(),
        )
        .unwrap();
        let matched = handles
            .iter()
            .find(|handle| update.id.is_subset(&handle.id().outer))
            .unwrap();

        let remaining = PhysicalDisplayManagerLinux::apply_backlight_updates_with_manager(
            vec![super::BacklightApplyUpdate {
                id: matched.id(),
                brightness: Some(50),
                path: matched.metadata.path.clone(),
            }],
            &fixture.manager(),
        )
        .unwrap();

        assert!(remaining.is_empty());
        assert_eq!(fixture.read_brightness("intel_backlight"), 200);
    }
}

#[cfg(test)]
impl PhysicalDisplayManagerLinux {
    fn apply_backlight_updates_with_manager(
        updates: Vec<BacklightApplyUpdate>,
        manager: &displays_physical_linux_sys::PhysicalDisplayManagerLinuxSys,
    ) -> Result<Vec<DisplayUpdate>, PhysicalDisplayApplyError> {
        if updates.is_empty() {
            return Ok(Vec::new());
        }

        let mut remaining_updates = Vec::new();

        for update in updates {
            let Some(brightness) = update.brightness else {
                continue;
            };

            let request = DeviceUpdate {
                id: DeviceIdentifier {
                    class: Some(DeviceClass::Backlight),
                    id: None,
                    path: Some(update.path.clone()),
                },
                brightness: Some(LinuxBrightnessUpdate::Percent(brightness.min(100) as u8)),
            };

            match manager.update(vec![request]) {
                Ok(remaining) if remaining.is_empty() => {}
                Ok(_) => remaining_updates
                    .push(display_update_with_brightness(update.id.outer, brightness)),
                Err(err) => {
                    return Err(PhysicalDisplayApplyError::BacklightOperation {
                        display_id: update.path.clone(),
                        message: err.to_string(),
                    })
                }
            }
        }

        Ok(remaining_updates)
    }
}

#[cfg(test)]
struct BacklightFixture {
    tempdir: tempfile::TempDir,
}

#[cfg(test)]
impl BacklightFixture {
    fn new() -> Self {
        let tempdir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(tempdir.path().join("backlight")).unwrap();
        std::fs::create_dir_all(tempdir.path().join("leds")).unwrap();
        Self { tempdir }
    }

    fn manager(&self) -> displays_physical_linux_sys::PhysicalDisplayManagerLinuxSys {
        displays_physical_linux_sys::PhysicalDisplayManagerLinuxSys::with_sysfs_root(
            self.tempdir.path(),
        )
    }

    fn add_backlight(&self, id: &str, brightness: u32, max_brightness: u32) {
        self.add_device("backlight", id, brightness, max_brightness);
    }

    fn add_led(&self, id: &str, brightness: u32, max_brightness: u32) {
        self.add_device("leds", id, brightness, max_brightness);
    }

    fn add_device(&self, class: &str, id: &str, brightness: u32, max_brightness: u32) {
        let device_path = self.tempdir.path().join(class).join(id);
        std::fs::create_dir_all(&device_path).unwrap();
        std::fs::write(device_path.join("brightness"), brightness.to_string()).unwrap();
        std::fs::write(
            device_path.join("max_brightness"),
            max_brightness.to_string(),
        )
        .unwrap();
    }

    fn read_brightness(&self, id: &str) -> u32 {
        let content = std::fs::read_to_string(
            self.tempdir
                .path()
                .join("backlight")
                .join(id)
                .join("brightness"),
        )
        .unwrap();
        content.trim().parse().unwrap()
    }
}
