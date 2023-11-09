use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use semver::VersionReq;

use super::{Dependency, PackageManager, PackageVersion, Versions};

#[derive(Debug, Default)]
pub struct CargoPackageManager;

#[async_trait]
impl PackageManager for CargoPackageManager {
    async fn resolve(&self, package_name: &str, constraints: &VersionReq) -> Result<Versions> {
        if package_name == "std" || package_name == "core" {
            return Ok(Arc::new(vec![PackageVersion {
                name: package_name.into(),
                version: "1.0.0".parse().unwrap(),
                deps: Default::default(),
            }]));
        }

        let body = reqwest::get("https://www.rust-lang.org")
            .await?
            .text()
            .await?;
    }
}

fn build_url(name: &str) -> String {
    match name.len() {
        1 => format!("https://index.crates.io/1/{name}"),
        2 => format!("https://index.crates.io/2/{name}"),
        3 => {
            let first_char = name.chars().next().unwrap();
            format!("https://index.crates.io/3/{first_char}/{name}")
        }
        4 => {
            let first_two = &name[0..2];
            let second_two = &name[2..4];

            format!("https://index.crates.io/4/{first_two}/{second_two}/{name}",)
        }
    }
}
