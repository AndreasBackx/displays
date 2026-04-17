use super::{types::*, Backend};
use crate::enums::Orientation;

/// Deterministic backend used only when the `faked` feature is enabled.
///
/// This keeps the GI-facing API stable while making it safe to iterate on the
/// typelib and smoke tests without touching actual hardware.
pub struct FakeBackend;

impl Backend for FakeBackend {
    fn query(&self) -> Result<Vec<DisplayData>, glib::Error> {
        Ok(fake_displays())
    }

    fn get(&self, ids: Vec<DisplayIdentifierData>) -> Result<Vec<DisplayMatchData>, glib::Error> {
        let displays = fake_displays();
        Ok(ids
            .into_iter()
            .flat_map(|requested_id| {
                matching_displays(&displays, &requested_id)
                    .cloned()
                    .map(|display| DisplayMatchData {
                        requested_id: requested_id.clone(),
                        matched_id: display.id.clone(),
                        display,
                    })
                    .collect::<Vec<_>>()
            })
            .collect())
    }

    fn apply(
        &self,
        updates: Vec<DisplayUpdateData>,
        _validate: bool,
    ) -> Result<Vec<DisplayUpdateResultData>, glib::Error> {
        let displays = fake_displays();
        Ok(updates
            .into_iter()
            .map(|update| {
                let applied = matching_displays(&displays, &update.id)
                    .map(|display| display.id.clone())
                    .collect();

                DisplayUpdateResultData {
                    requested_update: update,
                    applied,
                    failed: Vec::new(),
                }
            })
            .collect())
    }
}

fn matching_displays<'a>(
    displays: &'a [DisplayData],
    requested_id: &'a DisplayIdentifierData,
) -> impl Iterator<Item = &'a DisplayData> {
    displays
        .iter()
        .filter(move |display| requested_id.is_subset_of(&display.id))
}

fn fake_displays() -> Vec<DisplayData> {
    vec![
        DisplayData {
            id: DisplayIdentifierData {
                name: Some("Dell U2720Q".to_string()),
                serial_number: Some("DELLA1".to_string()),
            },
            logical: LogicalDisplayData {
                is_enabled: true,
                orientation: Orientation::Landscape,
                width: Some(3840),
                height: Some(2160),
                position: Some(PointData { x: 0, y: 0 }),
            },
            physical: Some(PhysicalDisplayData {
                brightness: 62,
                scale_factor: 150,
            }),
        },
        DisplayData {
            id: DisplayIdentifierData {
                name: Some("LG UltraFine".to_string()),
                serial_number: Some("LGFINE2".to_string()),
            },
            logical: LogicalDisplayData {
                is_enabled: true,
                orientation: Orientation::Portrait,
                width: Some(1440),
                height: Some(2560),
                position: Some(PointData { x: 3840, y: 0 }),
            },
            physical: Some(PhysicalDisplayData {
                brightness: 47,
                scale_factor: 100,
            }),
        },
        DisplayData {
            id: DisplayIdentifierData {
                name: Some("Virtual Display".to_string()),
                serial_number: None,
            },
            logical: LogicalDisplayData {
                is_enabled: false,
                orientation: Orientation::LandscapeFlipped,
                width: Some(1920),
                height: Some(1080),
                position: Some(PointData { x: -1920, y: 0 }),
            },
            physical: None,
        },
    ]
}
