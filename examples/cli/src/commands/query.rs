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
                    "    Resolution: {}x{}\n",
                    "    Pixel Format: {}\n",
                    "    Position: {}\n",
                    "  Physical:\n",
                    "    Name: {}\n",
                    "    Path: {}\n",
                    "    Serial: {}\n",
                    "    Brightness: {}\n",
                    "    Brightness Scale: {}\n"
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
                display
                    .logical
                    .state
                    .width
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "unknown".to_string()),
                display
                    .logical
                    .state
                    .height
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "unknown".to_string()),
                format_optional(display.logical.state.pixel_format),
                format_point(display.logical.state.position.as_ref()),
                physical
                    .map(|physical| physical.metadata.name.clone())
                    .unwrap_or_else(|| "unavailable".to_string()),
                physical
                    .map(|physical| physical.metadata.path.clone())
                    .unwrap_or_else(|| "unavailable".to_string()),
                physical
                    .map(|physical| physical.metadata.serial_number.clone())
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "unavailable".to_string()),
                physical
                    .map(|physical| format!("{}%", physical.state.brightness.value()))
                    .unwrap_or_else(|| "unavailable".to_string()),
                physical
                    .map(|physical| physical.state.scale_factor.to_string())
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
