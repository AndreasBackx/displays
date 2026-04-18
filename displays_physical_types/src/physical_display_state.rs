use displays_types::Brightness;

/// The current physical monitor state.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalDisplayState {
    /// Current brightness percentage when supported.
    pub brightness: Option<Brightness>,
}
