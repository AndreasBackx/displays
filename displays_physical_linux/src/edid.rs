use std::fs;
use std::path::{Path, PathBuf};

use displays_physical_types::PhysicalDisplayMetadata;
use ddc_hi::{Backend, DisplayInfo};

pub(crate) fn metadata_from_backlight_path(path: &str, fallback_name: &str) -> Option<PhysicalDisplayMetadata> {
    let edid_path = drm_connector_dir_from_backlight_path(Path::new(path))?.join("edid");
    let bytes = fs::read(edid_path).ok()?;
    metadata_from_edid_bytes(path.to_string(), fallback_name.to_string(), &bytes)
}

pub(crate) fn drm_connector_dir_from_backlight_path(path: &Path) -> Option<PathBuf> {
    path.ancestors().find_map(|ancestor| {
        let file_name = ancestor.file_name()?.to_str()?;
        let has_edid = ancestor.join("edid").is_file();
        (file_name.starts_with("card") && file_name.contains('-') && has_edid)
            .then(|| ancestor.to_path_buf())
    })
}

pub(crate) fn metadata_from_edid_bytes(
    path: String,
    fallback_name: String,
    bytes: &[u8],
) -> Option<PhysicalDisplayMetadata> {
    let info = DisplayInfo::from_edid(Backend::I2cDevice, path.clone(), bytes.to_vec()).ok()?;
    Some(metadata_from_info(path, fallback_name, info))
}

fn metadata_from_info(
    path: String,
    fallback_name: String,
    info: DisplayInfo,
) -> PhysicalDisplayMetadata {
    let model = info
        .model_name
        .clone()
        .or_else(|| info.model_id.map(|model_id| format!("0x{model_id:04X}")));
    let serial_number = info
        .serial_number
        .clone()
        .or_else(|| info.serial.filter(|serial| *serial != 0).map(|serial| serial.to_string()))
        .unwrap_or_default();
    let name = info.model_name.unwrap_or(fallback_name);

    PhysicalDisplayMetadata {
        path,
        name,
        manufacturer: info.manufacturer_id,
        model,
        serial_number,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::{drm_connector_dir_from_backlight_path, metadata_from_backlight_path, metadata_from_edid_bytes};

    #[test]
    fn extracts_drm_connector_dir_from_backlight_path() {
        let path = Path::new(
            "/sys/devices/pci0000:00/0000:00:02.0/drm/card0/card0-eDP-1/intel_backlight",
        );

        assert_eq!(
            drm_connector_dir_from_backlight_path(path),
            Some(PathBuf::from(
                "/sys/devices/pci0000:00/0000:00:02.0/drm/card0/card0-eDP-1"
            ))
        );
    }

    #[test]
    fn parses_metadata_from_edid_bytes() {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/linux_internal_panel_lgd_0x07c6_2880x1800.edid");
        let edid = fs::read(&fixture).expect("fixture edid should be readable");
        let metadata = metadata_from_edid_bytes(
            "/sys/class/drm/card0-eDP-1".to_string(),
            "intel_backlight".to_string(),
            &edid,
        )
        .expect("fixture edid should parse");

        assert_eq!(metadata.path, "/sys/class/drm/card0-eDP-1");
        assert_eq!(metadata.name, "intel_backlight");
        assert_eq!(metadata.manufacturer.as_deref(), Some("LGD"));
        assert_eq!(metadata.model.as_deref(), Some("0x07C6"));
        assert_eq!(metadata.serial_number, "");
    }

    #[test]
    fn returns_none_when_backlight_edid_is_missing() {
        let fixture = tempfile::tempdir().unwrap();
        let backlight = fixture
            .path()
            .join("pci0000:00/0000:00:02.0/drm/card0/card0-eDP-1/intel_backlight");
        fs::create_dir_all(&backlight).unwrap();

        assert_eq!(
            metadata_from_backlight_path(backlight.to_str().unwrap(), "intel_backlight"),
            None
        );
    }
}
