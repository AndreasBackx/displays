use displays_types::DisplayIdentifier;

use crate::display::{Display, DisplayUpdate};

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
