use super::{types::*, Backend};
use crate::error::{error_message, DisplaysAstalError};

/// Adapter around the real `displays` crate.
///
/// This backend is selected by default. It preserves the fake backend's public
/// GI shape by converting between the domain types from `displays` and the
/// normalized `Display*Data` structs used internally by `displays-astal`.
pub struct RealBackend;

impl Backend for RealBackend {
    fn query(&self) -> Result<Vec<DisplayData>, glib::Error> {
        displays::manager::DisplayManager::query()
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(map_error)
    }

    fn get(&self, ids: Vec<DisplayIdentifierData>) -> Result<Vec<DisplayMatchData>, glib::Error> {
        let ids = ids
            .into_iter()
            .map(displays::types::DisplayIdentifier::from)
            .collect::<Vec<_>>();

        Ok(displays::manager::DisplayManager::get(ids)
            .map_err(map_error)?
            .into_iter()
            .map(|display_match| DisplayMatchData {
                requested_id: display_match.requested_id.into(),
                matched_id: display_match.matched_id.into(),
                display: display_match.display.into(),
            })
            .collect())
    }

    fn apply(
        &self,
        updates: Vec<DisplayUpdateData>,
        validate: bool,
    ) -> Result<Vec<DisplayUpdateResultData>, glib::Error> {
        displays::manager::DisplayManager::apply(
            updates.into_iter().map(Into::into).collect(),
            validate,
        )
        .map(|items| {
            items
                .into_iter()
                .map(|result| DisplayUpdateResultData {
                    requested_update: result.requested_update.into(),
                    applied: result.applied.into_iter().map(Into::into).collect(),
                    failed: result
                        .failed
                        .into_iter()
                        .map(|failed| FailedDisplayUpdateData {
                            matched_id: failed.matched_id.into(),
                            message: failed.message,
                        })
                        .collect(),
                })
                .collect()
        })
        .map_err(map_error)
    }
}

fn map_error(err: impl std::fmt::Display) -> glib::Error {
    error_message(DisplaysAstalError::Failed, &err.to_string())
}
