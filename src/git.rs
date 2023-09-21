//! Utils for interacting with git.
use std::{path::PathBuf, sync::Arc};

use anyhow::{bail, Context, Result};
use tracing::debug;

use crate::util::{wrap, PrettyCmd};

/// Utility for git hooks, which cannot use commands like `git add`.
#[derive(Debug)]
pub struct GitWorkflow {
    matched_file_chunks: Arc<Vec<Vec<String>>>,
    git_dir: Arc<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct PrepareResult {
    pub partially_staged_files: Arc<Vec<String>>,
}

/// Methods ported from lint-staged.
impl GitWorkflow {
    /// Create a diff of partially staged files and backup stash if enabled.
    #[tracing::instrument(name = "GitWorkflow::prepare", skip_all)]
    pub async fn prepare(self: Arc<Self>) -> Result<PrepareResult> {
        wrap(async move { self.prepare_inner().await })
            .await
            .context("failed to prepare a git workflow")
    }

    async fn prepare_inner(self: Arc<Self>) -> Result<PrepareResult> {
        debug!("Backing up original state...");

        let partially_staged_files = self.get_partially_staged_files().await?;

        if !partially_staged_files.is_empty() {
            let unstage_patch = self.get_hidden_filepath(PATCH_UNSTAGED);
            let files = process_renames(partially_staged_files);

            let mut args = vec![String::from("diff")];
            args.extend(GIT_DIFF_ARGS);
            args.push("--output".into());

            args.push(unstage_patch);
            args.push("--".into());
            args.extend(files);

            self.exec_git(args).await?;
        }

        // TODO: https://github.com/okonet/lint-staged/blob/19a6527c8ac07dbafa2b8c1774e849d3cab635c3/lib/gitWorkflow.js#L210-L229

        Ok(PrepareResult {
            partially_staged_files,
        })
    }

    /// Remove unstaged changes to all partially staged files, to avoid tasks
    /// from seeing them
    #[tracing::instrument(name = "GitWorkflow::hide_unstaged_changes", skip_all)]
    pub async fn hide_unstaged_changes(
        self: Arc<Self>,
        partially_staged_files: Arc<Vec<String>>,
    ) -> Result<()> {
        wrap(async move {
            self.hide_unstaged_changes_inner(partially_staged_files)
                .await
        })
        .await
        .context("failed to hide unstaged changes")
    }

    async fn hide_unstaged_changes_inner(
        self: Arc<Self>,
        partially_staged_files: Arc<Vec<String>>,
    ) -> Result<()> {
        let files = process_renames(partially_staged_files, false).await?;

        let mut args = vec![String::from("checkout"), "--force".into(), "--".into()];
        args.extend(files);
        self.exec_git(args).await?;

        Ok(())
    }

    /// Applies back task modifications, and unstaged changes hidden in the
    /// stash.
    /// In case of a merge-conflict retry with 3-way merge.
    #[tracing::instrument(name = "GitWorkflow::apply_modifications", skip_all)]
    pub async fn apply_modifications(self: Arc<Self>) -> Result<()> {
        wrap(async move { self.apply_modifications_inner().await })
            .await
            .context("failed to apply modifications")
    }

    async fn apply_modifications_inner(self: Arc<Self>) -> Result<()> {
        debug!("Adding task modifications to index...");

        // `matchedFileChunks` includes staged files that lint-staged originally
        // detected and matched against a task. Add only these files so any
        // 3rd-party edits to other files won't be included in the commit. These
        // additions per chunk are run "serially" to prevent race conditions.
        // Git add creates a lockfile in the repo causing concurrent operations to fail.
        // for (const files of this.matchedFileChunks) {
        //     await this.execGit(['add', '--', ...files])
        //   }
        for files in self.matched_file_chunks.iter() {
            let mut args = vec![String::from("add"), "--".into()];
            args.extend(files.iter().cloned());
            self.exec_git(args).await?;
        }

        debug!("Done adding task modifications to index!");

        let staged_files_after_add = self
            .exec_git(get_diff_command(self.diff, self.diffFilter))
            .await;
        if !staged_files_after_add && !self.allowEmpty {
            // Tasks reverted all staged changes and the commit would be empty
            // Throw error to stop commit unless `--allow-empty` was used
            bail!("Prevented an empty git commit!")
        }

        Ok(())
    }

    #[tracing::instrument(name = "GitWorkflow::restore_unstaged_changes", skip_all)]
    pub async fn restore_unstaged_changes(self: Arc<Self>) -> Result<()> {
        self.restore_unstaged_changes_inner().await
    }

    async fn restore_unstaged_changes_inner(self: Arc<Self>) -> Result<()> {}

    #[tracing::instrument(name = "GitWorkflow::restore_original_state", skip_all)]
    pub async fn restore_original_state(self: Arc<Self>) -> Result<()> {
        self.restore_original_state_inner().await
    }

    async fn restore_original_state_inner(self: Arc<Self>) -> Result<()> {}

    #[tracing::instrument(name = "GitWorkflow::cleanup", skip_all)]
    pub async fn cleanup(self: Arc<Self>) -> Result<()> {
        self.cleanup_inner().await
    }

    async fn cleanup_inner(self: Arc<Self>) -> Result<()> {
        debug!("Dropping backup stash...");

        let backup_stash = self.get_backup_stash().await?;
        let args = vec![
            String::from("stash"),
            "drop".into(),
            "--quiet".into(),
            backup_stash,
        ];
        self.exec_git(args).await?;

        debug!("Done dropping backup stash!")
    }

    #[tracing::instrument(name = "GitWorkflow::exec_git", skip_all)]
    async fn exec_git(self: Arc<Self>, args: Vec<String>) -> Result<()> {
        self.exec_git_inner(args).await
    }

    async fn exec_git_inner(self: Arc<Self>, args: Vec<String>) -> Result<()> {
        PrettyCmd::new("Running git command", "git")
            .dir(&*self.git_dir)
            .args(args)
            .exec()
            .await
            .context("git command failed")?;

        Ok(())
    }
}
