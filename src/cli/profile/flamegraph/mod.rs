use clap::{Args, Subcommand};

use self::run::RunCommand;

mod cargo;
mod run;

#[derive(Debug, Args)]
pub(super) struct FlamegraphCommand {
    #[clap(subcommand)]
    cmd: Inner,
}

#[derive(Debug, Subcommand)]
enum Inner {
    Run(RunCommand),
    Cargo(CargoCommand),
}
