use anyhow::{anyhow, Result};
use crates_index::DependencyKind;
use pubgrub::range::Range;

use super::solver::{PackageConstraint, PackageInfo, PackageManager, Semver};

#[derive(Debug, Default)]
pub struct CargoPackageManager;

impl PackageManager for CargoPackageManager {
    fn resolve(&self, package_name: &str, range: &Range<Semver>) -> Result<Vec<PackageInfo>> {
        if package_name == "std" || package_name == "core" {
            return Ok(vec![PackageInfo {
                name: package_name.into(),
                version: "1.0.0".parse().unwrap(),
                deps: Default::default(),
            }]);
        }

        let index = crates_index::GitIndex::new_cargo_default()?;
        let pkg = index
            .crate_(package_name)
            .ok_or_else(|| anyhow!("Package `{}@{}` not found in index", package_name, range))?;

        Ok(pkg
            .versions()
            .iter()
            .map(|v| {
                let ver = v.version().parse::<Semver>().expect("invalid version");

                (
                    ver,
                    v.dependencies()
                        .into_iter()
                        .filter(|dep| dep.kind() == DependencyKind::Normal)
                        .collect::<Vec<_>>(),
                )
            })
            .filter(|(v, _)| range.contains(v))
            .map(|(ver, deps)| {
                let deps = deps
                    .iter()
                    .map(|d| PackageConstraint {
                        name: d.crate_name().into(),
                        range: Semver::parse_range(d.requirement())
                            .expect("invalid version requirenment"),
                    })
                    .collect();

                PackageInfo {
                    name: package_name.into(),
                    version: ver,
                    deps,
                }
            })
            .collect())
    }
}
