use std::fmt::Debug;

use clap::Parser;

use crate::commands::Command;
use displays::{self as lib};

fn format_optional<T>(value: Option<T>) -> String
where
    T: Debug,
{
    value
        .map(|value| format!("{value:?}"))
        .unwrap_or_else(|| "unknown".to_string())
}

fn format_point(value: Option<&lib::types::Point>) -> String {
    value
        .map(|value| format!("{},{}", value.x, value.y))
        .unwrap_or_else(|| "unknown".to_string())
}

fn format_size(value: Option<&lib::types::Size>) -> String {
    value
        .map(|value| format!("{}x{}", value.width, value.height))
        .unwrap_or_else(|| "unknown".to_string())
}

fn format_scale_ratio(value: Option<u32>) -> String {
    value
        .map(|value| format!("{:.3}x", value as f64 / 1000.0))
        .unwrap_or_else(|| "unknown".to_string())
}

#[derive(Parser)]
pub struct QueryCommand {
    #[clap(long("enabled"))]
    is_enabled: Option<bool>,
}

impl Command for QueryCommand {
    fn run(&self) -> eyre::Result<()> {
        let displays = lib::manager::DisplayManager::query()?
            .into_iter()
            .collect::<Vec<_>>();

        let mut found_match = false;
        for display in displays {
            if let Some(is_enabled) = self.is_enabled {
                if is_enabled != display.logical.state.is_enabled {
                    continue;
                }
            }
            found_match = true;

            let metadata = display.metadata();
            let id = display.id();
            let display_name = id
                .outer
                .name
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            let serial_number = id
                .outer
                .serial_number
                .clone()
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "unknown".to_string());
            let physical = display.physical.as_ref();

            println!(
                concat!(
                    "Display: {}\n",
                    "  Identifier Name: {}\n",
                    "  Identifier Serial: {}\n",
                    "  Identifier Path: {}\n",
                    "  Logical:\n",
                    "    Name: {}\n",
                    "    Path: {}\n",
                    "    Manufacturer: {}\n",
                    "    Model: {}\n",
                    "    Serial: {}\n",
                    "    Enabled: {}\n",
                    "    Orientation: {:?}\n",
                    "    Logical Size: {}\n",
                    "    Mode Size: {}\n",
                    "    Scale: {}\n",
                    "    Pixel Format: {}\n",
                    "    Mode Position: {}\n",
                    "    Logical Position: {}\n",
                    "  Physical:\n",
                    "    Name: {}\n",
                    "    Path: {}\n",
                    "    Manufacturer: {}\n",
                    "    Model: {}\n",
                    "    Serial: {}\n",
                    "    Brightness: {}\n"
                ),
                display_name,
                id.outer.name.unwrap_or_else(|| "unknown".to_string()),
                serial_number,
                id.path.unwrap_or_else(|| "unknown".to_string()),
                metadata.logical.name,
                metadata.logical.path,
                metadata
                    .logical
                    .manufacturer
                    .unwrap_or_else(|| "unknown".to_string()),
                metadata
                    .logical
                    .model
                    .unwrap_or_else(|| "unknown".to_string()),
                metadata
                    .logical
                    .serial_number
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "unknown".to_string()),
                display.logical.state.is_enabled,
                display.logical.state.orientation,
                format_size(display.logical.state.logical_size.as_ref()),
                format_size(display.logical.state.mode_size.as_ref()),
                format_scale_ratio(display.logical.state.scale_ratio_milli),
                format_optional(display.logical.state.pixel_format),
                format_point(display.logical.state.mode_position.as_ref()),
                format_point(display.logical.state.logical_position.as_ref()),
                physical
                    .map(|physical| physical.metadata.name.clone())
                    .unwrap_or_else(|| "unavailable".to_string()),
                physical
                    .map(|physical| physical.metadata.path.clone())
                    .unwrap_or_else(|| "unavailable".to_string()),
                physical
                    .and_then(|physical| physical.metadata.manufacturer.clone())
                    .unwrap_or_else(|| "unavailable".to_string()),
                physical
                    .and_then(|physical| physical.metadata.model.clone())
                    .unwrap_or_else(|| "unavailable".to_string()),
                physical
                    .and_then(|physical| physical.metadata.serial_number.clone())
                    .unwrap_or_else(|| "unavailable".to_string()),
                physical
                    .and_then(|physical| {
                        physical
                            .state
                            .brightness
                            .as_ref()
                            .map(|brightness| format!("{}%", brightness.value()))
                    })
                    .unwrap_or_else(|| "unavailable".to_string()),
            );
            println!();
        }

        if !found_match {
            println!("No displays matched the query.");
        }
        Ok(())
    }
}
