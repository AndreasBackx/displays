#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DisplayIdentifier {
    pub name: Option<String>,
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
