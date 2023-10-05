use std::path::Path;

use anyhow::{bail, Result};

pub mod cargo;

pub fn open_file(filename: &Path) -> Result<()> {
    use std::process::Command;

    let status = Command::new("open").arg(filename).status()?;

    if !status.success() {
        bail!("`open` failed")
    }
    Ok(())
}
