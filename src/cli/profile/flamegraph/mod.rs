use clap::{Args, Subcommand};

/// Create a flamegraph by running a program
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
