use std::{env, io::Cursor, path::Path, process::Command};

use anyhow::{Context, Error};
use inferno::collapse::{
    perf::{Folder, Options as CollapseOptions},
    Collapse,
};

/// Invoked perf to record cpu usages.
pub(super) fn perf(
    root: bool,
    file: &Path,
    freq: Option<u32>,
    args: &[String],
) -> Result<Command, Error> {
    let perf = env::var("PERF").unwrap_or_else(|_| "perf".to_string());

    let mut c = if root {
        let mut c = Command::new("sudo");
        c.arg(perf);
        c
    } else {
        Command::new(perf)
    };

    c.arg("record")
        .arg("-F")
        .arg(format!("{}", freq.unwrap_or(997)))
        .arg("--call-graph")
        .arg("dwarf")
        .arg("-g");

    c.arg(&file);

    c.args(args);

    Ok(c)
}

pub(super) fn to_collapsed() -> Result<Vec<u8>, Error> {
    let perf = env::var("PERF").unwrap_or_else(|_| "perf".to_string());

    let input = Command::new(perf)
        .arg("script")
        .output()
        .context("failed to run `perf script`")?
        .stdout;

    let perf_reader = Cursor::new(input);

    let mut collapsed = vec![];

    let collapse_options = CollapseOptions::default();

    Folder::from(collapse_options)
        .collapse(perf_reader, &mut collapsed)
        .expect("unable to collapse generated profile data");

    Ok(collapsed)
}
