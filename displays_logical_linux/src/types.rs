use displays_logical_types::LogicalDisplay;
use displays_types::DisplayIdentifierInner;

pub(crate) fn logical_display_matches(
    display: &LogicalDisplay,
    id: &DisplayIdentifierInner,
) -> bool {
    id.outer
        .name
        .as_ref()
        .is_none_or(|name| display.metadata.name.starts_with(name))
        && id
            .path
            .as_ref()
            .is_none_or(|path| display.metadata.path.starts_with(path))
}
