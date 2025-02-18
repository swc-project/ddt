use std::{ffi::OsStr, fmt::Display, future::Future, path::Path, process::Stdio};

use anyhow::{Context, Result};
use tokio::process::Command;
use tracing::info;

pub mod cargo_build;

/// Type annotation for [anyhow::Result]
pub async fn wrap<Fut, Ret>(op: Fut) -> Result<Ret>
where
    Fut: Future<Output = Result<Ret>>,
{
    op.await
}

pub async fn ensure_bin_exists(name: &str) -> Result<()> {
    if which::which(name).is_err() {
        Err(anyhow::anyhow!("{} is not installed", name))
    } else {
        Ok(())
    }
}

pub async fn ensure_cargo_subcommand(name: &str) -> Result<()> {
    ensure_bin_exists(&format!("cargo-{}", name)).await
}

pub(crate) struct PrettyCmd {
    description: String,
    inner: Command,
}

impl PrettyCmd {
    pub fn new(description: impl Display, command: impl AsRef<str>) -> Self {
        let mut c = Command::new(command.as_ref());
        c.kill_on_drop(true);
        Self {
            description: description.to_string(),
            inner: c,
        }
    }

    pub fn arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self {
        self.inner.arg(arg);
        self
    }

    pub fn args<E>(&mut self, arg: impl IntoIterator<Item = E>) -> &mut Self
    where
        E: AsRef<OsStr>,
    {
        self.inner.args(arg);
        self
    }

    pub fn dir(&mut self, dir: impl AsRef<Path>) -> &mut Self {
        self.inner.current_dir(dir);
        self
    }

    pub fn env(&mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> &mut Self {
        self.inner.env(key, value);
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

    pub async fn output(&mut self) -> Result<String> {
        info!("Running: {}\n{:?}", self.description, self.inner);

        let output = self.inner.stderr(Stdio::inherit()).output().await?;

        if output.status.success() {
            String::from_utf8(output.stdout).context("failed to parse output as utf-8")
        } else {
            Err(anyhow::anyhow!("{} failed", self.description))
        }
    }
}
