use clap::Parser;

use crate::commands::Command;
use displays::{self as lib};

#[derive(Parser)]
pub struct QueryCommand {
    #[clap(long("enabled"))]
    is_enabled: Option<bool>,
}

impl Command for QueryCommand {
    fn run(&self) -> eyre::Result<()> {
        #[cfg(target_os = "linux")]
        if self.is_enabled.is_some() {
            eyre::bail!("--enabled uses logical display state and is not supported on Linux");
        }

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

            let id = display.id().outer;
            let serial_number = id
                .serial_number
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "unknown".to_string());
            let brightness = display
                .physical
                .as_ref()
                .map(|physical| format!("{}%", physical.state.brightness.value()))
                .unwrap_or_else(|| "unavailable".to_string());

            println!(
                "Display: {}\n  Serial: {}\n  Enabled: {}\n  Resolution: {}x{}\n  Brightness: {}\n",
                id.name.unwrap_or_else(|| "unknown".to_string()),
                serial_number,
                display.logical.state.is_enabled,
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
                brightness,
            );
        }

        if !found_match {
            println!("No displays matched the query.");
        }
        Ok(())
    }
}
