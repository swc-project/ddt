use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Args;

use crate::{
    semver::solver::{nqueens, solve, Constraints, PackageConstraint},
    util::wrap,
};

#[derive(Debug, Args)]
pub struct SolveVersionsCommand {}

impl SolveVersionsCommand {
    pub async fn run(self) -> Result<()> {
        wrap(async move {
            //
            for i in 1..7 {
                nqueens(i);
            }

            // TODO: This should be a real data
            let result = solve(Arc::new(Constraints {
                candidate_packages: vec!["swc_ecma_ast".into(), "swc_common".into()],
                compatible_packages: vec![PackageConstraint {
                    name: "swc_core".into(),
                    constraints: "^0.83.1".parse().unwrap(),
                }],
            }))
            .await?;
            dbg!(result);
            Ok(())
        })
        .await
        .context("failed to solve versions")
    }
}
