/// A user-facing identifier used to match one or more displays.
///
/// Matching is subset-based: any field set on this identifier must match the
/// corresponding field on the target display.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DisplayIdentifier {
    /// Human-readable display name when available.
    pub name: Option<String>,
    /// Physical serial number when available.
    pub serial_number: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DisplayIdentifierInner {
    pub outer: DisplayIdentifier,
    pub(crate) path: Option<String>,
    // pub(crate) source_id: Option<u32>,
    pub(crate) gdi_device_id: Option<u32>,
}

impl DisplayIdentifier {
    /// Returns `true` when this identifier is a subset of `other`.
    #[tracing::instrument(ret, level = "debug")]
    pub fn is_subset(&self, other: &DisplayIdentifier) -> bool {
        if let Some(ref name) = self.name {
            if let Some(ref other_name) = other.name {
                if name != other_name {
                    return false;
                }
            }
        }

        if let Some(ref serial_number) = self.serial_number {
            if let Some(ref other_serial_number) = other.serial_number {
                if serial_number != other_serial_number {
                    return false;
                }
            }
        }
        true
    }
}
