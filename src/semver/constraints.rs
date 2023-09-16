use std::sync::Arc;

use ahash::AHashMap;
use semver::VersionReq;

use super::PackageName;

pub(crate) type ConstraintsPerPkg = AHashMap<PackageName, VersionReq>;

#[derive(Debug)]
pub(crate) struct ConstraintStorage {
    actual: ConstraintsPerPkg,
    parent: Arc<ConstraintStorageParent>,
}

#[derive(Debug, Default)]
struct ConstraintStorageParent {
    cur: Arc<ConstraintsPerPkg>,
    parent: Option<Arc<ConstraintStorageParent>>,
}

impl ConstraintStorage {
    pub fn root() -> Self {
        Self {
            actual: Default::default(),
            parent: Default::default(),
        }
    }

    pub fn new(actual: ConstraintsPerPkg, parent: ConstraintStorage) -> Self {
        Self {
            actual: Default::default(),
            parent: Arc::new(ConstraintStorageParent {
                cur: Arc::new(actual),
                parent: Some(Arc::new(ConstraintStorageParent {
                    cur: Arc::new(parent.actual),
                    parent: Some(parent.parent),
                })),
            }),
        }
    }
}
