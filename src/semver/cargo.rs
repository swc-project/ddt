use anyhow::{anyhow, Result};
use async_trait::async_trait;
use semver::VersionReq;

use super::solver::{Dependency, PackageManager, PackageVersion};

#[derive(Debug, Default)]
pub struct CargoPackageManager;

#[async_trait]
impl PackageManager for CargoPackageManager {
    async fn resolve(
        &self,
        package_name: &str,
        constraints: &VersionReq,
    ) -> Result<Vec<PackageVersion>> {
        if package_name == "std" || package_name == "core" {
            return Ok(vec![PackageVersion {
                name: package_name.into(),
                version: "1.0.0".parse().unwrap(),
                deps: Default::default(),
            }]);
        }

        let index = crates_index::GitIndex::new_cargo_default()?;
        let pkg = index.crate_(package_name).ok_or_else(|| {
            anyhow!(
                "Package `{}@{}` not found in index",
                package_name,
                constraints
            )
        })?;

        Ok(pkg
            .versions()
            .iter()
            .map(|v| {
                let ver = v.version().parse().expect("invalid version");

                (ver, v.dependencies().to_vec())
            })
            .filter(|(v, _)| constraints.matches(v))
            .map(|(ver, deps)| {
                let deps = deps
                    .iter()
                    .map(|d| Dependency {
                        name: d.name().into(),
                        range: d
                            .requirement()
                            .parse()
                            .expect("invalid version requirenment"),
                    })
                    .collect();

                PackageVersion {
                    name: package_name.into(),
                    version: ver,
                    deps,
                }
            })
            .collect())
    }
}
