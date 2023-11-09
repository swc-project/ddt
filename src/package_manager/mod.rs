use std::{str::FromStr, sync::Arc};

use anyhow::{Context, Result};
use async_trait::async_trait;
use auto_impl::auto_impl;
use hstr::Atom;
use semver::{Version, VersionReq};
use serde::Serialize;

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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Dependency {
    pub name: PackageName,
    pub constraints: VersionReq,
}

impl FromStr for Dependency {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts = s.splitn(2, '@');

        let mut parts = parts.map(|s| s.trim());

        let name = parts.next().unwrap().into();
        let constraints = parts
            .next()
            .map(|s| {
                s.parse::<VersionReq>().with_context(|| {
                    format!("failed to parse version constraints (`{s}`) of {}", name)
                })
            })
            .transpose()?;

        Ok(Self {
            constraints: constraints
                .unwrap_or_else(|| panic!("failed to parse constraints of {}", name)),
            name,
        })
    }
}
