use std::{
    fs::OpenOptions,
    io::{BufWriter, Cursor},
    path::Path,
};

use anyhow::{bail, Context, Error};
use structopt::StructOpt;
use tempdir::TempDir;

use crate::{
    cargo::{compile, CargoTarget},
    cli_tools::{dtrace::make_dtrace_command, profiler::run_profiler},
};

mod linux;
mod macos;

/// Creates a flamegraph for given target.
#[derive(Debug, Clone, StructOpt)]
pub struct FlameGraphCommand {
    /// Use sudo.
    #[structopt(long)]
    root: bool,

    /// Compile library
    #[structopt(flatten)]
    target: CargoTarget,
}

impl FlameGraphCommand {
    pub fn run(self) -> Result<(), Error> {
        let Self { root, target } = self;

        let binaries = compile(&target).context("cargo execution failed")?;

        if binaries.len() != 1 {
            // TODO
            bail!(
                "Currently cargo profile flaemgraph only supports single binary, but cargo \
                 produced {} binaries",
                binaries.len()
            )
        }

        for binary in &binaries {}

        Ok(())
    }
}
