use anyhow::{bail, Context, Result};
use clap::Args;

use crate::util::wrap;

/// Invoke a binary file under the `instruments` tool.
#[derive(Debug, Args)]
pub(super) struct RunCommand {}

impl RunCommand {
    pub async fn run(self) -> Result<()> {
        wrap(async move {
            // TODO

            bail!("not implemented")
        })
        .await
        .context("failed to run instruments with a specified binary")
    }
}
