use anyhow::{Context, Result};
use clap::Args;

use super::run::RunCommand;
use crate::{
    cli::util::cargo::get_one_binary_using_cargo,
    util::{cargo_build::CargoBuildTarget, wrap},
};

/// Invoke a binary file built using `cargo` under the `instruments` tool.
#[derive(Debug, Clone, Args)]
pub(super) struct CargoCommand {
    #[clap(long)]
    time_limit: Option<usize>,

    #[clap(long)]
    no_open: bool,

    #[clap(flatten)]
    build_target: CargoBuildTarget,

    /// Arguments passed to the target binary.
    ///
    /// To pass flags, precede child args with `--`,
    /// e.g. `ddt profile subcommand -- -t test1.txt --slow-mode`.
    args: Vec<String>,
}

impl CargoCommand {
    pub async fn run(self) -> Result<()> {
        let (cmd, envs) = wrap(async move {
            let (bin, envs) = get_one_binary_using_cargo(&self.build_target).await?;

            Ok((
                RunCommand {
                    time_limit: self.time_limit,
                    no_open: self.no_open,
                    args: self.args,
                    bin: bin.path,
                },
                envs,
            ))
        })
        .await
        .context("failed to build the target binary using cargo")?;

        cmd.run(envs)
            .await
            .context("failed to run `ddt profile samply run` with the built binary")
    }
}
