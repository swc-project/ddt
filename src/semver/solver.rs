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
    async fn solve(&self) -> Result<Solution> {
        let package_info = AHashMap::new();

        for constraint in self.constraints.compatible_packages {
            let versions = self
                .pkg_mgr
                .resolve(&constraint.name, &constraint.constraints)
                .await?;
        }

        // We are interesected only in these packages.
        for p in self.constraints.candidate_packages.iter() {}

        let mut possible_packages = AHashMap::new();

        for constraint in self.constraints.compatible_packages {}
    }
}
