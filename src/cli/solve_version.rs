use std::sync::Arc;

use ahash::AHashSet;
use anyhow::{Context, Result};
use clap::Args;

use crate::{
    package_manager::{cargo::CargoPackageManager, Dependency, PackageName},
    semver::solver::{solve, Constraints},
    util::wrap,
};

#[derive(Debug, Args)]
pub struct SolveVersionsCommand {
    /// Only these packages will be included in the solution.
    ///
    /// Defaults to the direct dependencies of the current cargo workspace.
    #[clap(short = 'p', long = "package")]
    pub intersecting_packages: Vec<PackageName>,

    /// Require these packages to be satisfied by the solution.
    #[clap(short = 'r', long = "require")]
    pub constraints: Vec<Dependency>,
}

impl SolveVersionsCommand {
    pub async fn run(self) -> Result<()> {
        wrap(async move {
            //
            let intersecting_packages = if self.intersecting_packages.is_empty() {
                self.get_direct_deps_of_current_cargo_workspace()?
            } else {
                self.intersecting_packages
            };

            let solution = solve(
                Arc::new(Constraints {
                    candidate_packages: intersecting_packages,
                    compatible_packages: self.constraints,
                }),
                Arc::new(CargoPackageManager),
            )
            .await?;

            let s =
                serde_json::to_string_pretty(&solution).context("failed to serialize solution")?;

            println!("{}", s);
            Ok(())
        })
        .await
        .context("failed to solve versions")
    }

    fn get_direct_deps_of_current_cargo_workspace(&self) -> Result<Vec<PackageName>> {
        let ws = cargo_metadata::MetadataCommand::new()
            .exec()
            .context("failed to run `cargo metadata`")?;

        let ws_pkg_names = ws
            .workspace_members
            .iter()
            .map(|p| p.to_string().splitn(2, '0').next().unwrap().trim().into())
            .collect::<AHashSet<PackageName>>();

        let ws_pkgs = ws
            .packages
            .iter()
            .filter(|pkg| ws_pkg_names.contains(&pkg.name.clone().into()))
            .collect::<Vec<_>>();

        Ok(ws_pkgs
            .into_iter()
            .flat_map(|pkg| pkg.dependencies.iter().map(|d| d.name.clone()))
            .map(PackageName::from)
            .collect())
    }
}
