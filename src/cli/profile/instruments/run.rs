use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::Args;

use crate::{
    cli::profile::instruments::util::{profile_target, CmdArgs, XcodeInstruments},
    util::wrap,
};

/// Invoke a binary file under the `instruments` tool.
#[derive(Debug, Args)]
pub(super) struct RunCommand {
    pub bin: PathBuf,

    pub args: Vec<String>,
}

impl RunCommand {
    pub async fn run(self) -> Result<()> {
        wrap(async move {
            let xctrace_tool = XcodeInstruments::detect().context("failed to detect xctrace")?;

            profile_target(&self.bin, &xctrace_tool, &CmdArgs {})
                .context("failed to run instruments");

            bail!("not implemented")
        })
        .await
        .with_context(|| {
            format!(
                "failed to run instruments with `{}` `{:?}",
                self.bin.display(),
                self.args
            )
        })
    }
}
