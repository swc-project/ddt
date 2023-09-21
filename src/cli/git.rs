use anyhow::{bail, Context, Result};
use clap::{Args, Subcommand};
use tokio::fs;

use crate::util::{wrap, PrettyCmd};

/// Extra commands like auto-completion or self-update.
#[derive(Debug, Args)]
pub struct GitCommand {
    #[clap(subcommand)]
    cmd: Inner,
}

impl GitCommand {
    pub async fn run(self) -> Result<()> {
        match self.cmd {
            Inner::ResolveLockfileConflict(cmd) => cmd.run().await,
        }
    }
}

#[derive(Debug, Subcommand)]
enum Inner {
    ResolveLockfileConflict(ResolveLockfileConflictCommand),
}

/// Resolve merge conflicts in the lockfile.
///
/// Note that this command do **not** understand the lockfile.
/// This command simply ignores the conflict and runs some command which can
/// generate the lockfile.
#[derive(Debug, Args)]
struct ResolveLockfileConflictCommand {
    args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LockfileType {
    Pnpm,
    Yarn,
    Npm,
    Cargo,
}
impl LockfileType {
    pub fn from_suffix(s: &str) -> Result<Self> {
        if s.ends_with("pnpm-lock.yaml") {
            return Ok(Self::Pnpm);
        }

        if s.ends_with("yarn.lock") {
            return Ok(Self::Yarn);
        }

        if s.ends_with("package-lock.json") {
            return Ok(Self::Npm);
        }

        if s.ends_with("Cargo.lock") {
            return Ok(Self::Cargo);
        }

        bail!("unknown lockfile type: `{}`", s)
    }
}

impl ResolveLockfileConflictCommand {
    pub async fn run(self) -> Result<()> {
        wrap(async move {
            if self.args.len() != 5 {
                bail!(
                    "The ddt-lockfile merge driver expects 5 arguments. Please ensure that you \
                     configured git driver properly. It should be


                    driver = ddt git resolve-lockfile-conflict %O %A %B %L %P
                     "
                )
            }

            let ancestor_path = &self.args[0];
            let a_path = &self.args[1];
            let b_path = &self.args[2];
            let file_name = &self.args[4];

            let lockfile_type = LockfileType::from_suffix(file_name)?;

            fs::remove_file(a_path)
                .await
                .context("failed to remove `a`")?;

            fs::remove_file(b_path)
                .await
                .context("failed to remove `b`")?;

            fs::rename(ancestor_path, a_path)
                .await
                .context("failed to rename")?;

            match lockfile_type {
                LockfileType::Pnpm => {
                    let mut cmd = PrettyCmd::new("pnpm install".to_string(), "pnpm".to_string());
                    cmd.arg("install");
                    cmd.exec().await?;
                }
                LockfileType::Yarn => {
                    let mut cmd = PrettyCmd::new("yarn install".to_string(), "yarn".to_string());
                    cmd.exec().await?;
                }
                LockfileType::Npm => {
                    let mut cmd = PrettyCmd::new("npm ci".to_string(), "npm".to_string());
                    cmd.arg("ci");
                    cmd.exec().await?;
                }
                LockfileType::Cargo => {
                    bail!("cargo check? not sure")
                }
            }

            Ok(())
        })
        .await
        .context("failed to resolve lockfile conflict")
    }
}
