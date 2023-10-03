use anyhow::Result;
use clap::Args;

use crate::cli::profile::instruments::util::{render_template_catalog, XcodeInstruments};

/// List available templates
#[derive(Debug, Clone, Args)]
pub(super) struct ListTemplatesCommand {}

impl ListTemplatesCommand {
    // Render available templates if the user asked
    pub async fn run(self, xctrace_tool: XcodeInstruments) -> Result<()> {
        let catalog = xctrace_tool.available_templates()?;
        println!("{}", render_template_catalog(&catalog));
        return Ok(());
    }
}
