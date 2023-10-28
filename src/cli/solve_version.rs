use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Args;
use pubgrub::range::Range;

use crate::{
    semver::{
        cargo::CargoPackageManager,
        solver::{solve, Constraints, PackageConstraint, Semver},
    },
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
                    compatible_packages: vec![PackageConstraint {
                        name: "swc_core".into(),
                        range: Range::singleton("0.79.0".parse::<Semver>()?),
                    }],
                }),
                Arc::new(CargoPackageManager {
                    index: crates_index::GitIndex::new_cargo_default()
                        .context("failed to open crates.io git index")?,
                    target_repo: Some("".into()),
                }),
            )
            .await?;

            Ok(())
        })
        .await
        .context("failed to solve versions")
    }
}
