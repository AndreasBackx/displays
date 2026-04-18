/// Stable metadata describing a physical monitor.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalDisplayMetadata {
    /// Platform-specific monitor path.
    pub path: String,
    /// Human-readable monitor name.
    pub name: String,
    /// Monitor manufacturer when available.
    pub manufacturer: Option<String>,
    /// Monitor model when available.
    pub model: Option<String>,
    /// Monitor serial number when available.
    pub serial_number: Option<String>,
}
