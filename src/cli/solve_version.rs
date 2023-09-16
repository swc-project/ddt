use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Args;

use crate::{
    semver::solver::{solve, CargoPackageManager, Constraints},
    util::wrap,
};

#[derive(Debug, Args)]
pub struct SolveVersionsCommand {}

impl SolveVersionsCommand {
    pub async fn run(self) -> Result<()> {
        wrap(async move {
            //

            solve(
                Arc::new(Constraints {
                    candidate_packages: vec![],
                    compatible_packages: vec![],
                }),
                Arc::new(CargoPackageManager),
            )
            .await?;

            Ok(())
        })
        .await
        .context("failed to solve versions")
    }
}
