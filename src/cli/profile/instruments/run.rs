use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::Args;

use crate::{
    cli::profile::instruments::{
        launch_instruments,
        util::{profile_target, CmdArgs, XcodeInstruments},
    },
    util::wrap,
};

/// Invoke a binary file under the `instruments` tool.
#[derive(Debug, Clone, Args)]
pub(super) struct RunCommand {
    /// The target binary to profile
    pub bin: PathBuf,

    #[clap(long, short = 't')]
    pub template: String,

    #[clap(long)]
    pub time_limit: Option<usize>,

    #[clap(long)]
    pub no_open: bool,

    pub args: Vec<String>,
}

impl RunCommand {
    pub async fn run(self, xctrace_tool: XcodeInstruments) -> Result<()> {
        let c = self.clone();

        wrap(async move {
            // Profile the built target, will display menu if no template was selected
            let trace_file_path = profile_target(
                &self.bin,
                &xctrace_tool,
                &CmdArgs {
                    args: self.args.clone(),
                    template_name: self.template.clone(),
                    time_limit: self.time_limit,
                },
            )
            .context("failed to profile target binary")?;

            // Open Xcode Instruments if asked
            if !self.no_open {
                launch_instruments(&trace_file_path)?;
            }

            bail!("not implemented")
        })
        .await
        .with_context(|| {
            format!(
                "failed to run instruments with `{}` `{:?}",
                c.bin.display(),
                c.args
            )
        })
    }
}
