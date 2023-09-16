use std::sync::Arc;

use ahash::AHashMap;
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use auto_impl::auto_impl;
use semver::{Version, VersionReq};
use string_cache::DefaultAtom;
use tokio::sync::RwLock;
use tracing::{debug, info};

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

                PackageVersion {
                    name: package_name.into(),
                    version: ver,
                    deps,
                }
            })
            .collect())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageVersion {
    pub name: PackageName,
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
    cached_pkgs: RwLock<AHashMap<PackageName, Arc<Vec<PackageVersion>>>>,
}

impl Solver {
    async fn get_pkg(&self, c: &PackageConstraint) -> Result<Arc<Vec<PackageVersion>>> {
        if let Some(pkgs) = self.cached_pkgs.read().await.get(&c.name) {
            return Ok(pkgs.clone());
        }

        debug!("Resolving package `{}`", c.name);

        let versions = self.pkg_mgr.resolve(&c.name, &c.constraints).await?;

        let versions = Arc::new(versions);

        self.cached_pkgs
            .write()
            .await
            .insert(c.name.clone(), versions.clone());

        Ok(versions)
    }

    async fn solve(&self) -> Result<Solution> {
        info!("solving versions using Solver");

        let ws = cargo_metadata::MetadataCommand::new()
            .exec()
            .context("failed to run `cargo metadata`")?;

        let ws_pkg_names = ws
            .workspace_members
            .iter()
            .map(|p| p.to_string())
            .map(PackageName::from)
            .collect::<Vec<_>>();

        // Merge all constraints into one, but per package.
        let mut constarints_per_pkg = AHashMap::<_, Vec<_>>::default();

        for constraint in self.constraints.compatible_packages.iter() {
            let versions = self.get_pkg(constraint).await?;

            let e = constarints_per_pkg
                .entry(constraint.name.clone())
                .or_default();

            for v in versions.iter() {
                for dep in v.deps.iter() {
                    e.push(dep.range.clone());
                }
            }
        }

        // We now iterate over the merged constraints (again, per package) and combine
        // them to one per a package.
        let mut merged_constraints = AHashMap::<_, VersionReq>::default();

        dbg!(&constarints_per_pkg);

        Ok(Solution {})
    }
}
