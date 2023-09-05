use semver::{Version, VersionReq};

#[async_trait::async_trait]
pub trait PackageManager {
    async fn resolve(&self, package_name: &str, constraints: &VersionReq) -> Vec<Version>;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Constraints {
    pub compatible_packages: Vec<PackageConstraint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageConstraint(pub String, pub VersionReq);
