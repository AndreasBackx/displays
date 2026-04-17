use clap::Parser;

use crate::commands::Command;
use displays::{
    self as lib,
    display::DisplayUpdate,
    types::{DisplayIdentifier, LogicalDisplayUpdateContent, PhysicalDisplayUpdateContent, Point},
};

#[derive(Parser)]
pub struct ApplyCommand {
    #[clap(flatten)]
    update: DisplayUpdateArgs,

    #[clap(long, action = clap::ArgAction::SetTrue)]
    validate: bool,
}

#[derive(Parser, Clone)]
struct DisplayUpdateArgs {
    #[clap(flatten)]
    id: DisplayIdentifierArgs,
    #[clap(flatten)]
    pub logical: Option<LogicalDisplayUpdateContentArgs>,
    #[clap(flatten)]
    pub physical: Option<PhysicalDisplayUpdateContentArgs>,
}

impl From<DisplayUpdateArgs> for DisplayUpdate {
    fn from(value: DisplayUpdateArgs) -> Self {
        DisplayUpdate {
            id: value.id.into(),
            logical: value.logical.map(|val| val.into()),
            physical: value.physical.map(|val| val.into()),
        }
    }
}

#[derive(Parser, Clone)]
struct DisplayIdentifierArgs {
    #[clap(long)]
    name: Option<String>,
    #[clap(long)]
    serial_number: Option<String>,
}

impl From<DisplayIdentifierArgs> for DisplayIdentifier {
    fn from(value: DisplayIdentifierArgs) -> Self {
        DisplayIdentifier {
            name: value.name,
            serial_number: value.serial_number,
        }
    }
}

#[derive(Parser, Clone)]
struct LogicalDisplayUpdateContentArgs {
    #[clap(long)]
    is_enabled: Option<bool>,
    // #[clap(long)]
    // orientation: Option<Orientation>,
    #[clap(long)]
    width: Option<u32>,
    #[clap(long)]
    height: Option<u32>,
    // #[clap(long)]
    // pixel_format: Option<PixelFormat>,
    #[clap(long, allow_hyphen_values = true)]
    position: Option<Point>,
}

impl From<LogicalDisplayUpdateContentArgs> for LogicalDisplayUpdateContent {
    fn from(value: LogicalDisplayUpdateContentArgs) -> Self {
        LogicalDisplayUpdateContent {
            is_enabled: value.is_enabled,
            // orientation: value.orientation,
            width: value.width,
            height: value.height,
            // pixel_format: value.pixel_format,
            position: value.position,
            ..Default::default()
        }
    }
}

#[derive(Parser, Clone)]
struct PhysicalDisplayUpdateContentArgs {
    #[clap(long)]
    brightness: Option<u32>,
}

impl From<PhysicalDisplayUpdateContentArgs> for PhysicalDisplayUpdateContent {
    fn from(value: PhysicalDisplayUpdateContentArgs) -> Self {
        PhysicalDisplayUpdateContent {
            brightness: value.brightness,
        }
    }
}

impl Command for ApplyCommand {
    fn run(&self) -> eyre::Result<()> {
        let update = self.update.clone().into();

        let results = lib::manager::DisplayManager::apply(vec![update], self.validate)?;

        for result in results {
            let target = result
                .requested_update
                .id
                .name
                .or_else(|| {
                    result
                        .requested_update
                        .id
                        .serial_number
                        .filter(|value| !value.is_empty())
                })
                .unwrap_or_else(|| "unknown display".to_string());

            if result.applied.is_empty() && result.failed.is_empty() {
                println!("No displays matched {target}.");
                continue;
            }

            if result.failed.is_empty() {
                let verb = if self.validate {
                    "validated"
                } else {
                    "updated"
                };
                println!(
                    "Successfully {verb} {} display(s) for {target}.",
                    result.applied.len()
                );
                continue;
            }

            if !result.applied.is_empty() {
                println!(
                    "Partially applied {target}: {} succeeded, {} failed.",
                    result.applied.len(),
                    result.failed.len()
                );
            } else {
                println!("Failed to apply {target}:");
            }

            for failure in result.failed {
                let display_name = failure
                    .matched_id
                    .name
                    .or(failure.matched_id.serial_number)
                    .unwrap_or_else(|| "unknown display".to_string());
                println!("- {display_name}: {}", failure.message);
            }
        }
        Ok(())
    }
}
