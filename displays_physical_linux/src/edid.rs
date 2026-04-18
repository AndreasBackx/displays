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
        .or_else(|| info.serial.filter(|serial| *serial != 0).map(|serial| serial.to_string()));
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
    use std::collections::BTreeSet;
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
        assert_eq!(metadata.serial_number, None);
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

    #[test]
    #[ignore = "uses the optional EDID corpus submodule"]
    fn parses_sample_of_edid_corpus_files() {
        let corpus_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../EDID");
        if !corpus_root.exists() {
            return;
        }

        let mut files = Vec::new();
        collect_corpus_files(&corpus_root.join("Digital"), &mut files);
        files.sort();

        let total = files.len();
        let mut parsed = 0usize;
        let mut empty_entries = 0usize;
        let mut decode_failures = BTreeSet::new();
        let mut parse_failures = BTreeSet::new();
        for file in files.into_iter() {
            match decode_linuxhw_edid_file(&file) {
                Some(CorpusEdidDecode::Bytes(edid)) => {
                    // The parser may return None for real-world corpus files when the decoded
                    // file contains EDID bytes, but the EDID itself is malformed or unsupported.
                    let metadata = metadata_from_edid_bytes(
                        file.display().to_string(),
                        "fixture-display".to_string(),
                        &edid,
                    );
                    if metadata.is_none() {
                        parse_failures.insert(relative_corpus_path(&corpus_root, &file));
                    }
                    parsed += 1;
                }
                Some(CorpusEdidDecode::EmptyEntry) => {
                    empty_entries += 1;
                }
                None => {
                    decode_failures.insert(relative_corpus_path(&corpus_root, &file));
                }
            };
        }

        assert!(parsed > 0, "expected to parse at least one EDID corpus file");
        assert!(
            decode_failures.is_empty(),
            "expected no EDID corpus decode failures, these failed {decode_failures:#?}"
        );
        assert!(
            parse_failures.is_empty(),
            "expected no EDID corpus parse failures, these failed {parse_failures:#?}"
        );

        eprintln!("{empty_entries}/{total} corpus files contain no EDID payload");
    }

    fn collect_corpus_files(root: &Path, files: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(root) else {
            return;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_corpus_files(&path, files);
                continue;
            }

            if path.file_name().and_then(|file_name| file_name.to_str()).is_some_and(is_corpus_file_name) {
                files.push(path);
            }
        }
    }

    fn is_corpus_file_name(file_name: &str) -> bool {
        file_name.len() == 12 && file_name.chars().all(|character| character.is_ascii_hexdigit())
    }

    fn relative_corpus_path(corpus_root: &Path, path: &Path) -> PathBuf {
        path.strip_prefix(corpus_root)
            .unwrap_or(path)
            .to_path_buf()
    }

    enum CorpusEdidDecode {
        Bytes(Vec<u8>),
        EmptyEntry,
    }

    fn decode_linuxhw_edid_file(path: &Path) -> Option<CorpusEdidDecode> {
        let contents = fs::read_to_string(path).ok()?;
        if contents.trim() == "EDID of 'stdin' was empty." {
            return Some(CorpusEdidDecode::EmptyEntry);
        }

        let mut bytes = Vec::new();

        for line in contents.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                if !bytes.is_empty() {
                    break;
                }
                continue;
            }

            if trimmed.chars().all(|character| character.is_ascii_hexdigit() || character == ' ') {
                for chunk in trimmed.split_whitespace() {
                    bytes.push(u8::from_str_radix(chunk, 16).ok()?);
                }
            } else if !bytes.is_empty() {
                break;
            }
        }

        (!bytes.is_empty()).then_some(CorpusEdidDecode::Bytes(bytes))
    }
}
