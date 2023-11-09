use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Args;

use crate::{
    package_manager::{cargo::CargoPackageManager, Dependency},
    semver::solver::{solve, Constraints},
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
                    candidate_packages: vec![
                        "swc_ecma_utils".into(),
                        "swc_ecma_ast".into(),
                        "swc_common".into(),
                    ],
                    compatible_packages: vec![Dependency {
                        name: "swc_core".into(),
                        constraints: "0.79.0".parse().unwrap(),
                    }],
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
