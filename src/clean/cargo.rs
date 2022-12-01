use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use cargo_metadata::{CargoOpt, MetadataCommand};
use futures::{future::try_join_all, try_join};
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

            let dep_files = read_deps_dir(&base_dir.join("deps")).await?;

            try_join_all(dep_files.iter().map(|dep| {
                wrap(async move {
                    for (_, deps) in dep.map.iter() {
                        // Workspace-local file
                        if deps.iter().any(|path| path.is_relative()) {
                            return Ok(());
                        }
                    }

                    for (file, _) in dep.map.iter() {
                        if file.ancestors().all(|dir| dir != target_dir) {
                            continue;
                        }

                        if let Some(ext) = file.extension() {
                            if ext == "rlib" || ext == "rmeta" {
                                // We only delete rlib and rmeta
                            } else {
                                continue;
                            }
                        } else {
                            continue;
                        }

                        if self.dry_run {
                            println!("cargo: remove {}", file.display());
                        } else {
                            fs::remove_file(file).await?;
                        }
                    }

                    Ok(())
                })
            }))
            .await?;

            Ok(())
        })
        .await
        .with_context(|| format!("failed to clear target {}", flavor))
    }
}

/// .d file
#[derive(Debug)]
struct DepFile {
    map: HashMap<PathBuf, Vec<PathBuf>, ahash::RandomState>,
}

async fn read_deps_dir(dir: &Path) -> Result<Vec<DepFile>> {
    wrap(async move {
        let mut entries = fs::read_dir(dir).await?;
        let mut files = vec![];

        while let Some(e) = entries.next_entry().await? {
            if e.path().extension().map_or(false, |ext| ext == "d") {
                let content = fs::read_to_string(e.path()).await?;
                let file = parse_dep_file(&content)?;
                files.push(file);
            }
        }

        Ok(files)
    })
    .await
    .with_context(|| format!("failed to read cargo deps at {}", dir.display()))
}

fn parse_dep_file(s: &str) -> Result<DepFile> {
    let entries = s
        .lines()
        .map(|s| s.trim())
        .filter(|&s| !s.is_empty())
        .map(|line| line.split_once(':').unwrap())
        .map(|(k, v)| {
            (
                PathBuf::from(k),
                v.split_whitespace().map(PathBuf::from).collect(),
            )
        })
        .collect();

    Ok(DepFile { map: entries })
}
