pub mod windows;

// use std::{collections::BTreeSet, string::FromUtf16Error};

// use anyhow::bail;
// use tracing::info;
// use windows::Win32::{
//     Devices::Display::{
//         DisplayConfigGetDeviceInfo, GetDisplayConfigBufferSizes, QueryDisplayConfig,
//         SetDisplayConfig, DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME, DISPLAYCONFIG_MODE_INFO,
//         DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_TARGET_DEVICE_NAME, QDC_ALL_PATHS,
//         SDC_ALLOW_PATH_ORDER_CHANGES, SDC_APPLY, SDC_TOPOLOGY_SUPPLIED, SDC_VALIDATE,
//     },
//     Foundation::ERROR_SUCCESS,
//     Graphics::Gdi::{DISPLAYCONFIG_PATH_ACTIVE, DISPLAYCONFIG_PATH_MODE_IDX_INVALID},
// };
// pub trait LogicalDisplay {
//     fn is_enabled(&self) -> bool;
// }
// impl LogicalDisplay for LogicalDisplayWindows {
//     fn is_enabled(&self) -> bool {
//         self.is_enabled
//     }
// }
