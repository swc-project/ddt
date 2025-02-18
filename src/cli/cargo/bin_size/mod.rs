use anyhow::Result;
use clap::Args;

#[derive(Debug, Args)]
pub(super) struct BinSizeCommand {}

impl BinSizeCommand {
    pub async fn run(self) -> Result<()> {
        Ok(())
    }
}
