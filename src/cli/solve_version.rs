use anyhow::{Context, Result};
use clap::Args;

use crate::util::wrap;

#[derive(Debug, Args)]
pub struct SolveVersionsCommand {}

impl SolveVersionsCommand {
    pub async fn run(self) -> Result<()> {
        wrap(async move {
            //

            Ok(())
        })
        .await
        .context("failed to solve versions")
    }
}
