use anyhow::Result;
use std::fs::{write, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::{tempdir, TempDir};

fn cargo_invoke() -> Command {
    let mut c = Command::new("cargo");
    c.arg("--offline");
    c
}

fn setup_source() -> Result<TempDir> {
    let dir = tempdir()?;
    cargo_invoke()
        .args(["new", "--lib", "primary"])
        .current_dir(&dir)
        .status()?;
    Ok(dir)
}

fn add_dep(dir: &Path, dep: &str) -> Result<()> {
    cargo_invoke()
        .args(["new", "--lib"])
        .arg(&dep)
        .current_dir(&dir)
        .status()?;
    let primary_path = dir.join("primary");
    cargo_invoke()
        .args(["add", "--path"])
        .arg(format!("../{}", dep))
        .current_dir(&primary_path)
        .status()?;
    Ok(())
}

fn target_dir_glob(testdir: &TempDir, file_pattern: &str) -> Result<Vec<PathBuf>> {
    let mut pattern = testdir.path().join("primary/target/debug/deps/");
    pattern.push(file_pattern);
    let pattern = pattern.to_str().expect("file pattern should be utf8");

    Ok(glob::glob(&pattern)?.filter_map(Result::ok).collect())
}

#[test]
fn cleanup_3_removed_libs() -> Result<()> {
    let testdir = setup_source()?;
    let primary_toml_path = testdir.path().join("primary/Cargo.toml");
    let original_cargo_toml = {
        let mut string = String::new();
        File::open(&primary_toml_path)?.read_to_string(&mut string)?;
        string
    };

    add_dep(testdir.path(), "dep0")?;
    add_dep(testdir.path(), "dep1")?;
    add_dep(testdir.path(), "dep2")?;

    cargo_invoke()
        .arg("build")
        .current_dir(testdir.path().join("primary"))
        .output()?;

    assert_eq!(4, target_dir_glob(&testdir, "*.rlib")?.len());
    assert_eq!(4, target_dir_glob(&testdir, "*.rmeta")?.len());

    write(&primary_toml_path, &original_cargo_toml).expect("Could not write to primary Cargo.toml");

    assert_eq!(1, target_dir_glob(&testdir, "*.rlib")?.len());
    assert_eq!(1, target_dir_glob(&testdir, "*.rmeta")?.len());

    Ok(())
}
