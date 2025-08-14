use clap::Parser;

use crate::commands::Command;
use displays_lib::{self as lib};

#[derive(Parser)]
pub struct QueryCommand {}

impl Command for QueryCommand {
    fn run(&self) -> eyre::Result<()> {
        let displays = lib::manager::DisplayManager::query()?
            .into_iter()
            .collect::<Vec<_>>();

        for display in displays {
            println!("{display:#?}");
        }
        Ok(())
    }
}
