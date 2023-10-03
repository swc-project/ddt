use std::path::Path;

use anyhow::{bail, Context, Result};
use clap::{Args, Subcommand};

use self::{run::RunCommand, util::XcodeInstruments};
use crate::{cli::profile::instruments::util::render_template_catalog, util::wrap};

mod run;
mod util;

/// Invokes `instruments` from xcode. Works only on macOS.
#[derive(Debug, Args)]
pub(super) struct InstrumentsCommand {
    /// List available templates
    #[clap(short = 'l', long)]
    list_templates: bool,

    #[clap(subcommand)]
    cmd: Inner,
}

impl InstrumentsCommand {
    pub async fn run(self) -> Result<()> {
        wrap(async move {
            // Detect the type of Xcode Instruments installation
            let xctrace_tool = XcodeInstruments::detect().context("failed to detect xctrace")?;

            // Render available templates if the user asked
            if self.list_templates {
                let catalog = xctrace_tool.available_templates()?;
                println!("{}", render_template_catalog(&catalog));
                return Ok(());
            }

            match self.cmd {
                Inner::Run(cmd) => cmd.run(xctrace_tool).await,
            }
        })
        .await
        .context("failed to run instruments")
    }
}

#[derive(Debug, Subcommand)]
enum Inner {
    Run(RunCommand),
}

/// Launch Xcode Instruments on the provided trace file.
fn launch_instruments(trace_filepath: &Path) -> Result<()> {
    use std::process::Command;

    let status = Command::new("open").arg(trace_filepath).status()?;

    if !status.success() {
        bail!("`open` failed")
    }
    Ok(())
}
