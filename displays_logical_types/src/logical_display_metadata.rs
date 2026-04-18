/// Stable metadata describing a logical display.
#[derive(Debug, Clone, Default, PartialOrd, Ord, PartialEq, Eq)]
pub struct LogicalDisplayMetadata {
    /// Human-readable display name.
    pub name: String,
    /// Platform-specific display path.
    pub path: String,
    /// Display manufacturer when available.
    pub manufacturer: Option<String>,
    /// Display model when available.
    pub model: Option<String>,
    /// Display serial number when available.
    pub serial_number: Option<String>,
    /// Windows GDI device id when available.
    #[cfg(target_os = "windows")]
    pub gdi_device_id: Option<u32>,
}
