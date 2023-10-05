use std::{
    fs::OpenOptions,
    io::{BufWriter, Cursor},
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use clap::Args;
use tempfile::TempDir;
use tracing::info;

use crate::{
    cli::{
        profile::util::{
            dtrace::{self, make_dtrace_command},
            profiler::run_profiler,
        },
        util::open_file,
    },
    util::wrap,
};

mod linux;

/// Invoke a binary file and create a flamegraph
#[derive(Debug, Clone, Args)]
pub(super) struct RunCommand {
    /// The target binary to profile
    pub bin: PathBuf,

    #[clap(long)]
    pub time_limit: Option<usize>,

    /// The path to the output flamegraph file
    #[clap(long, short = 'o')]
    pub output_path: Option<PathBuf>,

    #[clap(long)]
    pub no_open: bool,

    #[clap(long)]
    pub root: bool,

    pub args: Vec<String>,
}

const DTRACE_OUTPUT_FILENAME: &str = "cargo-profile-flamegraph.stacks";

impl RunCommand {
    pub async fn run(self, envs: Vec<(String, String)>) -> Result<()> {
        let c = self.clone();

        wrap(async move {
            //

            let dir = TempDir::new().context("failed to create temp dir")?;

            //
            info!("Profiling {}", self.bin.display());

            let mut cmd = if cfg!(target_os = "macos") {
                make_dtrace_command(
                    self.root,
                    &self.bin,
                    &dir.path().join(DTRACE_OUTPUT_FILENAME),
                    None,
                    None,
                    &self.args,
                )?
            } else if cfg!(target_os = "linux") {
                self::linux::perf(self.root, &self.bin, None, &self.args)?
            } else {
                bail!("ddt profile flamegraph currently supports only `linux` and `macos`")
            };
            for (k, v) in envs {
                cmd.env(k, v);
            }

            run_profiler(cmd).context("failed to profile program")?;

            let collapsed: Vec<u8> = if cfg!(target_os = "macos") {
                dtrace::to_collapsed(&dir.path().join(DTRACE_OUTPUT_FILENAME))?
            } else if cfg!(target_os = "linux") {
                self::linux::to_collapsed()?
            } else {
                bail!("ddt profile flamegraph currently supports only `linux` and `macos`")
            };
            let mut collapsed = Cursor::new(collapsed);

            // TODO
            let flamegraph_file_path = Path::new("flamegraph.svg");
            let flamegraph_file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(&flamegraph_file_path)
                .context("unable to create flamegraph.svg output file")?;

            let flamegraph_writer = BufWriter::new(flamegraph_file);

            let mut flamegraph_options = inferno::flamegraph::Options::default();

            inferno::flamegraph::from_reader(
                &mut flamegraph_options,
                &mut collapsed,
                flamegraph_writer,
            )
            .with_context(|| {
                format!(
                    "unable to generate a flamegraph file ({}) from the collapsed stack data",
                    flamegraph_file_path.display()
                )
            })?;

            info!("Flamegraph printed to {}", flamegraph_file_path.display());

            if !self.no_open {
                let _ = open_file(&flamegraph_file_path);
            }

            Ok(())
        })
        .await
        .with_context(|| {
            format!(
                "failed to create a flamegraph from `{}` `{:?}",
                c.bin.display(),
                c.args
            )
        })
    }
}
