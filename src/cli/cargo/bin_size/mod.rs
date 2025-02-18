use std::fmt::{self, Display};

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use hstr::Atom;
use humansize::{format_size, DECIMAL};
use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;
use serde::Deserialize;
use toml_edit::{table, value, DocumentMut};

use crate::{
    cli::util::cargo::to_original_crate_name,
    util::{
        cargo_build::{cargo_root_manifest, CargoBuildTarget},
        ensure_cargo_subcommand, PrettyCmd,
    },
};

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
        let root_cargo_toml_path =
            cargo_root_manifest().context("failed to get the root cargo.toml")?;
        let root_content = std::fs::read_to_string(&root_cargo_toml_path)
            .context("failed to read the root cargo.toml")?;

        let mut toml = root_content
            .parse::<DocumentMut>()
            .context("failed to parse the root cargo.toml")?;

        let profile_name = self.build_target.profile.as_deref().unwrap_or("release");

        if !toml["profile"].is_table() {
            toml["profile"] = table();
        }

        if !toml["profile"]
            .as_table_mut()
            .unwrap()
            .contains_key(profile_name)
        {
            toml["profile"][profile_name] = table();
        }

        if !toml["profile"][profile_name].is_table() {
            toml["profile"][profile_name] = table();
        }

        if !toml["profile"][profile_name]
            .as_table_mut()
            .unwrap()
            .contains_key("package")
        {
            toml["profile"][profile_name]["package"] = table();
        }

        let package_table = toml["profile"][profile_name]["package"]
            .as_table_mut()
            .context("failed to get the package table")?;

        let mut crates = IndexMap::<Atom, _, FxBuildHasher>::default();

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
                            size: PerOptLevel::default(),
                        });

                    info.size.insert(opt_level, crate_.size);
                }
            }
        }

        // Remove from the crate list if it's already in the package table.
        crates.retain(|name, _| {
            let Ok(name) = to_original_crate_name(name.clone()) else {
                return true;
            };

            !package_table.contains_key(&name)
        });

        // Remove from the crate list if the size is the same for all opt levels.
        crates.retain(|_, info| {
            let mut sizes = info.size.values().collect::<Vec<_>>();
            sizes.sort_unstable();
            !sizes.iter().all(|size| size == &sizes[0])
        });

        for (name, info) in crates {
            let Ok(name) = to_original_crate_name(name) else {
                continue;
            };

            let selected = dialoguer::Select::new()
                .with_prompt(format!(
                    "Select the optimization level for {} (Esc to skip)",
                    name
                ))
                .items(
                    &info
                        .size
                        .iter()
                        .map(|(k, v)| format!("{}: {}", k, format_size(*v, DECIMAL)))
                        .collect::<Vec<_>>(),
                )
                .interact_opt()
                .context("failed to select the optimization level")?;

            if let Some(selected) = selected {
                let (selected_opt_level, _) = info.size.get_index(selected).unwrap();

                let mut t = table();
                {
                    t.as_table_mut().unwrap().insert(
                        "opt-level",
                        match selected_opt_level {
                            OptLevel::Performance => value(3),
                            OptLevel::Size => value("s"),
                            OptLevel::SizeWithLoopVec => value("z"),
                        },
                    );
                }

                package_table[&*name] = t;
            }
        }

        std::fs::write(root_cargo_toml_path, toml.to_string())
            .context("failed to write the root cargo.toml")?;

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
    // file_size: u64,
    // text_section_size: u64,
    crates: Vec<BloatCrate>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct BloatCrate {
    name: Atom,
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
    #[allow(unused)]
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

type PerOptLevel<T> = IndexMap<OptLevel, T, FxBuildHasher>;

#[derive(Debug)]
struct CrateInfo {
    size: PerOptLevel<u64>,
}
