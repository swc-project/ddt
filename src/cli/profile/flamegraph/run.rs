use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;
use tempfile::TempDir;

use crate::util::wrap;

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

    pub args: Vec<String>,
}

impl RunCommand {
    pub async fn run(self, envs: Vec<(String, String)>) -> Result<()> {
        let c = self.clone();

        wrap(async move {
            //

            let dir = TempDir::new_in("ddt-flamegraph").context("failed to create temp dir")?;

            //
            eprintln!("Profiling {}", binary.path.display());

            let cmd = if cfg!(target_os = "macos") {
                make_dtrace_command(
                    root,
                    binary,
                    &dir.path().join(self::macos::DTRACE_OUTPUT_FILENAME),
                    None,
                    None,
                    target.args(),
                )?
            } else if cfg!(target_os = "linux") {
                self::linux::perf(root, binary, None, target.args())?
            } else {
                bail!("cargo profile flamegraph currently supports only `linux` and `macos`")
            };

            run_profiler(cmd).context("failed to profile program")?;

            let collapsed: Vec<u8> = if cfg!(target_os = "macos") {
                crate::cli_tools::dtrace::to_collapsed(
                    &dir.path().join(self::macos::DTRACE_OUTPUT_FILENAME),
                )?
            } else if cfg!(target_os = "linux") {
                self::linux::to_collapsed()?
            } else {
                bail!("cargo profile flamegraph currently supports only `linux` and `macos`")
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
