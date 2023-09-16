use std::sync::Arc;

use ahash::AHashMap;
use semver::VersionReq;

use super::PackageName;

pub(crate) type ConstraintsPerPkg = AHashMap<PackageName, VersionReq>;

#[derive(Debug)]
pub(crate) struct ConstraintStorage {
    cur: ConstraintsPerPkg,
    parent: Option<Arc<ConstraintStorage>>,
}

impl ConstraintStorage {
    pub fn root() -> Self {
        Self {
            cur: Default::default(),
            parent: Default::default(),
        }
    }

    pub fn new(cur: ConstraintsPerPkg, parent: Arc<ConstraintStorage>) -> Self {
        Self {
            cur,
            parent: Some(parent),
        }
    }
}
