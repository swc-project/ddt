use std::{
    path::{Path, PathBuf},
    process::Stdio,
};

use anyhow::{Context, Result};
use clap::Args;
use futures::{future::try_join_all, try_join};
use tokio::process::Command;

use crate::util::wrap;

mod cargo;

/// Clean unused, old project files.
///
/// 1. This runs `git fetch --all --prune` on all projects. (does not support
/// dry run) 2. This removes
///
///  - the unused files in `target` directory.
///  - local git brach which has a remote branch but it's removed.
#[derive(Debug, Args)]
pub(crate) struct CleanCommand {
    #[clap(short, long)]
    dry_run: bool,

    /// The directory to clean.
    ///
    /// If this is a child of a git repository, this command will run `git fetch
    /// --all --prune` on it and clean only subdirectories.
    dir: PathBuf,
}

impl CleanCommand {
    pub async fn run(&self) -> Result<()> {
        let git_projects = find_git_projects(&self.dir)
            .await
            .with_context(|| format!("failed to find git projects from {}", self.dir.display()))?;

        try_join_all(
            git_projects
                .iter()
                .map(|git_dir| run_git_fetch_all_prune(git_dir)),
        )
        .await
        .context("failed to run git fetch step")?;

        let clean_dead_branches = async {
            try_join_all(
                git_projects
                    .iter()
                    .map(|git_dir| self.remove_dead_branches(git_dir)),
            )
            .await
            .context("failed to clean up dead branches")
        };
        let remove_unused_files = async {
            try_join_all(
                git_projects
                    .iter()
                    .map(|git_dir| self.remove_unused_files_of_cargo(git_dir)),
            )
            .await
            .context("failed to clean up unused files")
        };

        try_join!(clean_dead_branches, remove_unused_files).context("failed to clean up")?;

        Ok(())
    }

    async fn remove_dead_branches(&self, git_dir: &Path) -> Result<()> {
        wrap(async move {
            let branches = Command::new("git")
                .arg("for-each-ref")
                .arg("--format")
                .arg("%(refname:short) %(upstream:track)")
                .current_dir(git_dir)
                .stderr(Stdio::inherit())
                .kill_on_drop(true)
                .output()
                .await
                .context("failed to get git refs")?;

            let branches = String::from_utf8(branches.stdout)
                .context("failed to parse output of git refs as utf9")?;

            for line in branches.lines() {
                let items = line.split_whitespace().collect::<Vec<_>>();
                if items.len() == 2 && items[1] == "[gone]" {
                    let branch = items[0];

                    if self.dry_run {
                        println!("git branch -D {} # {}", branch, git_dir.display());
                    } else {
                        // TODO: Log status
                        let _status = Command::new("git")
                            .arg("branch")
                            .arg("-D")
                            .arg(branch)
                            .current_dir(git_dir)
                            .kill_on_drop(true)
                            .status()
                            .await
                            .with_context(|| format!("failed to delete branch {}", branch,))?;
                    }
                }
            }

            Ok(())
        })
        .await
        .with_context(|| format!("failed to clean up dead branches in {}", git_dir.display()))
    }
}

async fn find_git_projects(dir: &Path) -> Result<Vec<PathBuf>> {
    /// Find recursively git projects from the given directory.
    async fn find(dir: &Path) -> Result<Vec<PathBuf>> {
        Ok(vec![dir.to_path_buf()])
    }

    // TODO: Check if `dir` is in a git repository.

    find(dir).await
}

/// - `dir`: The root directory of git repository.
async fn run_git_fetch_all_prune(git_dir: &Path) -> Result<()> {
    let mut c = Command::new("git");
    c.arg("fetch").arg("--all").arg("--prune");
    c.kill_on_drop(true);

    // TODO: Log status code
    let _status = c.status().await.with_context(|| {
        format!(
            "failed to get status of `git fetch --all --prune` for {}",
            git_dir.display()
        )
    })?;

    Ok(())
}
