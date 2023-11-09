use std::{sync::Arc, time::Instant};

use ahash::{AHashMap, AHashSet};
use anyhow::{Context, Result};
use async_recursion::async_recursion;
use async_trait::async_trait;
use auto_impl::auto_impl;
use futures::{stream::FuturesUnordered, StreamExt};
use semver::{Version, VersionReq};
use tokio::sync::RwLock;
use tracing::{debug, info};

use super::{constraints::ConstraintStorage, PackageName};

#[async_trait]
#[auto_impl(Arc, Box, &)]
pub trait PackageManager: Send + Sync {
    async fn resolve(
        &self,
        package_name: &str,
        constraints: &VersionReq,
    ) -> Result<Vec<PackageVersion>>;
}

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

/// All versions of a **single** package.
type Versions = Arc<Vec<PackageVersion>>;

pub async fn solve(
    constraints: Arc<Constraints>,
    pkg_mgr: Arc<dyn PackageManager>,
) -> Result<Solution> {
    let solver = Solver {
        constraints,
        pkg_mgr,
        cached_pkgs: Default::default(),
        resolution_started: Default::default(),
    };

    solver.solve().await
}

struct Solver {
    constraints: Arc<Constraints>,
    pkg_mgr: Arc<dyn PackageManager>,

    cached_pkgs: RwLock<AHashMap<PackageConstraint, Versions>>,

    /// Used to prevent infinite recursion of `resolve_pkg_recursively`.
    resolution_started: RwLock<AHashSet<PackageName>>,
}

impl Solver {
    async fn get_pkg(&self, c: &PackageConstraint) -> Result<Versions> {
        if let Some(pkgs) = self.cached_pkgs.read().await.get(c) {
            return Ok(pkgs.clone());
        }

        debug!("Resolving package `{}`", c.name);

        let versions = self.pkg_mgr.resolve(&c.name, &c.constraints).await?;

        let versions = Arc::new(versions);

        self.cached_pkgs
            .write()
            .await
            .insert(c.clone(), versions.clone());

        Ok(versions)
    }

    #[tracing::instrument(skip(self, parent_constraints), fields(name = %name))]
    #[async_recursion]
    async fn resolve_pkg_recursively(
        &self,
        name: PackageName,
        parent_constraints: Arc<ConstraintStorage>,
    ) -> Result<()> {
        let pkg_constraints = parent_constraints
            .get(&name)
            .cloned()
            .unwrap_or_else(|| panic!("the constraint for package `{}` does not exist", name));

        if !self.resolution_started.write().await.insert(name.clone()) {
            return Ok(());
        }

        debug!("Resolving package `{}` recursively", name);

        let pkg = self
            .get_pkg(&PackageConstraint {
                name: name.clone(),
                constraints: pkg_constraints,
            })
            .await
            .with_context(|| {
                format!("failed to fetch package data to resolve {name} recursively")
            })?;

        let futures = FuturesUnordered::new();

        for pkg in pkg.iter() {
            let name = name.clone();
            let pkg = pkg.clone();
            let parent_constraints = parent_constraints.clone();

            futures.push(async move {
                self.resolve_deps(name.clone(), pkg, parent_constraints)
                    .await
            });
        }

        let futures = futures.collect::<Vec<_>>().await;

        for f in futures {
            f?;
        }

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    #[async_recursion]
    async fn resolve_deps(
        &self,
        name: PackageName,
        pkg: PackageVersion,
        parent_constraints: Arc<ConstraintStorage>,
    ) -> Result<()> {
        let mut dep_constraints = ConstraintStorage::new(parent_constraints);

        for dep in pkg.deps.iter() {
            dep_constraints.insert(dep.name.clone(), dep.range.clone());
        }

        let dep_constraints = dep_constraints.freeze();

        let futures = FuturesUnordered::new();

        for dep in pkg.deps.iter() {
            let name = name.clone();
            let dep_name = dep.name.clone();
            let dep_constraints = dep_constraints.clone();

            futures.push(async move {
                self.resolve_pkg_recursively(dep_name.clone(), dep_constraints)
                    .await
                    .with_context(|| {
                        format!("failed to resolve a dependency package `{dep_name}` of `{name}`")
                    })
            });
        }

        let futures = futures.collect::<Vec<_>>().await;

        ConstraintStorage::unfreeze(dep_constraints)
            .remove_parent()
            .await;

        for f in futures {
            f?;
        }

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn solve(&self) -> Result<Solution> {
        self.solver_inner().await
    }

    async fn solver_inner(&self) -> Result<Solution> {
        info!("Solving versions using Solver");

        let start = Instant::now();
        let mut constraints = {
            let mut constraints = ConstraintStorage::root();

            for constraint in self.constraints.compatible_packages.iter() {
                constraints.insert(constraint.name.clone(), constraint.constraints.clone());
            }

            let constraints = constraints.freeze();

            for pkg in self.constraints.compatible_packages.iter() {
                self.resolve_pkg_recursively(pkg.name.clone(), constraints.clone())
                    .await?;
            }

            ConstraintStorage::unfreeze(constraints)
        };
        info!("Resolved recursively in {:?}", start.elapsed());

        // dbg!(&constraints);

        let interesing_pkgs = if !self.constraints.candidate_packages.is_empty() {
            self.constraints.candidate_packages.clone()
        } else {
            self.get_direct_deps_of_current_cargo_workspace()?
        };

        constraints.finalize().await;

        for pkg in interesing_pkgs.iter() {
            let req = constraints.get(pkg).unwrap();
            println!("{}: {}", pkg, req);
        }

        // dbg!(&interesing_pkgs);
        // dbg!(&constraints);

        Ok(Solution {})
    }

    fn get_direct_deps_of_current_cargo_workspace(&self) -> Result<Vec<PackageName>> {
        let ws = cargo_metadata::MetadataCommand::new()
            .exec()
            .context("failed to run `cargo metadata`")?;

        let ws_pkg_names = ws
            .workspace_members
            .iter()
            .map(|p| p.to_string())
            .map(PackageName::from)
            .collect::<AHashSet<_>>();

        let ws_pkgs = ws
            .packages
            .iter()
            .filter(|pkg| ws_pkg_names.contains(&pkg.name.clone().into()));

        Ok(ws_pkgs
            .flat_map(|pkg| pkg.dependencies.iter().map(|d| d.name.clone()))
            .map(PackageName::from)
            .collect())
    }
}
