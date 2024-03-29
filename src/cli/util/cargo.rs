use anyhow::{bail, Context, Result};
use dialoguer::Select;

use crate::util::cargo_build::{cargo_workspace_dir, compile, BinFile, CargoBuildTarget};

pub async fn get_one_binary_using_cargo(
    build_target: &CargoBuildTarget,
) -> Result<(BinFile, Vec<(String, String)>)> {
    let bins = compile(&build_target).context("failed to build the binary using cargo")?;

    if bins.is_empty() {
        bail!("cargo build did not produce any binaries")
    }

    let bin = if bins.len() == 1 {
        bins.into_iter().next().unwrap()
    } else {
        let items = bins
            .iter()
            .map(|bin| format!("[{}] {}", bin.crate_name, bin.path.display().to_string()))
            .collect::<Vec<_>>();

        let selection = Select::new()
            .with_prompt("What do you choose?")
            .items(&items)
            .interact()
            .unwrap();

        bins.into_iter().nth(selection).unwrap()
    };

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
