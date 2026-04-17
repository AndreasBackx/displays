use std::collections::BTreeSet;

use displays_logical_types::{LogicalDisplay, LogicalDisplayUpdate};

use crate::{
    error::{ApplyError, QueryError},
    wayland,
};

pub struct LogicalDisplayManager;

impl LogicalDisplayManager {
    #[tracing::instrument(ret, level = "trace")]
    pub fn query() -> Result<BTreeSet<LogicalDisplay>, QueryError> {
        wayland::query()
    }

    #[tracing::instrument(ret, skip_all, level = "trace")]
    pub fn apply(
        updates: Vec<LogicalDisplayUpdate>,
        validate: bool,
    ) -> Result<Vec<LogicalDisplayUpdate>, ApplyError> {
        wayland::apply(updates, validate)
    }
}
