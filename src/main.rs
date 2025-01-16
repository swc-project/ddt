#![allow(clippy::large_enum_variant)]

extern crate swc_malloc;

use std::io;

use anyhow::Result;
use clap::Parser;
use tracing::info;

use crate::cli::CliArgs;

mod cli;
mod git;
mod package_manager;
mod util;

#[tokio::main]

async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_writer(io::stderr)
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_level(true)
        .pretty()
        .init();

    let args = CliArgs::parse();

    let start = std::time::Instant::now();

    info!("Start");

    args.run().await?;

    info!("End in {:?}", start.elapsed());

    Ok(())
}
