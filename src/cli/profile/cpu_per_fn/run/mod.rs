use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::Args;
use tempfile::TempDir;

use crate::{
    cli::profile::util::{
        dtrace::{self, make_dtrace_command},
        profiler::run_profiler,
    },
    util::wrap,
};

mod merge;

/// Invoke a binary file and profile the cpu count per function
#[derive(Debug, Clone, Args)]
pub(super) struct RunCommand {
    /// The target binary to profile
    pub bin: PathBuf,

    pub root: bool,

    #[clap(long)]
    pub time_limit: Option<usize>,

    pub args: Vec<String>,
}

impl RunCommand {
    pub async fn run(self, envs: Vec<(String, String)>) -> Result<()> {
        let c = self.clone();

        wrap(async move {
            let dir = TempDir::new().context("failed to create temp dir")?;

            let cmd = if cfg!(target_os = "macos") {
                make_dtrace_command(
                    self.root,
                    &self.bin,
                    &dir.path().join("program.stacks"),
                    None,
                    None,
                    &self.args,
                )?
            } else {
                bail!("cargo profile cpu currently supports only `macos`")
            };
            run_profiler(cmd).context("failed to profile program")?;

            let collapsed: Vec<u8> = if cfg!(target_os = "macos") {
                dtrace::to_collapsed(&dir.path().join("program.stacks"))?
            } else {
                unreachable!()
            };

            let collapsed = String::from_utf8_lossy(&collapsed);

            let (time, mut data) =
                process_collapsed(&collapsed).context("failed to process collapsed stack data")?;
            data.sort_by_key(|info| info.total_used);

            println!(
                "{: <10}  | {: <10}  | {}",
                "Totql time", "Own time", "File name",
            );
            for info in data.iter().rev() {
                println!(
                    "{: <10.1}% | {: <10.1}% | {}",
                    info.total_used as f64 / time as f64 * 100f64,
                    info.self_used as f64 / time as f64 * 100f64,
                    info.name,
                );
            }

            Ok(())
        })
        .await
        .with_context(|| {
            format!(
                "failed to profile cpu count per function with `{}` `{:?}",
                c.bin.display(),
                c.args
            )
        })
    }
}
