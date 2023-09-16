use std::sync::Arc;

use ahash::AHashMap;
use anyhow::{anyhow, Context, Result};
use async_recursion::async_recursion;
use async_trait::async_trait;
use auto_impl::auto_impl;
use futures::{stream::FuturesUnordered, StreamExt};
use semver::{Version, VersionReq};
use string_cache::DefaultAtom;
use tokio::sync::RwLock;
use tracing::{debug, info};

#[async_trait]
#[auto_impl(Arc, Box, &)]
pub trait PackageManager: Send + Sync {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageVersion {
    pub name: PackageName,
    pub version: Version,
    pub deps: Vec<Dependency>,
}

#[derive(Debug, Clone)]
pub struct FullPackage {
    pub version: PackageVersion,
    pub constraints_for_deps: ConstraintsPerPkg,
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
        cache_full_pkg: Default::default(),
    };

    solver.solve().await
}

struct Solver {
    constraints: Arc<Constraints>,
    pkg_mgr: Arc<dyn PackageManager>,

    cached_pkgs: RwLock<AHashMap<PackageName, Versions>>,

    cache_full_pkg: RwLock<AHashMap<Versions, Vec<Arc<FullPackage>>>>,
}

/// All versions of a **single** package.
type Versions = Arc<Vec<PackageVersion>>;

type ConstraintsPerPkg = AHashMap<PackageName, VersionReq>;

impl Solver {
    async fn get_pkg(&self, c: &PackageConstraint) -> Result<Versions> {
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

    #[async_recursion]
    async fn resolve_pkg_recursively(
        &self,
        name: PackageName,
        constraints: Arc<ConstraintsPerPkg>,
    ) -> Result<Vec<Arc<FullPackage>>> {
        let pkg_constraints = constraints
            .get(&name)
            .cloned()
            .ok_or_else(|| anyhow!("the constraint for package `{}` does not exist", name))?;

        let pkg = self
            .get_pkg(&PackageConstraint {
                name: name.clone(),
                constraints: pkg_constraints,
            })
            .await
            .with_context(|| {
                format!("failed to fetch package data to resolve {name} recursively")
            })?;

        if let Some(res) = self.cache_full_pkg.read().await.get(&pkg).cloned() {
            return Ok(res);
        }

        let mut result = vec![];

        for p in pkg.iter() {
            let mut dep_constraints = ConstraintsPerPkg::default();

            for dep in p.deps.iter() {
                dep_constraints.insert(p.name.clone(), dep.range.clone());
            }
            let dep_constraints = Arc::new(dep_constraints);

            let futures = FuturesUnordered::new();

            for dep in p.deps.iter() {
                let name = name.clone();
                let dep_name = dep.name.clone();
                let constraints = dep_constraints.clone();

                futures.push(async move {
                    self.resolve_pkg_recursively(dep_name.clone(), constraints)
                        .await
                        .with_context(|| {
                            format!(
                                "failed to resolve a dependency package `{dep_name}` of `{name}`"
                            )
                        })
                });
            }

            let futures = futures.collect::<Vec<_>>().await;

            for f in futures {
                result.extend(
                    f?.into_iter()
                        .map(|f| FullPackage {
                            version: f.version.clone(),
                            constraints_for_deps: {
                                let mut map = (*dep_constraints).clone();
                                map.extend(f.constraints_for_deps.clone());
                                map
                            },
                        })
                        .map(Arc::new),
                );
            }
        }

        self.cache_full_pkg
            .write()
            .await
            .insert(pkg.clone(), result.clone());

        Ok(result)
    }

    async fn solve(&self) -> Result<Solution> {
        info!("Solving versions using Solver");

        let ws = cargo_metadata::MetadataCommand::new()
            .exec()
            .context("failed to run `cargo metadata`")?;

        let ws_pkg_names = ws
            .workspace_members
            .iter()
            .map(|p| p.to_string())
            .map(PackageName::from)
            .collect::<Vec<_>>();

        let constraints = {
            let mut constraints = ConstraintsPerPkg::default();

            for constraint in self.constraints.compatible_packages.iter() {
                constraints.insert(constraint.name.clone(), constraint.constraints.clone());
            }

            let mut result = constraints.clone();
            let constraints = Arc::new(constraints);

            for pkg in self.constraints.compatible_packages.iter() {
                let full_pkgs = self
                    .resolve_pkg_recursively(pkg.name.clone(), constraints.clone())
                    .await?;

                for pkg in full_pkgs.iter() {
                    result.extend((*pkg.constraints_for_deps).clone());
                }
            }

            result
        };

        dbg!(&constraints);

        Ok(Solution {})
    }
}
