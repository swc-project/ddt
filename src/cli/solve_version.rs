use anyhow::{Context, Result};
use clap::Args;

use crate::{semver::solver::nqueens, util::wrap};

#[derive(Debug, Args)]
pub struct SolveVersionsCommand {}

impl SolveVersionsCommand {
    pub async fn run(self) -> Result<()> {
        wrap(async move {
            //
            for i in 3..7 {
                nqueens(i);
            }
            Ok(())
        })
        .await
        .context("failed to solve versions")
    }
}
