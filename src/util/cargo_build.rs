use std::{
    env,
    io::BufReader,
    path::PathBuf,
    process::{Command, Stdio},
    sync::Arc,
};

use anyhow::{bail, Context, Result};
use cached::proc_macro::cached;
use cargo_metadata::{ArtifactProfile, Message};
use clap::Parser;
use is_executable::IsExecutable;
use tracing::info;

/// Built bin file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BinFile {
    pub path: PathBuf,
    /// `.dSYM`,
    pub extra_files: Vec<PathBuf>,
    pub profile: ArtifactProfile,

    pub crate_name: String,
    pub manifest_path: PathBuf,
}

#[derive(Debug, Clone, Parser)]
pub struct CargoBuildTarget {
    #[clap(long)]
    pub lib: bool,

    #[clap(long)]
    pub release: bool,

    #[clap(long)]
    pub bin: Option<String>,

    #[clap(long)]
    pub bench: Option<String>,

    #[clap(long)]
    pub benches: bool,

    #[clap(long)]
    pub test: Option<String>,

    #[clap(long)]
    pub tests: bool,

    #[clap(long)]
    pub example: Option<String>,

    #[clap(long)]
    pub examples: bool,

    #[clap(long)]
    pub features: Option<Vec<String>>,

    #[clap(long = "package", short = 'p')]
    pub packages: Vec<String>,

    #[clap(long)]
    pub profile: Option<String>,
}

/// Compile one or more targets.
pub fn compile(config: &CargoBuildTarget) -> Result<Vec<BinFile>> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());

    let mut cmd = Command::new(&cargo);

    cmd.arg("build");

    if config.release {
        cmd.arg("--release");
    }

    // if !config.lib {
    //     cmd.arg("--no-lib");
    // }

    if config.benches {
        cmd.arg("--benches");
    }

    if let Some(target) = &config.bench {
        cmd.arg("--bench").arg(target);
    }

    if config.tests {
        cmd.arg("--tests");
    }

    if let Some(target) = &config.test {
        cmd.arg("--test").arg(target);
    }
    if config.examples {
        cmd.arg("--examples");
    }

    if let Some(target) = &config.example {
        cmd.arg("--example").arg(target);
    }

    if let Some(features) = &config.features {
        cmd.arg("--features").arg(features.join(","));
    }

    if let Some(profile) = &config.profile {
        cmd.arg("--profile").arg(profile);
    }

    for pkg in &config.packages {
        cmd.arg("-p").arg(pkg);
    }

    cmd.arg("--message-format=json");

    let cmd_str = format!("{:?}", cmd);

    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| format!("failed to spawn cargo\n{}", cmd_str))?;

    let mut binaries = vec![];
    let reader = BufReader::new(child.stdout.take().unwrap());
    for message in Message::parse_stream(reader) {
        match message.unwrap() {
            Message::CompilerMessage(..) => {}
            Message::CompilerArtifact(mut artifact) => {
                if artifact.target.kind.contains(&"bin".to_string())
                    || artifact.target.kind.contains(&"test".to_string())
                    || artifact.target.kind.contains(&"bench".to_string())
                    || artifact.target.kind.contains(&"example".to_string())
                {
                    let mut executable = None;

                    artifact.filenames.retain(|path| {
                        if executable.is_none() && PathBuf::from(path).is_executable() {
                            executable = Some(path.clone());
                            return false;
                        }

                        true
                    });

                    binaries.push(BinFile {
                        path: match executable {
                            Some(v) => v.into(),
                            None => continue,
                        },
                        extra_files: artifact.filenames.into_iter().map(From::from).collect(),
                        profile: artifact.profile,
                        manifest_path: artifact.manifest_path.into(),
                        crate_name: artifact.target.name,
                    });
                    continue;
                }

                if artifact.target.kind == vec!["lib".to_string()] {
                    continue;
                }
                // println!("{:?}", artifact);
            }
            Message::BuildScriptExecuted(_script) => {
                // eprintln!("Executed build script of `{}`",
                // script.package_id.repr);
            }
            Message::BuildFinished(finished) => {
                if !finished.success {
                    bail!("Failed to compile binary using cargo\n{}", cmd_str)
                }
            }
            _ => (),
        }
    }

    let _output = child
        .wait()
        .with_context(|| format!("Couldn't get cargo's exit status\n{}", cmd_str))?;

    if binaries.is_empty() {
        bail!("cargo did not produce any useful binary\n{}", cmd_str)
    }

    binaries.sort_by_key(|b| b.path.clone());

    info!("cargo build produced {:?}", binaries);

    Ok(binaries)
}

#[cached(result = true)]
pub fn run_cargo_metadata_no_deps() -> Result<Arc<cargo_metadata::Metadata>> {
    let md = cargo_metadata::MetadataCommand::new()
        .no_deps()
        .exec()
        .context("cargo metadata failed")?;

    Ok(Arc::new(md))
}

#[cached(result = true)]
pub fn run_cargo_metadata_with_deps() -> Result<Arc<cargo_metadata::Metadata>> {
    let md = cargo_metadata::MetadataCommand::new()
        .exec()
        .context("cargo metadata failed")?;

    Ok(Arc::new(md))
}

pub fn cargo_target_dir() -> Result<PathBuf> {
    let md = run_cargo_metadata_no_deps()?;

    Ok(md.target_directory.clone().into())
}

pub fn cargo_workspace_dir() -> Result<PathBuf> {
    let md = run_cargo_metadata_no_deps()?;

    Ok(md.workspace_root.clone().into())
}

pub fn cargo_root_manifest() -> Result<PathBuf> {
    Ok(cargo_workspace_dir()?.join("Cargo.toml"))
}
