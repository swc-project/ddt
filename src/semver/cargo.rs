use anyhow::{anyhow, Result};
use crates_index::DependencyKind;
use pubgrub::range::Range;

use super::solver::{PackageConstraint, PackageInfo, PackageManager, Semver};

pub struct CargoPackageManager {
    pub index: crates_index::GitIndex,

    pub target_repo: Option<String>,

    pub metadata: cargo_metadata::Metadata,
}

impl CargoPackageManager {
    fn is_interesting(&self, pkg: &str) -> bool {
        self.metadata
            .packages
            .iter()
            .find(|p| p.name == pkg)
            .is_some()
    }
}

impl PackageManager for CargoPackageManager {
    fn resolve(&self, package_name: &str, range: &Range<Semver>) -> Result<Vec<PackageInfo>> {
        if package_name == "std" || package_name == "core" {
            return Ok(vec![PackageInfo {
                name: package_name.into(),
                version: "1.0.0".parse().unwrap(),
                deps: Default::default(),
            }]);
        }

        let pkg = self
            .index
            .crate_(package_name)
            .ok_or_else(|| anyhow!("Package `{}@{}` not found in index", package_name, range))?;

        Ok(pkg
            .versions()
            .iter()
            .filter_map(|v| {
                let ver = v.version().parse::<Semver>().expect("invalid version");

                if !range.contains(&ver) {
                    return None;
                }
                Some((
                    ver,
                    v.dependencies()
                        .into_iter()
                        .filter(|dep| dep.kind() == DependencyKind::Normal)
                        .filter(|dep| self.is_interesting(dep.crate_name()))
                        .collect::<Vec<_>>(),
                ))
            })
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
