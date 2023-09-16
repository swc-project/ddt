use std::sync::Arc;

use ahash::AHashMap;
use semver::VersionReq;
use tokio::sync::RwLock;

use super::PackageName;

pub(crate) type ConstraintsPerPkg = AHashMap<PackageName, VersionReq>;

#[derive(Debug)]
pub(crate) struct ConstraintStorage {
    actual: Arc<RwLock<ConstraintsPerPkg>>,
    parent: Option<Arc<ConstraintStorage>>,
}
