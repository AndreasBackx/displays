use crate::Brightness;

/// The current physical monitor state.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalDisplayState {
    /// Current brightness percentage.
    pub brightness: Brightness,
    /// Current OS scale factor percentage.
    pub scale_factor: i32,
}
