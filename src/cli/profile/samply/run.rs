use std::{path::PathBuf, process::Command};

use anyhow::{Context, Result};
use clap::Args;

use crate::util::wrap;

/// Invoke a binary file under the `instruments` tool.
#[derive(Debug, Clone, Args)]
pub(super) struct RunCommand {
    /// The target binary to profile
    pub bin: PathBuf,

    #[clap(long)]
    pub time_limit: Option<usize>,

    #[clap(long)]
    pub no_open: bool,

    pub args: Vec<String>,
}

impl RunCommand {
    pub async fn run(self, envs: Vec<(String, String)>) -> Result<()> {
        let c = self.clone();

        wrap(async move {
            let mut cmd = Command::new("samply");
            cmd.arg("record").arg(&self.bin);

            for (k, v) in envs {
                cmd.env(k, v);
            }

            for arg in self.args.iter() {
                cmd.arg(arg);
            }

            if !cmd.status()?.success() {
                anyhow::bail!("failed to run samply with `{:?}`", cmd);
            }

            Ok(())
        })
        .await
        .with_context(|| {
            format!(
                "failed to run samply with `{}` `{:?}",
                c.bin.display(),
                c.args
            )
        })
    }
}
