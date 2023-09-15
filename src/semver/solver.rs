use std::sync::Arc;

use ahash::AHashMap;
use anyhow::Result;
use async_trait::async_trait;
use auto_impl::auto_impl;
use semver::{Version, VersionReq};
use string_cache::DefaultAtom;

#[async_trait]
#[auto_impl(Arc, Box, &)]
pub trait PackageManager {
    async fn resolve(
        &self,
        package_name: &str,
        constraints: &VersionReq,
    ) -> Result<Vec<PackageVersion>>;
}

pub type PackageName = DefaultAtom;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Constraints {
    /// Only packages with these names will be considered.
    pub candidate_packages: Vec<PackageName>,

    /// These packages must be included in the solution.
    pub compatible_packages: Vec<PackageConstraint>,
}

#[derive(Debug, Clone)]
pub struct Solution {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageConstraint {
    pub name: PackageName,
    pub constraints: VersionReq,
}

#[derive(Debug, Default)]
struct CargoPackageManager {}

#[async_trait]
impl PackageManager for CargoPackageManager {
    async fn resolve(
        &self,
        package_name: &str,
        constraints: &VersionReq,
    ) -> Result<Vec<PackageVersion>> {
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageVersion {
    pub version: Version,
    pub deps: Vec<Dependency>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Dependency {
    pub version: Version,
    pub range: VersionReq,
}

pub async fn solve(
    constraints: Arc<Constraints>,
    pkg_mgr: Arc<dyn PackageManager>,
) -> Result<Solution> {
    let solver = Solver {
        constraints,
        pkg_mgr,
    };

    solver.solve().await
}

struct Solver {
    constraints: Arc<Constraints>,
    pkg_mgr: Arc<dyn PackageManager>,
}

impl Solver {
    async fn get_all_packkages(
        &self,
    ) -> Result<AHashMap<PackageName, AHashMap<Version, PackageVersion>>> {
        let mut all_pkgs: AHashMap<PackageName, AHashMap<Version, PackageVersion>> =
            AHashMap::new();

        for constraint in self.constraints.compatible_packages.iter() {
            let versions = self
                .pkg_mgr
                .resolve(&constraint.name, &constraint.constraints)
                .await?;

            let e = all_pkgs.entry(constraint.name.clone()).or_default();

            for v in versions {
                e.insert(v.version.clone(), v);
            }
        }

        Ok(all_pkgs)
    }

    async fn solve(&self) -> Result<Solution> {
        let all_pkgs = self.get_all_packkages();

        Ok(Solution {})
    }
}
