use auto_impl::auto_impl;
use semver::{Version, VersionReq};
use string_cache::DefaultAtom;

#[async_trait::async_trait]
#[auto_impl(Arc, Box, &)]
pub trait PackageManager {
    async fn resolve(&self, package_name: &str, constraints: &VersionReq) -> Vec<Version>;
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

pub async fn solve() {}
