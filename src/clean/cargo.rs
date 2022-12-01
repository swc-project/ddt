use std::path::Path;

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
            let base_dir = target_dir.join("flavor");
            if !base_dir.exists() {
                return Ok(());
            }

            let fingerprints = read_cargo_fingerprints(&base_dir.join(".fingerprint")).await?;

            Ok(())
        })
        .await
        .with_context(|| format!("failed to clear target {}", flavor))
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fingerprint {}

/// `dir`: `.fingerprint`
async fn read_cargo_fingerprints(dir: &Path) -> Result<Vec<Fingerprint>> {
    wrap(async move {
        let entries = fs::read_dir(dir).await?;

        while let Some(e) = entries.next_entry().await? {}

        Ok(())
    })
    .await
    .with_context(|| format!("failed to read cargo fingerprints at {}", dir.display()))
}
