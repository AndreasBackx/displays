use clap::Parser;

use crate::commands::Command;
use displays_lib::{self as lib};

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

        for display in displays {
            if let Some(is_enabled) = self.is_enabled {
                if is_enabled != display.logical.state.is_enabled {
                    continue;
                }
            }
            println!("{display:#?}");
        }
        Ok(())
    }
}
