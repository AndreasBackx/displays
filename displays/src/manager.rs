use thiserror::Error;

use displays_logical_types::{LogicalDisplay, LogicalDisplayUpdate};
use displays_physical_types::{PhysicalDisplay, PhysicalDisplayUpdate};
use displays_types::{DisplayIdentifier, DisplayIdentifierInner};

use crate::display::{Display, DisplayUpdate};

#[cfg(target_os = "windows")]
use displays_logical_windows::{
    ApplyError as LogicalDisplayApplyError, LogicalDisplayManager,
    QueryError as LogicalDisplayQueryError,
};

#[cfg(target_os = "windows")]
use displays_physical_windows::{
    ApplyError as PhysicalDisplayApplyError, PhysicalDisplayManager,
    QueryError as PhysicalDisplayQueryError,
};

#[cfg(target_os = "linux")]
use displays_logical_linux::{
    ApplyError as LogicalDisplayApplyError, LogicalDisplayManager,
    QueryError as LogicalDisplayQueryError,
};

#[cfg(target_os = "linux")]
use displays_physical_linux::{
    ApplyError as PhysicalDisplayApplyError, PhysicalDisplayManager,
    QueryError as PhysicalDisplayQueryError,
};

/// Errors that can occur while querying display state.
#[derive(Error, Debug)]
pub enum DisplayQueryError {
    #[error("physical querying error")]
    Physical {
        #[from]
        source: PhysicalDisplayQueryError,
    },
    #[error("logical querying error")]
    Logical {
        #[from]
        source: LogicalDisplayQueryError,
    },
}

/// Errors that can occur while applying display updates.
#[derive(Error, Debug)]
pub enum DisplayApplyError {
    #[error("error while first querying displays")]
    Query {
        #[from]
        source: DisplayQueryError,
    },
    #[error("physical applying error: {source}")]
    Physical {
        #[from]
        source: PhysicalDisplayApplyError,
    },
    #[error("logical applying error")]
    Logical {
        #[from]
        source: LogicalDisplayApplyError,
    },
}

/// A concrete display matched by a user-facing identifier.
#[derive(Debug, Clone)]
pub struct DisplayMatch {
    pub requested_id: DisplayIdentifier,
    pub matched_id: DisplayIdentifier,
    pub display: Display,
}

/// A per-display failure encountered while applying an update.
#[derive(Debug, Clone)]
pub struct FailedDisplayUpdate {
    pub matched_id: DisplayIdentifier,
    pub message: String,
}

/// Best-effort result for a single requested display update.
#[derive(Debug, Clone)]
pub struct DisplayUpdateResult {
    pub requested_update: DisplayUpdate,
    pub applied: Vec<DisplayIdentifier>,
    pub failed: Vec<FailedDisplayUpdate>,
}

/// High-level entry point for querying and updating displays.
///
/// On Windows, logical and physical display operations are supported.
/// On Linux, physical display operations are supported and logical display
/// operations are supported on wlroots-based Wayland compositors.
pub struct DisplayManager;

impl DisplayManager {
    /// Queries the current display state.
    #[tracing::instrument(ret, level = "trace")]
    pub fn query() -> Result<Vec<Display>, DisplayQueryError> {
        let logical_displays: Vec<_> = LogicalDisplayManager::query()?.into_iter().collect();
        let mut physical_displays = PhysicalDisplayManager::query()?;

        Ok(logical_displays
            .into_iter()
            .map(|logical| Display {
                physical: take_matching_physical_display(&logical, &mut physical_displays),
                logical,
            })
            .collect())
    }

    #[tracing::instrument(ret, skip_all, level = "trace")]
    fn get_inner(ids: Vec<DisplayIdentifier>) -> Result<Vec<DisplayMatch>, DisplayQueryError> {
        let displays = Self::query()?;
        Ok(ids
            .into_iter()
            .flat_map(|requested_id| {
                matching_displays(&displays, &requested_id)
                    .map(|display| DisplayMatch {
                        requested_id: requested_id.clone(),
                        matched_id: display.id().outer,
                        display: display.clone(),
                    })
                    .collect::<Vec<_>>()
            })
            .collect())
    }

    /// Looks up displays matching the provided user-facing identifiers.
    pub fn get(ids: Vec<DisplayIdentifier>) -> Result<Vec<DisplayMatch>, DisplayQueryError> {
        Self::get_inner(ids)
    }

    /// Applies the requested display updates.
    ///
    /// When `validate` is `true`, backends may validate updates without applying
    /// them if the platform supports that behavior.
    #[tracing::instrument(ret, level = "trace")]
    pub fn apply(
        updates: Vec<DisplayUpdate>,
        validate: bool,
    ) -> Result<Vec<DisplayUpdateResult>, DisplayApplyError> {
        let matched_updates = matched_updates(updates)?;
        let mut results = Vec::with_capacity(matched_updates.len());

        for (requested_update, matched_ids) in matched_updates {
            let mut result = new_update_result(&requested_update);

            for matched_id in matched_ids {
                let matched_outer = matched_id.outer.clone();

                if let Some(logical_content) = requested_update.logical.clone() {
                    let logical_update = LogicalDisplayUpdate {
                        id: matched_id.clone(),
                        content: logical_content,
                    };

                    match apply_logical_update(logical_update, validate) {
                        Ok(true) => {}
                        Ok(false) => {
                            push_failed(
                                &mut result,
                                matched_outer.clone(),
                                "logical update was not applied",
                            );
                            continue;
                        }
                        Err(err) => {
                            push_failed(&mut result, matched_outer.clone(), err.to_string());
                            continue;
                        }
                    }
                }

                if validate || requested_update.physical.is_none() {
                    push_applied_once(&mut result, matched_outer);
                    continue;
                }

                let physical_update = PhysicalDisplayUpdate {
                    id: matched_id,
                    content: requested_update
                        .physical
                        .clone()
                        .expect("physical update presence checked above"),
                };

                match apply_physical_update(physical_update, validate) {
                    Ok(true) => push_applied_once(&mut result, matched_outer.clone()),
                    Ok(false) => push_failed(
                        &mut result,
                        matched_outer.clone(),
                        "physical update was not applied",
                    ),
                    Err(err) => push_failed(&mut result, matched_outer.clone(), err.to_string()),
                }
            }

            results.push(result);
        }

        Ok(results)
    }

    /// Applies the requested display updates without validation-only mode.
    pub fn update(
        updates: Vec<DisplayUpdate>,
    ) -> Result<Vec<DisplayUpdateResult>, DisplayApplyError> {
        Self::apply(updates, false)
    }

    /// Validates the requested display updates when supported by the platform backend.
    pub fn validate(
        updates: Vec<DisplayUpdate>,
    ) -> Result<Vec<DisplayUpdateResult>, DisplayApplyError> {
        Self::apply(updates, true)
    }
}

#[cfg(target_os = "windows")]
fn take_matching_physical_display(
    logical: &LogicalDisplay,
    remaining_physical: &mut Vec<PhysicalDisplay>,
) -> Option<PhysicalDisplay> {
    remaining_physical
        .iter()
        .position(|physical| logical.metadata.path.starts_with(&physical.metadata.path))
        .map(|index| remaining_physical.remove(index))
}

#[cfg(target_os = "linux")]
fn take_matching_physical_display(
    logical: &LogicalDisplay,
    remaining_physical: &mut Vec<PhysicalDisplay>,
) -> Option<PhysicalDisplay> {
    // Linux physical/logical correlation is heuristic across separate backends,
    // so only accept uniquely identifying matches and prefer no match over a
    // potentially wrong association.
    if let Some(logical_connector) = logical_connector_name(logical) {
        if let Some(index) = unique_match_index(remaining_physical, |physical| {
            physical_connector_name(physical)
                .as_deref()
                .is_some_and(|physical_connector| physical_connector == logical_connector)
        }) {
            return Some(remaining_physical.remove(index));
        }
    }

    if let Some(index) = unique_match_index(remaining_physical, |physical| {
        logical
            .metadata
            .serial_number
            .as_deref()
            .filter(|serial_number| !serial_number.is_empty())
            .is_some_and(|serial_number| physical.metadata.serial_number == serial_number)
    }) {
        return Some(remaining_physical.remove(index));
    }

    let name_candidates = logical_name_candidates(logical);

    if let Some(index) = unique_match_index(remaining_physical, |physical| {
        name_candidates.iter().any(|candidate| {
            physical.metadata.name == *candidate
                || normalized_name(&physical.metadata.name) == normalized_name(candidate)
        })
    }) {
        return Some(remaining_physical.remove(index));
    }

    None
}

#[cfg(target_os = "linux")]
fn logical_connector_name(logical: &LogicalDisplay) -> Option<&str> {
    logical
        .metadata
        .path
        .rsplit(':')
        .find(|segment| is_connector_name(segment))
}

#[cfg(target_os = "linux")]
fn physical_connector_name(physical: &PhysicalDisplay) -> Option<String> {
    physical
        .metadata
        .path
        .split('/')
        .find_map(drm_connector_name)
        .map(ToString::to_string)
}

#[cfg(target_os = "linux")]
fn drm_connector_name(path_component: &str) -> Option<&str> {
    let remainder = path_component.strip_prefix("card")?;
    let separator_index = remainder.find('-')?;
    let (card_index, connector) = remainder.split_at(separator_index);
    if card_index.is_empty()
        || !card_index
            .chars()
            .all(|character| character.is_ascii_digit())
    {
        return None;
    }

    let connector = connector.trim_start_matches('-');
    is_connector_name(connector).then_some(connector)
}

#[cfg(target_os = "linux")]
fn is_connector_name(value: &str) -> bool {
    let Some((prefix, suffix)) = value.rsplit_once('-') else {
        return false;
    };

    !prefix.is_empty()
        && !suffix.is_empty()
        && suffix.chars().all(|character| character.is_ascii_digit())
}

#[cfg(target_os = "linux")]
fn unique_match_index(
    displays: &[PhysicalDisplay],
    mut predicate: impl FnMut(&PhysicalDisplay) -> bool,
) -> Option<usize> {
    let mut matches = displays
        .iter()
        .enumerate()
        .filter_map(|(index, display)| predicate(display).then_some(index));
    let first = matches.next()?;
    matches.next().is_none().then_some(first)
}

#[cfg(target_os = "linux")]
fn logical_name_candidates(logical: &LogicalDisplay) -> Vec<String> {
    let mut candidates = Vec::new();

    push_unique_candidate(&mut candidates, logical.metadata.name.clone());

    if let (Some(manufacturer), Some(model)) = (
        logical.metadata.manufacturer.as_deref().map(str::trim),
        logical.metadata.model.as_deref().map(str::trim),
    ) {
        if !manufacturer.is_empty() && !model.is_empty() {
            push_unique_candidate(&mut candidates, format!("{manufacturer} {model}"));
        }
    }

    if let Some(model) = logical
        .metadata
        .model
        .as_deref()
        .map(str::trim)
        .filter(|model| !model.is_empty())
    {
        push_unique_candidate(&mut candidates, model.to_string());
    }

    candidates
}

#[cfg(target_os = "linux")]
fn push_unique_candidate(candidates: &mut Vec<String>, candidate: String) {
    if candidate.is_empty() {
        return;
    }

    let normalized_candidate = normalized_name(&candidate);
    if candidates
        .iter()
        .any(|existing| normalized_name(existing) == normalized_candidate)
    {
        return;
    }

    candidates.push(candidate);
}

#[cfg(target_os = "linux")]
fn normalized_name(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

#[cfg(target_os = "windows")]
fn apply_logical_update(
    update: LogicalDisplayUpdate,
    validate: bool,
) -> Result<bool, LogicalDisplayApplyError> {
    Ok(LogicalDisplayManager::apply(vec![update], validate)?.is_empty())
}

#[cfg(target_os = "linux")]
fn apply_logical_update(
    update: LogicalDisplayUpdate,
    validate: bool,
) -> Result<bool, LogicalDisplayApplyError> {
    Ok(LogicalDisplayManager::apply(vec![update], validate)?.is_empty())
}

#[cfg(target_os = "windows")]
fn apply_physical_update(
    update: PhysicalDisplayUpdate,
    _validate: bool,
) -> Result<bool, PhysicalDisplayApplyError> {
    Ok(PhysicalDisplayManager::apply(vec![update])?.is_empty())
}

#[cfg(target_os = "linux")]
fn apply_physical_update(
    update: PhysicalDisplayUpdate,
    validate: bool,
) -> Result<bool, PhysicalDisplayApplyError> {
    Ok(PhysicalDisplayManager::apply(vec![update], validate)?.is_empty())
}

fn matched_updates(
    updates: Vec<DisplayUpdate>,
) -> Result<Vec<(DisplayUpdate, Vec<DisplayIdentifierInner>)>, DisplayQueryError> {
    let displays = DisplayManager::query()?;
    Ok(updates
        .into_iter()
        .map(|update| {
            let matched_ids = matching_displays(&displays, &update.id)
                .map(Display::id)
                .collect();
            (update, matched_ids)
        })
        .collect())
}

fn matching_displays<'a>(
    displays: &'a [Display],
    requested_id: &'a DisplayIdentifier,
) -> impl Iterator<Item = &'a Display> {
    displays
        .iter()
        .filter(move |display| requested_id.is_subset(&display.id().outer))
}

fn new_update_result(requested_update: &DisplayUpdate) -> DisplayUpdateResult {
    DisplayUpdateResult {
        requested_update: requested_update.clone(),
        applied: Vec::new(),
        failed: Vec::new(),
    }
}

fn push_applied_once(result: &mut DisplayUpdateResult, matched_id: DisplayIdentifier) {
    if !result.applied.contains(&matched_id) {
        result.applied.push(matched_id);
    }
}

fn push_failed(
    result: &mut DisplayUpdateResult,
    matched_id: DisplayIdentifier,
    message: impl Into<String>,
) {
    result.failed.push(FailedDisplayUpdate {
        matched_id,
        message: message.into(),
    });
}

pub struct QueryError {}
pub struct ValidateUpdateError {}
pub struct CreationError {}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    #[test]
    fn linux_matches_backlight_by_connector_name() {
        let logical = LogicalDisplay {
            metadata: displays_logical_types::LogicalDisplayMetadata {
                name: "LG Display 0x07C6".to_string(),
                path: "wayland:wlr:eDP-1".to_string(),
                manufacturer: Some("LG Display".to_string()),
                model: Some("0x07C6".to_string()),
                serial_number: Some("unknown".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let expected = PhysicalDisplay {
            metadata: displays_physical_types::PhysicalDisplayMetadata {
                name: "intel_backlight".to_string(),
                path: "/sys/devices/pci0000:00/0000:00:02.0/drm/card0/card0-eDP-1/intel_backlight"
                    .to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let mut physical = vec![expected.clone()];

        let matched = take_matching_physical_display(&logical, &mut physical);

        assert_eq!(matched, Some(expected));
        assert!(physical.is_empty());
    }

    #[test]
    fn linux_does_not_match_ambiguous_connectors() {
        let logical = LogicalDisplay {
            metadata: displays_logical_types::LogicalDisplayMetadata {
                name: "LG Display 0x07C6".to_string(),
                path: "wayland:wlr:eDP-1".to_string(),
                manufacturer: Some("LG Display".to_string()),
                model: Some("0x07C6".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let mut physical = vec![
            PhysicalDisplay {
                metadata: displays_physical_types::PhysicalDisplayMetadata {
                    name: "intel_backlight".to_string(),
                    path: "/sys/devices/pci0000:00/0000:00:02.0/drm/card0/card0-eDP-1/intel_backlight"
                        .to_string(),
                    ..Default::default()
                },
                ..Default::default()
            },
            PhysicalDisplay {
                metadata: displays_physical_types::PhysicalDisplayMetadata {
                    name: "acpi_video0".to_string(),
                    path: "/sys/devices/LNXSYSTM:00/LNXSYBUS:00/ACPI0008:00/backlight/acpi_video0/drm/card0/card0-eDP-1"
                        .to_string(),
                    ..Default::default()
                },
                ..Default::default()
            },
        ];

        let matched = take_matching_physical_display(&logical, &mut physical);

        assert_eq!(matched, None);
        assert_eq!(physical.len(), 2);
    }

    #[test]
    fn linux_falls_back_to_name_matching_when_connector_is_unavailable() {
        let logical = LogicalDisplay {
            metadata: displays_logical_types::LogicalDisplayMetadata {
                name: "LG Display 0x07C6".to_string(),
                path: "wayland:wlr:unknown".to_string(),
                manufacturer: Some("LG Display".to_string()),
                model: Some("0x07C6".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let expected = PhysicalDisplay {
            metadata: displays_physical_types::PhysicalDisplayMetadata {
                name: "LG Display 0x07C6".to_string(),
                path: "/dev/i2c-7".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let mut physical = vec![expected.clone()];

        let matched = take_matching_physical_display(&logical, &mut physical);

        assert_eq!(matched, Some(expected));
        assert!(physical.is_empty());
    }

    #[test]
    fn linux_extracts_connector_from_non_wlr_paths() {
        let logical = LogicalDisplay {
            metadata: displays_logical_types::LogicalDisplayMetadata {
                path: "wayland:gnome:eDP-1".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(logical_connector_name(&logical), Some("eDP-1"));
    }
}
