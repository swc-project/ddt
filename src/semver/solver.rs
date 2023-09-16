use std::sync::Arc;

use ahash::AHashMap;
use anyhow::Result;
use async_trait::async_trait;
use auto_impl::auto_impl;
use semver::{Version, VersionReq};
use string_cache::DefaultAtom;
use tokio::sync::RwLock;

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
        cached_pkgs: Default::default(),
    };

    solver.solve().await
}

struct Solver {
    constraints: Arc<Constraints>,
    pkg_mgr: Arc<dyn PackageManager>,

    /// Being lazy is very important here. It will reduce parallelism, but
    /// reducing network operation is much bigger.
    cached_pkgs: RwLock<AHashMap<PackageName, Arc<AHashMap<Version, PackageVersion>>>>,
}

impl Solver {
    async fn get_pkg(
        &self,
        c: &PackageConstraint,
    ) -> Result<Arc<AHashMap<Version, PackageVersion>>> {
        if let Some(pkgs) = self.cached_pkgs.read().await.get(&c.name) {
            return Ok(pkgs.clone());
        }

        let versions = self.pkg_mgr.resolve(&c.name, &c.constraints).await?;

        let versions: AHashMap<Version, PackageVersion> = versions
            .into_iter()
            .map(|v| (v.version.clone(), v))
            .collect();

        let versions = Arc::new(versions);

        self.cached_pkgs
            .write()
            .await
            .insert(c.name.clone(), versions.clone());

        Ok(versions)
    }

    async fn solve(&self) -> Result<Solution> {
        Ok(Solution {})
    }
}
