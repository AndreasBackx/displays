use clap::Parser;

use crate::commands::Command;

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
compile_error!("example_cli currently supports only Windows and Linux targets");

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
