use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use auto_impl::auto_impl;
use hstr::Atom;
use semver::{Version, VersionReq};

pub mod cargo;

/// All versions of a **single** package.
pub type Versions = Arc<Vec<PackageVersion>>;

pub type PackageName = Atom;

#[async_trait]
#[auto_impl(Arc, Box, &)]
pub trait PackageManager: Send + Sync {
    async fn resolve(&self, package_name: &str, constraints: &VersionReq) -> Result<Versions>;
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
    pub constraints: VersionReq,
}
