use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::Args;

use crate::{
    cli::profile::instruments::util::{
        profile_target, render_template_catalog, CmdArgs, XcodeInstruments,
    },
    util::wrap,
};

/// Invoke a binary file under the `instruments` tool.
#[derive(Debug, Clone, Args)]
pub(super) struct RunCommand {
    /// The target binary to profile
    pub bin: PathBuf,

    /// List available templates
    #[clap(short = 'l', long)]
    pub list_templates: bool,

    #[clap(long, short = 't')]
    pub template: String,

    #[clap(long)]
    pub time_limit: Option<usize>,

    #[clap(long)]
    pub no_open: bool,

    pub args: Vec<String>,
}

impl RunCommand {
    pub async fn run(self) -> Result<()> {
        let c = self.clone();

        wrap(async move {
            // 1. Detect the type of Xcode Instruments installation
            let xctrace_tool = XcodeInstruments::detect().context("failed to detect xctrace")?;

            // 2. Render available templates if the user asked
            if self.list_templates {
                let catalog = xctrace_tool.available_templates()?;
                println!("{}", render_template_catalog(&catalog));
                return Ok(());
            }

            // 3. Build the specified target
            let workspace = cargo_workspace()?;
            let binaries = compile(&self.target).context("failed to compile")?;

            if binaries.len() != 1 {
                bail!(
                    "This command only supports one binary, but got {:?}",
                    binaries
                )
            }

            let target_filepath = compile(&self.target).context("failed to compile")?;
            let target_filepath = if target_filepath.len() == 1 {
                target_filepath.into_iter().next().unwrap()
            } else {
                bail!(
                    "This command only supports one binary, but got {:?}",
                    target_filepath
                )
            };

            if cfg!(target_arch = "aarch64") {
                codesign(&target_filepath.path)?;
            }

            // 4. Profile the built target, will display menu if no template was selected
            let trace_filepath = profile_target(&target_filepath.path, &xctrace_tool, &self)
                .context("failed to profile built binary")?;

            // 5. Print the trace file's relative path
            {
                let trace_shortpath = trace_filepath
                    .strip_prefix(&workspace)
                    .unwrap_or_else(|_| trace_filepath.as_path())
                    .to_string_lossy();

                eprintln!("Trace file {}", trace_shortpath);
            }

            // 6. Open Xcode Instruments if asked
            if !self.no_open {
                launch_instruments(&trace_filepath)?;
            }

            let trace_file_path = profile_target(
                &self.bin,
                &xctrace_tool,
                &CmdArgs {
                    args: self.args.clone(),
                    template_name: self.template.clone(),
                    time_limit: self.time_limit,
                },
            )
            .context("failed to run instruments")?;

            if !self.no_open {}

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
