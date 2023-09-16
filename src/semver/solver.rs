use std::sync::Arc;

use ahash::AHashMap;
use anyhow::{anyhow, Context, Result};
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
pub struct CargoPackageManager;

#[async_trait]
impl PackageManager for CargoPackageManager {
    async fn resolve(
        &self,
        package_name: &str,
        constraints: &VersionReq,
    ) -> Result<Vec<PackageVersion>> {
        let index = crates_index::GitIndex::new_cargo_default()?;
        let pkg = index
            .crate_(package_name)
            .ok_or_else(|| anyhow!("Package `{}` not found in index", package_name))?;

        Ok(pkg
            .versions()
            .iter()
            .map(|v| {
                let ver = v.version().parse().expect("invalid version");

                (ver, v.dependencies().to_vec())
            })
            .filter(|(v, _)| constraints.matches(v))
            .map(|(ver, deps)| {
                let deps = deps
                    .iter()
                    .map(|d| Dependency {
                        name: d.name().into(),
                        range: d
                            .requirement()
                            .parse()
                            .expect("invalid version requirenment"),
                    })
                    .collect();

                PackageVersion { version: ver, deps }
            })
            .collect())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageVersion {
    pub version: Version,
    pub deps: Vec<Dependency>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Dependency {
    pub name: PackageName,
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
        let ws = cargo_metadata::MetadataCommand::new()
            .exec()
            .context("failed to run `cargo metadata`")?;

        let interesting_packages = ws
            .workspace_members
            .iter()
            .map(|p| p.to_string())
            .map(PackageName::from)
            .collect::<Vec<_>>();

        Ok(Solution {})
    }
}
