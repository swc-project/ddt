use anyhow::{bail, Context, Result};
use dialoguer::Select;

use crate::util::cargo_build::{compile, BinFile, CargoBuildTarget};

pub async fn get_one_binary_using_cargo(build_target: &CargoBuildTarget) -> Result<BinFile> {
    let bins = compile(&build_target).context("failed to build the binary using cargo")?;

    if bins.is_empty() {
        bail!("cargo build did not produce any binaries")
    }

    Ok(if bins.len() == 1 {
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
    })
}
