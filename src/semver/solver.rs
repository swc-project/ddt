use semver::{Version, VersionReq};
use string_cache::DefaultAtom;

#[async_trait::async_trait]
pub trait PackageManager {
    async fn resolve(&self, package_name: &str, constraints: &VersionReq) -> Vec<Version>;
}

pub type PackageName = DefaultAtom;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Constraints {
    pub candidate_packages: Vec<PackageName>,

    pub compatible_packages: Vec<PackageConstraint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageConstraint {
    pub name: PackageName,
    pub constraints: VersionReq,
}
