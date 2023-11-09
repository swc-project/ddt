use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use semver::{Version, VersionReq};
use serde::Deserialize;

use super::{Dependency, PackageManager, PackageName, PackageVersion, Versions};

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

        let body = reqwest::get(&build_url(package_name)).await?.text().await?;

        let mut v = body
            .lines()
            .into_iter()
            .filter_map(|line| {
                let desc = serde_json::from_str::<Descriptor>(&line);
                let line = match desc {
                    Ok(v) => v,
                    Err(err) => {
                        return Some(Err(anyhow!("failed to parse line: {:?}\n{}", err, line)))
                    }
                };

                if !constraints.matches(&line.vers) {
                    return None;
                }

                Some(Ok(PackageVersion {
                    name: line.name,
                    version: line.vers,
                    deps: line
                        .deps
                        .into_iter()
                        .filter(|dep| dep.kind == "normal")
                        .map(|d| Dependency {
                            name: d.package.unwrap_or(d.name),
                            constraints: d.req,
                        })
                        .collect(),
                }))
            })
            .collect::<Result<Vec<_>>>()
            .with_context(|| format!("failed to parse index of {}", package_name))?;

        v.sort_by(|a, b| (b.version).cmp(&a.version));

        Ok(Arc::new(v))
    }
}

fn build_url(name: &str) -> String {
    let name = name.to_ascii_lowercase();
    match name.len() {
        1 => format!("https://index.crates.io/1/{name}"),
        2 => format!("https://index.crates.io/2/{name}"),
        3 => {
            let first_char = name.chars().next().unwrap();
            format!("https://index.crates.io/3/{first_char}/{name}")
        }
        _ => {
            let first_two = &name[0..2];
            let second_two = &name[2..4];

            format!("https://index.crates.io/{first_two}/{second_two}/{name}",)
        }
    }
}

#[derive(Debug, Deserialize)]
struct Descriptor {
    pub name: PackageName,
    pub vers: Version,
    pub deps: Vec<DepDescriptor>,
}

#[derive(Debug, Deserialize)]
struct DepDescriptor {
    pub name: PackageName,
    pub req: VersionReq,
    pub kind: String,
    #[serde(default)]
    pub package: Option<PackageName>,
}
