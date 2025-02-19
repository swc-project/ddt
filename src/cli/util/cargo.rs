use std::process::Command;

use anyhow::{bail, Context, Result};
use cached::proc_macro::cached;
use dialoguer::Select;
use hstr::Atom;
use tempfile::tempdir;
use tracing::info;

use crate::util::cargo_build::{
    cargo_workspace_dir, compile, run_cargo_metadata_with_deps, BinFile, CargoBuildTarget,
};

pub async fn get_one_binary_using_cargo(
    build_target: &CargoBuildTarget,
) -> Result<(BinFile, Vec<(String, String)>)> {
    let bins = compile(build_target).context("failed to build the binary using cargo")?;

    if bins.is_empty() {
        bail!("cargo build did not produce any binaries")
    }

    let bin = if bins.len() == 1 {
        bins.into_iter().next().unwrap()
    } else {
        let items = bins
            .iter()
            .map(|bin| format!("[{}] {}", bin.crate_name, bin.path.display()))
            .collect::<Vec<_>>();

        let selection = Select::new()
            .with_prompt("What do you choose?")
            .items(&items)
            .interact()
            .unwrap();

        bins.into_iter().nth(selection).unwrap()
    };

    {
        let mut cmd = Command::new("codesign");
        cmd.arg("-s").arg("-").arg("-v").arg("-f");

        let tmp_dir = tempdir()?;
        let plist = tmp_dir.path().join("entitlements.xml");

        let entitlements = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
    <dict>
        <key>com.apple.security.get-task-allow</key>
        <true/>
    </dict>
</plist>
        
        "#;

        std::fs::write(&plist, entitlements).context("failed to write the entitlements file")?;

        cmd.arg("--entitlements").arg(&plist);

        cmd.arg(&bin.path);

        info!("Running codesign on the built binary...");
        let status = cmd.status().context("failed to codesign the binary")?;

        if !status.success() {
            bail!("failed to codesign the binary")
        }
    }

    if cfg!(target_os = "macos") {
        info!("Running dsymutil on the built binary...");

        let status = Command::new("dsymutil")
            .arg(&bin.path)
            .status()
            .context("failed to run dsymutil on the binary")?;

        if !status.success() {
            bail!("failed to run dsymutil on the binary")
        }
    }

    let mut envs = vec![];

    let mut add = |key: &str, value: String| {
        envs.push((key.to_string(), value));
    };

    add(
        "CARGO_MANIFEST_DIR",
        bin.manifest_path
            .parent()
            .unwrap()
            .to_string_lossy()
            .to_string(),
    );
    add(
        "CARGO_WORKSPACE_DIR",
        cargo_workspace_dir()?.to_string_lossy().to_string(),
    );

    Ok((bin, envs))
}

#[cached(result = true)]
pub fn to_original_crate_name(lib_name: Atom) -> Result<Atom> {
    if matches!(&*lib_name, "std" | "core" | "alloc" | "proc_macro") {
        return Ok(lib_name);
    }

    let md = run_cargo_metadata_with_deps().context("failed to run cargo metadata")?;

    md.packages
        .iter()
        .find(|p| {
            if lib_name == *p.name {
                return true;
            }

            for t in &p.targets {
                if t.kind.iter().any(|k| k == "lib" || k == "rlib") && lib_name == *t.name {
                    return true;
                }
            }

            false
        })
        .map(|pkg| Atom::from(&*pkg.name))
        .ok_or_else(|| anyhow::anyhow!("failed to find the crate for library: {}", lib_name))
}
