use std::{
    borrow::Borrow,
    fmt::{Display, Formatter},
    ops::Deref,
    str::FromStr,
    sync::Arc,
    time::Instant,
};

use ahash::AHashSet;
use anyhow::{Context, Result};
use auto_impl::auto_impl;
use pubgrub::{
    range::Range,
    solver::{choose_package_with_fewest_versions, resolve, DependencyProvider},
};
use semver::{Version, VersionReq};
use tracing::info;

use super::PackageName;

#[auto_impl(Arc, Box, &)]
pub trait PackageManager: Send + Sync {
    fn resolve(&self, package_name: &str, constraints: &Range<Semver>) -> Result<Vec<PackageInfo>>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constraints {
    /// Only packages with these names will be considered.
    pub candidate_packages: Vec<PackageName>,

    /// These packages must be included in the solution.
    pub compatible_packages: Vec<PackageConstraint>,
}

#[derive(Debug, Clone)]
pub struct Solution {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageConstraint {
    pub name: PackageName,
    pub range: Range<Semver>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Semver(Version);

impl Semver {
    pub(crate) fn parse_range(requirement: &str) -> Result<Range<Self>> {
        let version_req = VersionReq::parse(requirement)
            .with_context(|| format!("failed to parse version requirement `{}`", requirement))?;

        let mut range = Range::full();

        for c in version_req.comparators {
            use pubgrub::version::Version;

            let ver = semver::Version::new(
                c.major,
                c.minor.unwrap_or_default(),
                c.patch.unwrap_or_default(),
            );

            let new_range = match c.op {
                semver::Op::Exact => Range::singleton(Self(ver)),
                semver::Op::Greater => Range::higher_than(Self(ver)),
                semver::Op::GreaterEq => Range::strictly_lower_than(Self(ver)).complement(),
                semver::Op::Less => Range::strictly_lower_than(Self(ver)),
                semver::Op::LessEq => Range::strictly_lower_than(Self(ver).bump()),
                semver::Op::Tilde => {
                    let mut with_minor_bump = ver.clone();
                    with_minor_bump.minor += 1;

                    //
                    Range::higher_than(Self(ver))
                        .intersection(&Range::strictly_lower_than(Self(with_minor_bump)))
                }
                semver::Op::Caret => Range::higher_than(Self(ver)),
                semver::Op::Wildcard => Range::full(),
                _ => {
                    unimplemented!("{:?}", c.op)
                }
            };
            range = range.intersection(&new_range);
        }

        Ok(range)
    }
}

impl Deref for Semver {
    type Target = Version;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Semver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl FromStr for Semver {
    type Err = <Version as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Version::parse(s)?))
    }
}

impl pubgrub::version::Version for Semver {
    fn lowest() -> Self {
        Self(Version::new(0, 0, 0))
    }

    fn bump(&self) -> Self {
        let mut new = self.clone();
        new.0.patch += 1;
        new
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageInfo {
    pub name: PackageName,
    pub version: Semver,
    pub deps: Vec<PackageConstraint>,
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
    #[tracing::instrument(skip_all)]
    async fn solve(&self) -> Result<Solution> {
        self.solver_inner().await
    }

    async fn solver_inner(&self) -> Result<Solution> {
        info!("Solving versions using Solver");

        let start = Instant::now();
        let solution = resolve(
            &PkgMgr {
                inner: self.pkg_mgr.clone(),
                root_deps: self.constraints.clone(),
            },
            "@@root".into(),
            "0.0.0".parse::<Semver>().unwrap(),
        )
        .unwrap();
        info!("Resolved recursively in {:?}", start.elapsed());

        // dbg!(&constraints);

        let interesing_pkgs = if !self.constraints.candidate_packages.is_empty() {
            self.constraints.candidate_packages.clone()
        } else {
            self.get_direct_deps_of_current_cargo_workspace()?
        };

        dbg!(&interesing_pkgs);

        dbg!(&solution);

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

struct PkgMgr {
    inner: Arc<dyn PackageManager>,
    root_deps: Arc<Constraints>,
}

impl DependencyProvider<PackageName, Range<Semver>> for PkgMgr {
    /// Pub chooses the latest matching version of the package with the fewest
    /// versions that match the outstanding constraint. This tends to find
    /// conflicts earlier if any exist, since these packages will run out of
    /// versions to try more quickly. But there's likely room for improvement in
    /// these heuristics.
    fn choose_package_version<T: Borrow<PackageName>, U: Borrow<Range<Semver>>>(
        &self,
        potential_packages: impl Iterator<Item = (T, U)>,
    ) -> std::result::Result<
        (
            T,
            Option<<Range<Semver> as pubgrub::version_set::VersionSet>::V>,
        ),
        Box<dyn std::error::Error + Send + Sync>,
    > {
        Ok(choose_package_with_fewest_versions(
            |name: &PackageName| {
                if name == "@@root" {
                    return vec!["0.0.0".parse().unwrap()].into_iter();
                };

                let versions = self
                    .inner
                    .resolve(name.borrow(), &Range::full())
                    .unwrap_or_else(|_| Default::default());

                versions
                    .into_iter()
                    .map(|v| v.version)
                    .collect::<Vec<_>>()
                    .into_iter()
            },
            potential_packages,
        ))
    }

    fn get_dependencies(
        &self,
        package: &PackageName,
        version: &<Range<Semver> as pubgrub::version_set::VersionSet>::V,
    ) -> std::result::Result<
        pubgrub::solver::Dependencies<PackageName, Range<Semver>>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        if package == "@@root" {
            return Ok(pubgrub::solver::Dependencies::Known(
                self.root_deps
                    .compatible_packages
                    .iter()
                    .map(|c| (c.name.clone(), c.range.clone()))
                    .collect(),
            ));
        }

        let pkg = self
            .inner
            .resolve(&package, &Range::singleton(version.clone()))?;

        if pkg.is_empty() {
            return Ok(pubgrub::solver::Dependencies::Unknown);
        }

        let map = pkg[0]
            .deps
            .iter()
            .map(|pkg| (PackageName::from(pkg.name.clone()), pkg.range.clone()))
            .collect::<pubgrub::type_aliases::Map<_, _>>();

        Ok(pubgrub::solver::Dependencies::Known(map))
    }
}
