use std::collections::BTreeSet;

use super::{types::*, Backend};
use crate::error::{error_message, AstalDisplaysError};

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
        let requested_ids = ids.clone();
        let ids = ids
            .into_iter()
            .map(displays::display_identifier::DisplayIdentifier::from)
            .collect::<BTreeSet<_>>();

        let display_by_id = displays::manager::DisplayManager::get(ids).map_err(map_error)?;

        Ok(requested_ids
            .into_iter()
            .filter_map(|requested_id| {
                display_by_id
                    .get(&displays::display_identifier::DisplayIdentifier::from(
                        requested_id.clone(),
                    ))
                    .map(|display| DisplayMatchData {
                        requested_id,
                        display: display.into(),
                    })
            })
            .collect())
    }

    fn apply(
        &self,
        updates: Vec<DisplayUpdateData>,
        validate: bool,
    ) -> Result<Vec<DisplayUpdateData>, glib::Error> {
        displays::manager::DisplayManager::apply(
            updates.into_iter().map(Into::into).collect(),
            validate,
        )
        .map(|items| items.into_iter().map(Into::into).collect())
        .map_err(map_error)
    }
}

fn map_error(err: impl std::fmt::Display) -> glib::Error {
    error_message(AstalDisplaysError::Failed, &err.to_string())
}
