#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
use std::process::{Command, ExitStatus};

use anyhow::{bail, Context, Error};
use tracing::info;

/// Invokes profiler with proper signal hooks.
///
/// This function is expected to run only `dtrace` or `perf`.
pub fn run_profiler(mut cmd: Command) -> Result<(), Error> {
    let cmd_str = format!("{:?}", cmd);

    info!("Running profiler: {}", cmd_str);

    // Handle SIGINT with an empty handler. This has the
    // implicit effect of allowing the signal to reach the
    // process under observation while we continue to
    // generate our flamegraph.  (ctrl+c will send the
    // SIGINT signal to all processes in the foreground
    // process group).
    #[cfg(unix)]
    let handler = unsafe {
        signal_hook::low_level::register(signal_hook::consts::SIGINT, || {})
            .expect("cannot register signal handler")
    };

    let mut recorder = cmd
        .spawn()
        .with_context(|| format!("failed to spawn: {}", cmd_str))?;

    let exit_status = recorder
        .wait()
        .with_context(|| format!("failed to wait for child proceess: {}", cmd_str))?;

    #[cfg(unix)]
    signal_hook::low_level::unregister(handler);

    // only stop if perf exited unsuccessfully, but
    // was not killed by a signal (assuming that the
    // latter case usually means the user interrupted
    // it in some way)
    if terminated_by_error(exit_status) {
        bail!("the binary file exited with an error: {}", cmd_str);
    }

    Ok(())
}

#[cfg(unix)]
fn terminated_by_error(status: ExitStatus) -> bool {
    status
        .signal() // the default needs to be true because that's the neutral element for `&&`
        .map_or(true, |code| {
            code != signal_hook::consts::SIGINT && code != signal_hook::consts::SIGTERM
        })
        && !status.success()
}

#[cfg(not(unix))]
fn terminated_by_error(status: ExitStatus) -> bool {
    !status.success()
}
