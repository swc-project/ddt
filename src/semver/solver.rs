use semver::{Version, VersionReq};

#[async_trait::async_trait]
pub trait PackageManager {
    async fn resolve(&self, package_name: &str, constraints: &VersionReq) -> Vec<Version>;
}
