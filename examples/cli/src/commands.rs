use clap::Subcommand;
use enum_dispatch::enum_dispatch;

mod apply;
mod query;

#[enum_dispatch]
pub trait Command {
    fn run(&self) -> eyre::Result<()>;
}

#[enum_dispatch(Command)]
#[derive(Subcommand)]
pub enum Commands {
    Query(query::QueryCommand),
    Apply(apply::ApplyCommand),
}
