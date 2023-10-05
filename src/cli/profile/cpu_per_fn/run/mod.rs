use std::path::PathBuf;

use ahash::{HashMap, HashSet};
use anyhow::{bail, Context, Result};
use clap::Args;
use tempfile::TempDir;
use tracing::info;

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

    #[clap(long)]
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

            let mut cmd = if cfg!(target_os = "macos") {
                make_dtrace_command(
                    self.root,
                    &self.bin,
                    &dir.path().join("program.stacks"),
                    None,
                    None,
                    &self.args,
                )?
            } else {
                bail!("ddt profile cpu-per-fn currently supports only `macos`")
            };
            for (k, v) in envs {
                cmd.env(k, v);
            }

            run_profiler(cmd).context("failed to profile program")?;

            info!("Processing collapsed stack data");

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

struct FnTimingInfo {
    name: String,
    total_used: usize,
    /// The percentage of time used by function code itself.
    self_used: usize,
}

fn process_collapsed(data: &str) -> Result<(usize, Vec<FnTimingInfo>)> {
    let mut lines: Vec<&str> = data.lines().into_iter().collect();
    lines.reverse();
    let (frames, time, ignored) =
        merge::frames(lines, true).context("failed to merge collapsed stack frame")?;

    if time == 0 {
        bail!("No stack counts found")
    }

    if ignored > 0 {
        eprintln!("ignored {} lines with invalid format", ignored)
    }

    let mut total_time = HashMap::<_, usize>::default();
    let mut itself_time = HashMap::<_, usize>::default();

    // Check if time collapses
    for frame in &frames {
        let fn_dur = frame.end_time - frame.start_time;
        *total_time.entry(frame.location.function).or_default() += fn_dur;

        let children = frames.iter().filter(|child| {
            frame.location.depth + 1 == child.location.depth
                && frame.start_time <= child.start_time
                && child.end_time <= frame.end_time
        });

        let mut itself_dur = fn_dur;

        for child in children {
            let fn_dur = child.end_time - child.start_time;
            itself_dur -= fn_dur;
        }

        *itself_time.entry(frame.location.function).or_default() += itself_dur;
    }

    let mut result = vec![];
    let mut done = HashSet::default();

    for frame in &frames {
        if !done.insert(&frame.location.function) {
            continue;
        }
        result.push(FnTimingInfo {
            name: frame.location.function.to_string(),
            total_used: *total_time.entry(frame.location.function).or_default(),
            self_used: *itself_time.entry(frame.location.function).or_default(),
        });
    }

    Ok((time, result))
}
