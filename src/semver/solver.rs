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
    solver::{resolve, DependencyProvider},
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

        let mut range = Range::any();

        for c in version_req.comparators {
            use pubgrub::version::Version;

            let ver = semver::Version::new(
                c.major,
                c.minor.unwrap_or_default(),
                c.patch.unwrap_or_default(),
            );

            let new_range = match c.op {
                semver::Op::Exact => Range::exact(Self(ver)),
                semver::Op::Greater => Range::higher_than(Self(ver)),
                semver::Op::GreaterEq => Range::strictly_lower_than(Self(ver)).negate(),
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
                semver::Op::Wildcard => Range::any(),
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

impl DependencyProvider<PackageName, Semver> for PkgMgr {
    fn choose_package_version<T: Borrow<PackageName>, U: Borrow<Range<Semver>>>(
        &self,
        potential_packages: impl Iterator<Item = (T, U)>,
    ) -> std::result::Result<(T, Option<Semver>), Box<dyn std::error::Error>> {
        let mut highest = None;

        let potential_packages = potential_packages.collect::<Vec<_>>();

        let _tracing = tracing::span!(tracing::Level::TRACE, "choose_package_version").entered();

        for (pkg, range) in potential_packages {
            let name: &PackageName = pkg.borrow();
            let parsed_range: &Range<Semver> = range.borrow();

            info!(%name, %parsed_range, "Resolving package");

            let mut versions = if name == "@@root" {
                vec![PackageInfo {
                    name: name.clone(),
                    version: "0.0.0".parse().unwrap(),
                    deps: Default::default(),
                }]
            } else {
                self.inner.resolve(name, parsed_range)?
            };

            versions.sort_by_cached_key(|v| v.version.clone());

            let new_highest = versions.into_iter().max_by_key(|info| info.version.clone());

            if let Some(info) = new_highest {
                match &mut highest {
                    Some((_, Some(highest_version))) => {
                        if info.version > *highest_version {
                            *highest_version = info.version.clone();
                        }
                    }
                    _ => {
                        highest = Some((pkg, Some(info.version.clone())));
                    }
                }
            }
        }

        match highest {
            Some(v) => Ok(v),
            None => Err(anyhow::anyhow!("package does not exist"))?,
        }
    }

    fn get_dependencies(
        &self,
        package: &PackageName,
        version: &Semver,
    ) -> std::result::Result<
        pubgrub::solver::Dependencies<PackageName, Semver>,
        Box<dyn std::error::Error>,
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
            .resolve(&package, &Range::exact(version.clone()))?;

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
