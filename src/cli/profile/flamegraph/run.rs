use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;

use crate::util::wrap;

/// Invoke a binary file and create a flamegraph
#[derive(Debug, Clone, Args)]
pub(super) struct RunCommand {
    /// The target binary to profile
    pub bin: PathBuf,

    #[clap(long)]
    pub time_limit: Option<usize>,

    /// The path to the output flamegraph file
    #[clap(long, short = 'o')]
    pub output_path: Option<PathBuf>,

    #[clap(long)]
    pub no_open: bool,

    pub args: Vec<String>,
}

impl RunCommand {
    pub async fn run(self, envs: Vec<(String, String)>) -> Result<()> {
        let c = self.clone();

        wrap(async move {
            //

            Ok(())
        })
        .await
        .with_context(|| {
            format!(
                "failed to create a flamegraph from `{}` `{:?}",
                c.bin.display(),
                c.args
            )
        })
    }
}
