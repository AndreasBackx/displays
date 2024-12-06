use std::io::Cursor;

use anyhow::Context;
use edid_rs::{Reader, EDID};
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

use crate::logical_display;

#[derive(Debug, Default)]
pub struct PhysicalDisplay {
    brightness: Option<Brightness>,
}

impl PhysicalDisplay {
    fn new() -> Self {
        let brightness = Brightness::new();
        Self {
            brightness: Some(brightness),
        }
    }
}

pub struct PhysicalDisplayWindows {
    /// E.g: "\\.\DISPLAY1"
    path: String,
    /// E.g: "Lenovo Y32p-30"
    name: String,
    serial_number: String,
}

#[derive(Clone)]
pub struct PhysicalDisplayManagerWindows {}

impl PhysicalDisplayManagerWindows {
    pub fn try_new() -> anyhow::Result<Self> {
        Ok(Self {})
    }

    pub fn query(&self) -> anyhow::Result<Vec<PhysicalDisplayWindows>> {
        // Open the HKEY_LOCAL_MACHINE root key.
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

        // Open the DISPLAY registry key under Enum.
        let display_key_path = r"SYSTEM\CurrentControlSet\Enum\DISPLAY";
        let display_key = hklm.open_subkey(display_key_path)?;

        let mut physical_displays = vec![];

        // Iterate over each subkey in DISPLAY (each display device).
        for device_id in display_key.enum_keys() {
            let device_id = device_id?;
            let device_key = display_key.open_subkey(&device_id)?;

            // Each device may have multiple subkeys, so iterate over them.
            for instance_id in device_key.enum_keys() {
                let instance_id = instance_id?;
                let device_params_key = format!("{instance_id}\\Device Parameters",);
                let instance_key = device_key.open_subkey(&device_params_key)?;

                // Check if the EDID value exists within this instance key.
                if let Ok(edid_data) = instance_key.get_raw_value("EDID") {
                    println!("Found EDID for device {}\\{}:", device_id, instance_id);

                    let mut cursor = Cursor::new(edid_data.bytes);
                    let reader = &mut Reader::new(&mut cursor);
                    let edid = EDID::parse(reader).map_err(|err| anyhow::anyhow!(err))?;
                    println!("{:#?}", edid);
                    physical_displays.push(edid.try_into()?);
                } else {
                    println!("No EDID found for device {}\\{}", device_id, instance_id);
                }
            }
        }

        Ok(physical_displays)

        // display_key.enum_values()

        // let a = display_key
        //     .enum_keys()
        //     .into_iter()
        //     // device_id example: SAM7476
        //     .filter_map(|device_id| device_id.ok())
        //     .filter_map(|device_id| {
        //         display_key
        //             .open_subkey(&device_id)
        //             .ok()
        //             .map(|device_key| (device_id, device_key))
        //     })
        //     // instance_id example: 7&289ec95a&0&UID256
        //     .flat_map(|(device_id, device_key)| {
        //         //
        //         device_key
        //             .enum_keys()
        //             .filter_map(Result::ok)
        //             .filter_map(|instance_id| {
        //                 let device_params_key = format!("{instance_id}\\Device Parameters",);
        //                 device_key
        //                     .open_subkey(&device_params_key)
        //                     .map_err(|err| anyhow::anyhow!(err))
        //                     .and_then(|instance_key| {
        //                         instance_key
        //                             .get_raw_value("EDID")
        //                             .map_err(|err| anyhow::anyhow!(err))
        //                     })
        //                     .and_then(|edid_reg_value| {
        //                         let mut cursor = Cursor::new(edid_reg_value.bytes);
        //                         let reader = &mut Reader::new(&mut cursor);
        //                         EDID::parse(reader).map_err(|err| anyhow::anyhow!(err))
        //                         // .map_err(|err| anyhow::format_err!(err))
        //                     })
        //                     .ok()
        //             })
        //     });
        // .flat_map(|device_id| {
        //     let device_id = device_id?;
        //     let device_key = display_key.open_subkey(&device_id)?;
        // });
    }
}

impl TryFrom<EDID> for PhysicalDisplayWindows {
    type Error = anyhow::Error;
    fn try_from(value: EDID) -> Result<Self, Self::Error> {
        let name = value
            .descriptors
            .0
            .iter()
            .filter_map(|descriptor| match descriptor {
                edid_rs::MonitorDescriptor::MonitorName(name) => Some(name),
                _ => None,
            })
            .nth(0)
            .cloned()
            .context("no monitor name found")?;
        let serial_number = value
            .descriptors
            .0
            .iter()
            .filter_map(|descriptor| match descriptor {
                edid_rs::MonitorDescriptor::SerialNumber(serial_number) => Some(serial_number),
                _ => None,
            })
            .nth(0)
            .cloned()
            .context("no serial number found")?;
        Ok(Self {
            name,
            serial_number,
        })
    }
}

#[derive(Debug)]
pub struct Brightness {}

impl Brightness {
    fn new() -> Self {
        Self {}
    }
    fn set(&self, percentage: i8) {}
}
