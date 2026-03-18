use clap::Parser;

use crate::commands::Command;
use displays::{
    self as lib, display::DisplayUpdate, display_identifier::DisplayIdentifier,
    logical_display::LogicalDisplayUpdateContent, physical_display::PhysicalDisplayUpdateContent,
    types::Point,
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
            // position: value.position,
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
        #[cfg(target_os = "linux")]
        if self.update.logical.is_some() {
            eyre::bail!("logical display updates are not supported on Linux");
        }

        let update = self.update.clone().into();

        let remaining_updates = lib::manager::DisplayManager::apply(vec![update], self.validate)?
            .into_iter()
            .collect::<Vec<_>>();

        if remaining_updates.is_empty() {
            if self.validate {
                println!("Validation succeeded; all updates can be applied.");
            } else {
                println!("Update applied successfully.");
            }
            return Ok(());
        }

        println!("Some updates could not be matched or applied:");

        for update in remaining_updates {
            let target = update
                .id
                .name
                .or(update.id.serial_number)
                .unwrap_or_else(|| "unknown display".to_string());
            println!("- {target}");
        }
        Ok(())
    }
}
