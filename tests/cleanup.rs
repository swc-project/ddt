use anyhow::Result;
use std::process::Command;
use tempfile::{tempdir, TempDir};

fn setup_source() -> Result<TempDir> {
    let dir = tempdir()?;
    Command::new("cargo")
        .args(["new", "--lib", "primary"])
        .current_dir(&dir)
        .output()?;
    Ok(dir)
}

#[test]
fn cleanup_3_removed_libs() -> Result<()> {
    let testdir = setup_source()?;

    Ok(())
}
