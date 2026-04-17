/// Stable metadata describing a logical display.
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct LogicalDisplayMetadata {
    /// Human-readable display name.
    pub name: String,
    /// Platform-specific display path.
    pub path: String,
    /// Windows GDI device id when available.
    #[cfg(target_os = "windows")]
    pub gdi_device_id: Option<u32>,
}
