use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;

use crate::{
    cli::{
        profile::instruments::util::{profile_target, CmdArgs, XcodeInstruments},
        util::open_file,
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

    /// The path to the output trace file
    #[clap(long, short = 'o')]
    pub output_path: Option<PathBuf>,

    #[clap(long)]
    pub no_open: bool,

    pub args: Vec<String>,
}

impl RunCommand {
    pub async fn run(
        self,
        xctrace_tool: XcodeInstruments,
        envs: Vec<(String, String)>,
    ) -> Result<()> {
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
                    output_path: self.output_path,
                    envs,
                },
            )
            .context("failed to profile target binary")?;

            // Open Xcode Instruments if asked
            if !self.no_open {
                open_file(&trace_file_path)?;
            }

            Ok(())
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
