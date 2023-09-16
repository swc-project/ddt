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

    pub fn new(parent: Arc<ConstraintStorage>) -> Self {
        Self {
            cur: Default::default(),
            parent: Some(parent),
        }
    }

    pub(crate) fn freeze(self) -> Arc<ConstraintStorage> {
        Arc::new(self)
    }
}

impl ConstraintStorage {
    pub fn get(&self, name: &PackageName) -> Option<&VersionReq> {
        self.cur
            .get(name)
            .or_else(|| self.parent.as_ref().and_then(|p| p.get(name)))
    }

    pub fn insert(&mut self, name: PackageName, constraints: VersionReq) {
        // TODO: Intersect
        self.cur.insert(name, constraints);
    }
}
