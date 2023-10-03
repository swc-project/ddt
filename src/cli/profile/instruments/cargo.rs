use anyhow::{Context, Result};
use clap::Args;

use super::run::RunCommand;
use crate::cli::profile::instruments::util::XcodeInstruments;

/// Invoke a binary file under the `instruments` tool.
#[derive(Debug, Clone, Args)]
pub(super) struct CargoCommand {
    #[clap(long, short = 't')]
    pub template: String,

    #[clap(long)]
    pub time_limit: Option<usize>,

    #[clap(long)]
    pub no_open: bool,

    pub args: Vec<String>,
}

impl CargoCommand {
    pub async fn run(self, xctrace_tool: XcodeInstruments) -> Result<()> {
        let cmd = RunCommand {
            template: self.template,
            time_limit: self.time_limit,
            no_open: self.no_open,
            args: self.args,
        };

        cmd.run(xctrace_tool)
            .await
            .context("failed to run `ddt profile instruments run` with the built binary")
    }
}
