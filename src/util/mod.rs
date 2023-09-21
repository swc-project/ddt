use std::{ffi::OsStr, future::Future};

use anyhow::Result;
use tokio::process::Command;
use tracing::info;

pub mod intersection_union;

/// Type annotation for [anyhow::Result]
pub async fn wrap<Fut, Ret>(op: Fut) -> Result<Ret>
where
    Fut: Future<Output = Result<Ret>>,
{
    op.await
}

pub(crate) struct PrettyCmd {
    description: String,
    inner: Command,
}

impl PrettyCmd {
    pub fn new(description: String, command: String) -> Self {
        let mut c = Command::new(command);
        c.kill_on_drop(true);
        Self {
            description,
            inner: c,
        }
    }

    pub fn arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self {
        self.inner.arg(arg);
        self
    }

    pub async fn exec(&mut self) -> Result<()> {
        info!("Running: {}\n{:?}", self.description, self.inner);

        let status = self.inner.status().await?;

        if status.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("{} failed", self.description))
        }
    }
}
