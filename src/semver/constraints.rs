use std::sync::Arc;

use ahash::AHashMap;
use async_recursion::async_recursion;
use semver::VersionReq;
use tokio::sync::RwLock;

use crate::package_manager::PackageName;

pub(crate) type ConstraintsPerPkg = AHashMap<PackageName, VersionReq>;

#[derive(Debug)]
pub(crate) struct ConstraintStorage {
    cur: ConstraintsPerPkg,
    parent: Option<Arc<ConstraintStorage>>,

    children: RwLock<Vec<Arc<ConstraintStorage>>>,
}

impl ConstraintStorage {
    pub fn root() -> Self {
        Self {
            cur: Default::default(),
            parent: Default::default(),
            children: Default::default(),
        }
    }

    pub fn new(parent: Arc<ConstraintStorage>) -> Self {
        Self {
            cur: Default::default(),
            parent: Some(parent),
            children: Default::default(),
        }
    }

    pub(crate) fn freeze(self) -> Arc<ConstraintStorage> {
        Arc::new(self)
    }

    pub(crate) async fn remove_parent(mut self) {
        let parent = self.parent.take();
        if let Some(parent) = parent {
            parent.children.write().await.push(self.freeze());
        }
    }

    pub(crate) fn unfreeze(dep_constraints: Arc<Self>) -> Self {
        Arc::try_unwrap(dep_constraints).expect("failed to unfreeze constraint storage")
    }

    #[async_recursion]
    pub(super) async fn finalize(&mut self) {
        for c in self.children.write().await.drain(..) {
            let mut c = Self::unfreeze(c);

            c.finalize().await;

            for (name, constraints) in c.cur.drain() {
                self.cur.insert(name, constraints);
            }
        }
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
