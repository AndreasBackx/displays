use clap::Parser;

use crate::commands::Command;

#[cfg(all(feature = "windows", feature = "linux"))]
compile_error!("features 'windows' and 'linux' are mutually exclusive");
#[cfg(not(any(feature = "windows", feature = "linux")))]
compile_error!("enable exactly one backend feature: 'windows' or 'linux'");

mod commands;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: commands::Commands,
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    cli.command.run()
}
