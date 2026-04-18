use displays_physical_types::PhysicalDisplayMetadata;
use displays_types::{DisplayIdentifier, DisplayIdentifierInner};
use edid_rs::EDID;

use crate::error::QueryError;

pub(crate) fn physical_display_metadata_from_edid(
    path: String,
    edid: EDID,
) -> Result<PhysicalDisplayMetadata, QueryError> {
    let manufacturer = Some(format!(
        "{}{}{}",
        edid.product.manufacturer_id.0,
        edid.product.manufacturer_id.1,
        edid.product.manufacturer_id.2
    ));
    let name = edid
        .descriptors
        .0
        .iter()
        .filter_map(|descriptor| match descriptor {
            edid_rs::MonitorDescriptor::MonitorName(name) => Some(name),
            _ => None,
        })
        .nth(0)
        .cloned()
        .ok_or_else(|| QueryError::EDIDInvalid {
            message: "no monitor name found".to_string(),
            key: path.clone(),
        })?;
    let model = Some(name.clone());
    let serial_number = edid
        .descriptors
        .0
        .iter()
        .filter_map(|descriptor| match descriptor {
            edid_rs::MonitorDescriptor::SerialNumber(serial_number) => Some(serial_number),
            _ => None,
        })
        .nth(0)
        .cloned()
        .or_else(|| (edid.product.serial_number != 0).then(|| edid.product.serial_number.to_string()));
    Ok(PhysicalDisplayMetadata {
        path,
        name,
        manufacturer,
        model,
        serial_number,
    })
}

pub(crate) fn physical_display_id(
    metadata: &PhysicalDisplayMetadata,
    gdi_device_id: Option<u32>,
) -> DisplayIdentifierInner {
    DisplayIdentifierInner {
        outer: DisplayIdentifier {
            name: Some(metadata.name.clone()),
            serial_number: metadata.serial_number.clone(),
        },
        path: Some(metadata.path.clone()),
        gdi_device_id,
    }
}
