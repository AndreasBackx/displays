// #![windows_subsystem = "windows"]
use anyhow::Result;
use clap::Parser;
use dialoguer::Confirm;
use displays_lib::{
    display_config::{DisplayConfig, DisplayConfigs},
    state::State,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    tv: bool,
    #[arg(long)]
    yes: bool,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        // .event_format(
        //     tracing_subscriber::fmt::format()
        //         .with_file(true)
        //         .with_line_number(true),
        // )
        .init();

    let cli = Cli::parse();

    let setups = DisplayConfigs {
        displays: vec![
            DisplayConfig {
                name: "LG TV".to_owned(),
                path: None,
                enabled: cli.tv,
            },
            DisplayConfig {
                name: "AW3225QF".to_owned(),
                path: None,
                enabled: !cli.tv,
            },
            DisplayConfig {
                name: "Y32p-30".to_owned(),
                path: Some(
                    r"\\?\DISPLAY#LEN66F9#7&289ec95a&0&UID264#{e6f07b5f-ee97-4a90-b076-33f57bf4eaa7}".to_owned(),
                ),
                enabled: !cli.tv,
            },
            DisplayConfig {
                name: "Y32p-30".to_owned(),
                path: Some(
                    r"\\?\DISPLAY#LEN66F9#7&289ec95a&0&UID260#{e6f07b5f-ee97-4a90-b076-33f57bf4eaa7}".to_owned(),
                ),
                enabled: !cli.tv,
            },
        ],
    };

    let mut state = State::query()?;

    println!("Before");
    println!("{state}");

    state.update(setups)?;

    println!("After");
    println!("{state}");

    state.apply(true)?;

    let should_apply = cli.yes
        || Confirm::new()
            .with_prompt("Validation successful, apply?")
            .interact()?;

    if should_apply {
        state.apply(false)?;
        println!("Display configuration updated successfully.");
    }

    Ok(())
}
