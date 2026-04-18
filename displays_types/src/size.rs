/// A 2D size used for display dimensions.
#[derive(Debug, Clone, Default, PartialOrd, Ord, PartialEq, Eq)]
pub struct Size {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}
