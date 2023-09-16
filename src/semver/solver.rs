use std::sync::Arc;

use ahash::AHashMap;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use auto_impl::auto_impl;
use futures::{stream::FuturesUnordered, StreamExt};
use semver::{Version, VersionReq};
use string_cache::DefaultAtom;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::util::{intersection_union::Intersect, wrap};

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

    async fn resolve_pkg_recursively(
        &self,
        name: PackageName,
        constraints: Arc<AHashMap<PackageName, VersionReq>>,
    ) -> Result<()> {
        let constraints = constraints
            .get(&name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("constraint for package `{}` does not exist", name))?;

        let constraint = PackageConstraint { name, constraints };

        let pkg = self.get_pkg(&constraint).await?;

        Ok(())
    }

    /// Resolve all packages recursively.
    async fn resolve_all_pkgs(
        &self,
        pkgs: Arc<Vec<PackageName>>,
        constraints: Arc<AHashMap<PackageName, VersionReq>>,
    ) -> Result<()> {
        wrap({
            let pkgs = pkgs.clone();

            async move {
                //
                let futures = FuturesUnordered::new();

                for p in pkgs.iter().cloned() {
                    let constraints = constraints.clone();

                    futures.push(async move { self.resolve_pkg_recursively(p, constraints).await });
                }

                let futures = futures.collect::<Vec<_>>().await;

                for f in futures {
                    f?;
                }

                Ok(())
            }
        })
        .await
        .with_context(|| format!("failed to resolve a package in the list `{:?}`", pkgs))
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
            // Merge all constraints into one, but per package.
            let mut constarints_per_pkg = AHashMap::<_, Vec<_>>::default();

            for constraint in self.constraints.compatible_packages.iter() {
                let versions = self.get_pkg(constraint).await?;

                for v in versions.iter() {
                    for dep in v.deps.iter() {
                        let e = constarints_per_pkg.entry(dep.name.clone()).or_default();

                        e.push(dep.range.clone());
                    }
                }
            }

            // We now iterate over the merged constraints (again, per package) and combine
            // them to one per a package.
            let mut merged_constraints = AHashMap::<_, VersionReq>::default();

            for (pkg_name, constraints) in constarints_per_pkg.into_iter() {
                let mut merged = VersionReq::STAR;

                for c in constraints.into_iter() {
                    merged = merged.intersect(c).or_else(|_| {
                        bail!(
                            "failed to select a version of {} due to conflicting requirements",
                            pkg_name
                        )
                    })?;
                }

                merged_constraints.insert(pkg_name.clone(), merged);
            }

            dbg!(&merged_constraints);
            Arc::new(merged_constraints)
        };

        // Now we have optimal constraints per each package.
        // We now fetch all

        let pkgs = self
            .constraints
            .candidate_packages
            .iter()
            .cloned()
            .chain(
                self.constraints
                    .compatible_packages
                    .iter()
                    .map(|v| v.name.clone()),
            )
            .collect::<Vec<_>>();

        self.resolve_all_pkgs(Arc::new(pkgs), constraints.clone())
            .await?;

        Ok(Solution {})
    }
}
