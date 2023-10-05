use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::Args;

use super::run::RunCommand;
use crate::{
    cli::util::cargo::get_one_binary_using_cargo,
    util::{
        cargo_build::{cargo_target_dir, CargoBuildTarget},
        wrap,
    },
};

/// Invoke a binary file built using `cargo` and create a flamegraph
#[derive(Debug, Clone, Args)]
pub(super) struct CargoCommand {
    #[clap(long)]
    time_limit: Option<usize>,

    /// The path to the output flamegraph file
    #[clap(long, short = 'o')]
    output_path: Option<PathBuf>,

    #[clap(long)]
    no_open: bool,

    #[clap(long)]
    root: bool,

    #[clap(flatten)]
    build_target: CargoBuildTarget,

    /// Arguments passed to the target binary.
    ///
    /// To pass flags, precede child args with `--`,
    /// e.g. `ddt profile subcommand -- -t test1.txt --slow-mode`.
    args: Vec<String>,
}

impl CargoCommand {
    pub async fn run(self) -> Result<()> {
        let (cmd, envs) = wrap(async move {
            let (bin, envs) = get_one_binary_using_cargo(&self.build_target).await?;

            let target_shortname = bin
                .path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| anyhow!("invalid target path {:?}", bin.path))?;
            let now = chrono::Local::now();

            let output_path = cargo_target_dir()?.join("flamegraph").join(format!(
                "{}_{}.svg",
                target_shortname,
                now.format("%F_%H%M%S-%3f")
            ));

            Ok((
                RunCommand {
                    bin: bin.path,
                    time_limit: self.time_limit,
                    output_path: Some(output_path),
                    no_open: self.no_open,
                    root: self.root,
                    args: self.args,
                },
                envs,
            ))
        })
        .await
        .context("failed to build the target binary using cargo")?;

        cmd.run(envs)
            .await
            .context("failed to run `ddt profile flamegraph run` with the built binary")
    }
}
