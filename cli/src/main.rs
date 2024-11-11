use std::{
    mem,
    ptr::{self, null_mut},
};

// #![windows_subsystem = "windows"]
use anyhow::Result;
use clap::Parser;
use displays_lib::state::State;
use windows::{
    core::PCSTR,
    Win32::{
        Devices::Display::{
            GetNumberOfPhysicalMonitorsFromHMONITOR, GetPhysicalMonitorsFromHMONITOR,
            SetMonitorBrightness, PHYSICAL_MONITOR,
        },
        Foundation::{BOOL, LPARAM, RECT},
        Graphics::Gdi::{
            EnumDisplayDevicesA, EnumDisplayMonitors, GetMonitorInfoA, DISPLAY_DEVICEA, HDC,
            HMONITOR, MONITORINFO, MONITORINFOEXA,
        },
    },
};

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

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        // .event_format(
        //     tracing_subscriber::fmt::format()
        //         .with_file(true)
        //         .with_line_number(true),
        // )
        .init();

    let state = State::try_new()?;

    println!("{state}");

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

    // let devices = enumerate_display_devices();
    // println!("{devices:#?}");

    let monitors = enumerate_monitors()?;
    println!("{monitors:#?}");

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

        eprintln!("{}", convert_utf8(monitor_info.szDevice));
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
                    "{}",
                    convert_utf16(physical_monitor.szPhysicalMonitorDescription)
                );
                unsafe {
                    let _ = SetMonitorBrightness(physical_monitor.hPhysicalMonitor, 100);
                }
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
