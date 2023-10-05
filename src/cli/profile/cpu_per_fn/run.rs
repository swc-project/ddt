use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;

use crate::util::wrap;

/// Invoke a binary file and profile the cpu count per function
#[derive(Debug, Clone, Args)]
pub(super) struct RunCommand {
    /// The target binary to profile
    pub bin: PathBuf,

    #[clap(long)]
    pub time_limit: Option<usize>,

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
                "failed to profile cpu count per function with `{}` `{:?}",
                c.bin.display(),
                c.args
            )
        })
    }
}
