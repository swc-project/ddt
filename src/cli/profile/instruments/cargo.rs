use anyhow::{Context, Result};
use clap::Args;

use super::run::RunCommand;
use crate::{
    cli::profile::instruments::util::XcodeInstruments,
    util::{
        cargo_build::{compile, CargoBuildTarget},
        wrap,
    },
};

/// Invoke a binary file under the `instruments` tool.
#[derive(Debug, Clone, Args)]
pub(super) struct CargoCommand {
    #[clap(long, short = 't')]
    template: String,

    #[clap(long)]
    time_limit: Option<usize>,

    #[clap(long)]
    no_open: bool,

    #[clap(flatten)]
    build_target: CargoBuildTarget,

    /// Arguments passed to the target binary.
    ///
    /// To pass flags, precede child args with `--`,
    /// e.g. `cargo profile subcommand -- -t test1.txt --slow-mode`.
    args: Vec<String>,
}

impl CargoCommand {
    pub async fn run(self, xctrace_tool: XcodeInstruments) -> Result<()> {
        let cmd = wrap(async move {
            let bins =
                compile(&self.build_target).context("failed to build the binary using cargo")?;

            RunCommand {
                template: self.template,
                time_limit: self.time_limit,
                no_open: self.no_open,
                args: self.args,
            }
        })
        .await
        .context("failed to build the target binary using cargo")?;

        cmd.run(xctrace_tool)
            .await
            .context("failed to run `ddt profile instruments run` with the built binary")
    }
}
