use anyhow::{Context, Result};
use clap::{Args, Subcommand};

use crate::util::{cargo_build::CargoBuildTarget, ensure_cargo_subcommand, PrettyCmd};

/// Comamnds to reduce the size of the binary.
#[derive(Debug, Args)]
pub(super) struct BinSizeCommand {
    #[clap(subcommand)]
    cmd: Cmd,
}

impl BinSizeCommand {
    pub async fn run(self) -> Result<()> {
        match self.cmd {
            Cmd::SelectPerCrate(cmd) => cmd.run().await,
        }
    }
}

#[derive(Debug, Subcommand)]
enum Cmd {
    SelectPerCrate(SelectPerCrateCommand),
}

/// Select the optimization level for each crate.
#[derive(Debug, Args)]
struct SelectPerCrateCommand {
    #[clap(long)]
    compare: bool,

    #[clap(flatten)]
    build_target: CargoBuildTarget,
}

impl SelectPerCrateCommand {
    pub async fn run(self) -> Result<()> {
        ensure_cargo_subcommand("bloat")
            .await
            .context("You can install bloat by `cargo install cargo-bloat`")?;

        if self.compare {
            let for_perf = run_bloat(&self.build_target, "3").await?;
            let for_size = run_bloat(&self.build_target, "s").await?;
        }

        Ok(())
    }
}

async fn run_bloat(build_target: &CargoBuildTarget, opt_level: &str) -> Result<()> {
    let mut cmd = PrettyCmd::new("Running cargo bloat", "cargo");
    cmd.arg("bloat");

    cmd.arg("--crates");
    // Show all crates
    cmd.arg("-n").arg("0");

    cmd.env("CARGO_PROFILE_RELEASE_DEBUG", "1");
    cmd.env("CARGO_PROFILE_RELEASE_OPT_LEVEL", opt_level);

    if build_target.release {
        cmd.arg("--release");
    }

    if build_target.lib {
        cmd.arg("--lib");
    }

    if build_target.bin.is_some() {
        cmd.arg("--bin").arg(build_target.bin.as_ref().unwrap());
    }

    if build_target.benches {
        cmd.arg("--benches");
    }

    if let Some(bench) = &build_target.bench {
        cmd.arg("--bench").arg(bench);
    }

    if build_target.tests {
        cmd.arg("--tests");
    }

    if let Some(test) = &build_target.test {
        cmd.arg("--test").arg(test);
    }

    if build_target.examples {
        cmd.arg("--examples");
    }

    if let Some(example) = &build_target.example {
        cmd.arg("--example").arg(example);
    }

    if let Some(features) = &build_target.features {
        cmd.arg("--features").arg(features.join(","));
    }

    if let Some(profile) = &build_target.profile {
        cmd.arg("--profile").arg(profile);
    }

    let output = cmd.output().await.context("failed to run cargo bloat")?;

    eprintln!("Output:\n{}", output);

    Ok(())
}
