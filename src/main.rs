use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
struct Args {}

#[tokio::main]

async fn main() -> Result<()> {
    let args = Args::parse();

    println!("Hello, world!");

    Ok(())
}
