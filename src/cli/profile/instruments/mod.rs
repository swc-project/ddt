use anyhow::{Context, Result};
use clap::{Args, Subcommand};

use self::{
    cargo::CargoCommand, list_templates::ListTemplatesCommand, run::RunCommand,
    util::XcodeInstruments,
};
use crate::util::wrap;

mod cargo;
mod list_templates;
mod run;
mod util;

/// Invokes `instruments` from xcode. Works only on macOS.
#[derive(Debug, Args)]
pub(super) struct InstrumentsCommand {
    #[clap(subcommand)]
    cmd: Inner,
}

impl InstrumentsCommand {
    pub async fn run(self) -> Result<()> {
        wrap(async move {
            // Detect the type of Xcode Instruments installation
            let xctrace_tool = XcodeInstruments::detect().context("failed to detect xctrace")?;

            match self.cmd {
                Inner::ListTemplates(cmd) => cmd.run(xctrace_tool).await,
                Inner::Run(cmd) => cmd.run(xctrace_tool, Default::default()).await,
                Inner::Cargo(cmd) => cmd.run(xctrace_tool).await,
            }
        })
        .await
        .context("failed to run instruments")
    }
}

#[derive(Debug, Subcommand)]
enum Inner {
    Run(RunCommand),
    ListTemplates(ListTemplatesCommand),
    Cargo(CargoCommand),
}
