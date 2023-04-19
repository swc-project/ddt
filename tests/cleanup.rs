use anyhow::Result;
use std::path::Path;
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

fn add_dep(dir: &Path, dep: &str) -> Result<()> {
    Command::new("cargo")
        .args(["new", "--lib"])
        .arg(&dep)
        .current_dir(&dir)
        .output()?;
    let primary_path = dir.join("primary");
    Command::new("cargo")
        .args(["add", "--path"])
        .arg(format!("../{}", dep))
        .current_dir(&primary_path)
        .output()?;
    Ok(())
}

#[test]
fn cleanup_3_removed_libs() -> Result<()> {
    let testdir = setup_source()?;

    add_dep(testdir.path(), "dep0")?;
    add_dep(testdir.path(), "dep1")?;
    add_dep(testdir.path(), "dep2")?;

    Ok(())
}
