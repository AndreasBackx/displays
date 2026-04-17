/// Requested changes to physical monitor state.
#[derive(Debug, Clone, Default)]
pub struct PhysicalDisplayUpdateContent {
    /// Requested brightness percentage in the inclusive range `0..=100`.
    pub brightness: Option<u32>,
}
