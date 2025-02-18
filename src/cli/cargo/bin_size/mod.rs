use std::fmt::{self, Display};

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use humansize::{format_size, DECIMAL};
use indexmap::IndexMap;
use rustc_hash::{FxBuildHasher, FxHashMap};
use serde::Deserialize;

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
        let mut crates = IndexMap::<_, _, FxBuildHasher>::default();

        if self.compare {
            ensure_cargo_subcommand("bloat")
                .await
                .context("You can install bloat by `cargo install cargo-bloat`")?;

            let for_perf = run_bloat(&self.build_target, OptLevel::Performance).await?;
            let for_size = run_bloat(&self.build_target, OptLevel::Size).await?;

            for (opt_level, output) in [
                (OptLevel::Performance, for_perf),
                (OptLevel::Size, for_size),
            ] {
                for crate_ in output.crates {
                    let info = crates
                        .entry(crate_.name.clone())
                        .or_insert_with(|| CrateInfo {
                            name: crate_.name.clone(),
                            size: PerOptLevel::default(),
                        });

                    info.size.insert(opt_level, crate_.size);
                }
            }
        }

        // Remove from the crate list if the size is the same for all opt levels.
        crates.retain(|_, info| {
            let mut sizes = info.size.values().collect::<Vec<_>>();
            sizes.sort_unstable();
            !sizes.iter().all(|size| size == &sizes[0])
        });

        for (name, info) in crates {
            eprintln!("{}", name);
            for (opt_level, size) in info.size {
                eprintln!("  {} : {}", opt_level, format_size(size, DECIMAL));
            }
        }

        Ok(())
    }
}

async fn run_bloat(build_target: &CargoBuildTarget, opt_level: OptLevel) -> Result<BloatOutput> {
    let mut cmd = PrettyCmd::new("Running cargo bloat", "cargo");
    cmd.arg("bloat");

    cmd.arg("--crates");
    // Show all crates
    cmd.arg("-n").arg("0");

    // Ouptut in json format.
    cmd.arg("--message-format").arg("json");

    cmd.env("CARGO_PROFILE_RELEASE_DEBUG", "1");
    cmd.env("CARGO_PROFILE_RELEASE_OPT_LEVEL", opt_level.to_string());

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

    let output: BloatOutput =
        serde_json::from_str(&output).context("failed to parse bloat output")?;

    Ok(output)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct BloatOutput {
    file_size: u64,
    text_section_size: u64,

    crates: Vec<BloatCrate>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct BloatCrate {
    name: String,
    /// File size in bytes.
    size: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OptLevel {
    /// `3`
    Performance,
    /// `s`
    Size,
    /// `z`
    SizeWithLoopVec,
}

impl Display for OptLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OptLevel::Performance => write!(f, "3"),
            OptLevel::Size => write!(f, "s"),
            OptLevel::SizeWithLoopVec => write!(f, "z"),
        }
    }
}

type PerOptLevel<T> = FxHashMap<OptLevel, T>;

#[derive(Debug)]
struct CrateInfo {
    name: String,
    size: PerOptLevel<u64>,
}
