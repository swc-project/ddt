use anyhow::{bail, Context, Result};
use clap::Args;
use dialoguer::Select;

use super::{run::RunCommand, util::file_name_for_trace_file};
use crate::{
    cli::profile::instruments::util::XcodeInstruments,
    util::{
        cargo_build::{cargo_target_dir, cargo_workspace_dir, compile, CargoBuildTarget},
        wrap,
    },
};

/// Invoke a binary file under the `instruments` tool.
#[derive(Debug, Clone, Args)]
pub(super) struct CargoCommand {
    #[clap(long, short = 't')]
    template: String,

    #[clap(long)]
    time_limit: Option<usize>,

    #[clap(long)]
    no_open: bool,

    #[clap(flatten)]
    build_target: CargoBuildTarget,

    /// Arguments passed to the target binary.
    ///
    /// To pass flags, precede child args with `--`,
    /// e.g. `cargo profile subcommand -- -t test1.txt --slow-mode`.
    args: Vec<String>,
}

impl CargoCommand {
    pub async fn run(self, xctrace_tool: XcodeInstruments) -> Result<()> {
        let (cmd, envs) = wrap(async move {
            let bins =
                compile(&self.build_target).context("failed to build the binary using cargo")?;

            if bins.is_empty() {
                bail!("cargo build did not produce any binaries")
            }

            let bin = if bins.len() == 1 {
                bins.into_iter().next().unwrap()
            } else {
                let items = bins
                    .iter()
                    .map(|bin| format!("[{}] {}", bin.crate_name, bin.path.display().to_string()))
                    .collect::<Vec<_>>();

                let selection = Select::new()
                    .with_prompt("What do you choose?")
                    .items(&items)
                    .interact()
                    .unwrap();

                bins.into_iter().nth(selection).unwrap()
            };

            let output_path = cargo_target_dir()?
                .join("instruments")
                .join(file_name_for_trace_file(&bin.path, &self.template)?);

            let mut envs = vec![];

            let mut add = |key: &str, value: String| {
                envs.push((key.to_string(), value));
            };

            add(
                "CARGO_MANIFEST_DIR",
                bin.manifest_path
                    .parent()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
            );
            add(
                "CARGO_WORKSPACE_DIR",
                cargo_workspace_dir()?.to_string_lossy().to_string(),
            );

            Ok((
                RunCommand {
                    template: self.template,
                    time_limit: self.time_limit,
                    no_open: self.no_open,
                    args: self.args,
                    bin: bin.path,
                    output_path: Some(output_path),
                },
                envs,
            ))
        })
        .await
        .context("failed to build the target binary using cargo")?;

        cmd.run(xctrace_tool, envs)
            .await
            .context("failed to run `ddt profile instruments run` with the built binary")
    }
}
