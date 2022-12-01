use std::future::Future;

use anyhow::Result;

/// Type annotation for [anyhow::Result]
pub async fn wrap<Fut, Ret>(op: Fut) -> Result<Ret>
where
    Fut: Future<Output = Result<Ret>>,
{
    op.await
}
