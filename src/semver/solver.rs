use std::sync::Arc;

use ahash::{AHashMap, AHashSet};
use anyhow::{Context, Result};
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
        constraints_for_deps: Default::default(),
        resolution_started: Default::default(),
    };

    solver.solve().await
}

struct Solver {
    constraints: Arc<Constraints>,
    pkg_mgr: Arc<dyn PackageManager>,

    cached_pkgs: RwLock<AHashMap<PackageName, Versions>>,

    /// Used to prevent infinite recursion of `resolve_pkg_recursively`.
    resolution_started: RwLock<AHashSet<PackageName>>,
    constraints_for_deps: RwLock<AHashMap<PackageVersion, ConstraintsPerPkg>>,
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

    #[tracing::instrument(skip(self, constraints), fields(name = %name))]
    #[async_recursion]
    async fn resolve_pkg_recursively(
        &self,
        name: PackageName,
        constraints: Arc<ConstraintsPerPkg>,
    ) -> Result<()> {
        let pkg_constraints = constraints
            .get(&name)
            .cloned()
            .unwrap_or_else(|| panic!("the constraint for package `{}` does not exist", name));

        if !self.resolution_started.write().await.insert(name.clone()) {
            return Ok(());
        }

        info!("Resolving package `{}` recursively", name);

        let pkg = self
            .get_pkg(&PackageConstraint {
                name: name.clone(),
                constraints: pkg_constraints,
            })
            .await
            .with_context(|| {
                format!("failed to fetch package data to resolve {name} recursively")
            })?;

        for p in pkg.iter() {
            let mut dep_constraints = ConstraintsPerPkg::default();

            for dep in p.deps.iter() {
                // TODO: Intersect
                dep_constraints.insert(dep.name.clone(), dep.range.clone());
            }
            let dep_constraints = Arc::new(dep_constraints);

            let futures = FuturesUnordered::new();

            for dep in p.deps.iter() {
                let name = name.clone();
                let dep_name = dep.name.clone();
                let dep_constraints = dep_constraints.clone();

                futures.push(async move {
                    self.resolve_pkg_recursively(dep_name.clone(), dep_constraints)
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
                f?;
            }
        }

        Ok(())
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

        {
            let mut constraints = ConstraintsPerPkg::default();

            for constraint in self.constraints.compatible_packages.iter() {
                constraints.insert(constraint.name.clone(), constraint.constraints.clone());
            }

            let constraints = Arc::new(constraints);

            for pkg in self.constraints.compatible_packages.iter() {
                self.resolve_pkg_recursively(pkg.name.clone(), constraints.clone())
                    .await?;
            }
        };

        dbg!(&self.constraints_for_deps.read().await);

        Ok(Solution {})
    }
}
