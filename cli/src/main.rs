use std::{
    io::Cursor,
    mem,
    ptr::{self, null_mut},
};

// #![windows_subsystem = "windows"]
use anyhow::Result;
use clap::Parser;
// use displays_lib::displays::Displays;
// use displays_lib::state::State;
use edid_rs::{Reader, EDID};
use windows::{
    core::PCSTR,
    Win32::{
        Devices::Display::{
            GetNumberOfPhysicalMonitorsFromHMONITOR, GetPhysicalMonitorsFromHMONITOR,
            SetMonitorBrightness, PHYSICAL_MONITOR,
        },
        Foundation::{BOOL, LPARAM, RECT},
        Graphics::Gdi::{
            EnumDisplayDevicesA, EnumDisplayMonitors, GetMonitorInfoA, GetMonitorInfoW,
            DISPLAY_DEVICEA, HDC, HMONITOR, MONITORINFO, MONITORINFOEXA, MONITORINFOEXW,
        },
    },
};
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {}

#[derive(Debug)]
struct Device {
    name: String,
    id: String,
    string: String,
    key: String,
}

fn convert_utf8<const COUNT: usize>(chars: [i8; COUNT]) -> String {
    String::from_utf8(
        chars
            .into_iter()
            .take_while(|character| *character != 0)
            .map(|char| char as u8)
            .collect::<Vec<_>>(),
    )
    .unwrap_or_default()
}

fn convert_utf16<const COUNT: usize>(chars: [u16; COUNT]) -> String {
    String::from_utf16(
        &chars
            .into_iter()
            .take_while(|character| *character != 0)
            // .map(|char| char as u8)
            .collect::<Vec<_>>(),
    )
    .unwrap_or_default()
}

fn enumerate_display_devices() -> Result<Vec<Device>> {
    let mut devices = Vec::new();
    let mut device = DISPLAY_DEVICEA::default();
    device.cb = std::mem::size_of::<DISPLAY_DEVICEA>() as u32;

    let mut device_index = 0;
    while unsafe { EnumDisplayDevicesA(PCSTR(null_mut()), device_index, &mut device, 0) }.as_bool()
    {
        devices.push(device.clone());
        device_index += 1;
    }

    devices
        .into_iter()
        .map(|device| {
            Ok(Device {
                name: convert_utf8(device.DeviceName),
                id: convert_utf8(device.DeviceID),
                string: convert_utf8(device.DeviceString),
                key: convert_utf8(device.DeviceKey),
            })
        })
        .collect::<Result<_>>()
}

pub fn enumerate_monitors() -> windows::core::Result<Vec<HMONITOR>> {
    unsafe extern "system" fn callback(
        monitor: HMONITOR,
        _hdc_monitor: HDC,
        _lprc: *mut RECT,
        userdata: LPARAM,
    ) -> BOOL {
        let monitors: &mut Vec<HMONITOR> = &mut *(userdata.0 as *mut Vec<HMONITOR>);
        monitors.push(monitor);
        BOOL::from(true)
    }

    let mut monitors = Vec::<HMONITOR>::new();
    let userdata = LPARAM(ptr::addr_of_mut!(monitors) as _);
    unsafe { EnumDisplayMonitors(None, None, Some(callback), userdata) }.ok()?;
    Ok(monitors)
}

#[derive(Debug)]
struct MonitorInfo {
    handle: HMONITOR,
    name: String,
    primary: bool,
    rect: RECT,
    work_rect: RECT,
}

fn get_detailed_monitor_info() -> Result<Vec<MonitorInfo>> {
    let mut monitor_infos = Vec::new();
    let monitors = enumerate_monitors()?;

    for monitor in monitors {
        let mut monitor_info: MONITORINFOEXW = unsafe { std::mem::zeroed() };

        monitor_info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

        let success = unsafe {
            GetMonitorInfoW(monitor, ptr::addr_of_mut!(monitor_info) as *mut MONITORINFO)
        };

        if success.as_bool() {
            monitor_infos.push(MonitorInfo {
                handle: monitor,
                name: String::from_utf16_lossy(&monitor_info.szDevice),
                primary: monitor_info.monitorInfo.dwFlags & 1 != 0, // MONITORINFOF_PRIMARY
                rect: monitor_info.monitorInfo.rcMonitor,
                work_rect: monitor_info.monitorInfo.rcWork,
            });
        }
    }

    Ok(monitor_infos)
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        // .event_format(
        //     tracing_subscriber::fmt::format()
        //         .with_file(true)
        //         .with_line_number(true),
        // )
        .init();

    // let displays = Displays::try_new()?;
    // let d = displays.query()?;
    // println!("{d:#?}");

    // return Ok(());

    // Open the HKEY_LOCAL_MACHINE root key.
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    // Open the DISPLAY registry key under Enum.
    let display_key_path = r"SYSTEM\CurrentControlSet\Enum\DISPLAY";
    let display_key = hklm.open_subkey(display_key_path)?;

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
                let edid = EDID::parse(reader);
                println!("{:#?}", edid);
            } else {
                println!("No EDID found for device {}\\{}", device_id, instance_id);
            }
        }
    }

    // return Ok(());
    // let state = State::try_new()?;

    // println!("{state}");

    // TODO on how to map them:
    // The code below convert_utf8(monitor_info.szDevice)
    // This gives "\\.\DISPLAY2"
    // The index there refers to the source that is used on the adapter
    // +---------+--------+----------+----------+------------+----------+------------------------------+--------+----------+----------+
    // | enabled | source | adapter  | mode idx | size       | position | pixel format                 | target | adapter  | mode idx |
    // +=========+========+==========+==========+============+==========+==============================+========+==========+==========+
    // | true    | 1      | 82617, 0 | Some(3)  | 3840, 2160 | 0, 0     | DISPLAYCONFIG_PIXELFORMAT(4) | 268    | 82617, 0 | Some(3)  |
    // +---------+--------+----------+----------+------------+----------+------------------------------+--------+----------+----------+
    // It's referring to source 1, not 2 here because "\\.\DISPLAY${I}" starts
    // counting from 1.
    // This is how we map it.
    // Then, once it's mapped. We can technically get the EDID from the registry at:
    // Computer\HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Enum\DISPLAY\LEN66F9\7&289ec95a&0&UID256
    // Though, that might simply make it easier to use and not actually required.
    // Though if it would be xplatform, this would be required.

    let devices = enumerate_display_devices();
    println!("{devices:#?}");

    let monitors = enumerate_monitors()?;
    println!("{monitors:#?}");

    let monitor_info = get_detailed_monitor_info()?;
    println!("{monitor_info:#?}");

    for monitor in monitors {
        // Prepare to get information about the monitor
        let mut monitor_info = MONITORINFOEXA {
            monitorInfo: MONITORINFO {
                cbSize: mem::size_of::<MONITORINFOEXA>() as u32,
                ..Default::default()
            },
            ..Default::default()
        };

        let monitor_info_base = &mut monitor_info as *mut MONITORINFOEXA as *mut MONITORINFO;

        // Get the monitor info for this monitor
        unsafe { GetMonitorInfoA(monitor, monitor_info_base) }.ok()?;

        eprintln!("szDevice: {}", convert_utf8(monitor_info.szDevice));
        // eprintln!("{monitor_info:#?}");

        //

        let mut monitor_count = 1;

        unsafe { GetNumberOfPhysicalMonitorsFromHMONITOR(monitor, &mut monitor_count) }?;

        let mut physical_monitors = vec![PHYSICAL_MONITOR::default(); monitor_count as usize];
        if let Ok(_) = unsafe {
            GetPhysicalMonitorsFromHMONITOR(
                monitor,
                // &mut monitor_count,
                physical_monitors.as_mut_slice(),
            )
        } {
            // For each physical monitor, set brightness to 100%
            for physical_monitor in &physical_monitors {
                println!(
                    "szPhysicalMonitorDescription: {}",
                    convert_utf16(physical_monitor.szPhysicalMonitorDescription)
                );
                // unsafe {
                //     let _ = SetMonitorBrightness(physical_monitor.hPhysicalMonitor, 100);
                // }
            }
        }
    }

    // let cli = Cli::parse();

    // let monitors = Monitor::enumerate().unwrap();
    // for mut m in monitors {
    //     print!("{:?}: ", m);
    //     println!("{:?}", m.get_timing_report());
    //     m.capabilities_string()

    //     // print!(
    //     //     "{:?}: ",
    //     //     m.winapi_get_vcp_feature_and_vcp_feature_reply(code)
    //     // );
    // }

    // for mut display in Display::enumerate() {
    //     if let Err(err) = display.update_capabilities() {
    //         error!("{err:#?}");
    //         continue;
    //     }
    //     println!("{:#?}", display.info);
    //     // println!(
    //     //     "{:?} {}: {:?} {:?}",
    //     //     display.info.backend,
    //     //     display.info.id,
    //     //     display.info.manufacturer_id,
    //     //     display.info.model_name
    //     // );
    //     if let Some(feature) = display.info.mccs_database.get(0xdf) {
    //         let value = display.handle.get_vcp_feature(feature.code).unwrap();
    //         println!("{}: {:?}", feature.name.as_ref().unwrap(), value);
    //     }
    // }

    // let setups = DisplayConfigs {
    //     displays: vec![
    //         DisplayConfig {
    //             name: "LG TV".to_owned(),
    //             path: None,
    //             is_enabled: cli.tv,
    //         },
    //         DisplayConfig {
    //             name: "AW3225QF".to_owned(),
    //             path: None,
    //             is_enabled: !cli.tv,
    //         },
    //         DisplayConfig {
    //             name: "Y32p-30".to_owned(),
    //             path: Some(
    //                 r"\\?\DISPLAY#LEN66F9#7&289ec95a&0&UID264#{e6f07b5f-ee97-4a90-b076-33f57bf4eaa7}".to_owned(),
    //             ),
    //             is_enabled: !cli.tv,
    //         },
    //         DisplayConfig {
    //             name: "Y32p-30".to_owned(),
    //             path: Some(
    //                 r"\\?\DISPLAY#LEN66F9#7&289ec95a&0&UID260#{e6f07b5f-ee97-4a90-b076-33f57bf4eaa7}".to_owned(),
    //             ),
    //             is_enabled: !cli.tv,
    //         },
    //     ],
    // };
    Ok(())
}
