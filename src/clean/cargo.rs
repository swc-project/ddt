use std::{collections::HashMap, path::Path};

use anyhow::{Context, Result};
use cargo_metadata::{CargoOpt, MetadataCommand};
use futures::try_join;
use serde::Deserialize;
use tokio::fs;

use super::CleanCommand;
use crate::util::wrap;

impl CleanCommand {
    /// Clean up `target` of cargo.
    ///
    /// We only remove build outputs for outdated dependencies.
    pub(super) async fn remove_unused_files_of_cargo(&self, git_dir: &Path) -> Result<()> {
        wrap(async move {
            let metadata = MetadataCommand::new()
                .current_dir(git_dir)
                .features(CargoOpt::AllFeatures)
                .exec();
            // Not a cargo project?
            // TODO: Log
            let metadata = match metadata {
                Ok(metadata) => metadata,
                Err(_) => return Ok(()),
            };

            // Calculate current dependencies

            let target_dir = metadata.target_directory.as_std_path().to_path_buf();

            try_join!(
                self.clean_one_target(&target_dir, "debug"),
                self.clean_one_target(&target_dir, "release"),
            )?;

            Ok(())
        })
        .await
        .with_context(|| {
            format!(
                "failed to clean up cargo target dir at {}",
                git_dir.display()
            )
        })
    }

    async fn clean_one_target(&self, target_dir: &Path, flavor: &str) -> Result<()> {
        wrap(async move {
            let base_dir = target_dir.join(flavor);

            if !base_dir.exists() {
                return Ok(());
            }

            let fingerprints = read_cargo_fingerprints(&base_dir.join(".fingerprint")).await?;

            dbg!(fingerprints);

            Ok(())
        })
        .await
        .with_context(|| format!("failed to clear target {}", flavor))
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fingerprint {
    rustc: u128,
    features: String,
    target: u128,
    profile: u128,
    path: u128,
    deps: Vec<(u128, String, bool, u128)>,

    local: Vec<LocalData>,

    rustflags: Vec<String>,

    metadata: u128,
    config: u128,
    compile_kind: u128,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LocalData {
    #[serde(rename = "CheckDepInfo")]
    check_dep_info: CheckDepInfo,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckDepInfo {
    dep_info: String,
}

/// `dir`: `.fingerprint`
async fn read_cargo_fingerprints(dir: &Path) -> Result<Vec<Fingerprint>> {
    wrap(async move {
        let mut entries = fs::read_dir(dir).await?;
        let mut fingerprints = vec![];

        while let Some(e) = entries.next_entry().await? {
            let mut files = fs::read_dir(e.path()).await?;

            while let Some(f) = files.next_entry().await? {
                let path = f.path();
                if path.extension().map(|s| s == "json").unwrap_or(false) {
                    let content = fs::read_to_string(&path).await?;
                    let fingerprint: Fingerprint =
                        serde_json::from_str(&content).with_context(|| {
                            format!("failed to parse fingerprint file at {}", path.display())
                        })?;

                    fingerprints.push(fingerprint);
                }
            }
        }

        Ok(fingerprints)
    })
    .await
    .with_context(|| format!("failed to read cargo fingerprints at {}", dir.display()))
}
