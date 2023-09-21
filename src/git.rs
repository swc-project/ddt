//! Utils for interacting with git.
use std::sync::Arc;

use anyhow::Result;
use tracing::debug;

use crate::util::wrap;

/// Utility for git hooks, which cannot use commands like `git add`.
#[derive(Debug)]
pub struct GitWorkflow {}

/// Methods ported from lint-staged.
impl GitWorkflow {
    /// Create a diff of partially staged files and backup stash if enabled.
    #[tracing::instrument(name = "GitWorkflow::prepare", skip_all)]
    pub async fn prepare(self: Arc<Self>) -> Result<()> {
        self.prepare_inner().await
    }

    async fn prepare_inner(self: Arc<Self>) -> Result<()> {
        wrap(async move {
            debug!("Backing up original state...");

            let partially_staged_files = self.getPartiallyStagedFiles().await?;

            if !partially_staged_files.is_empty() {
                let unstage_patch = self.getHiddenFilepath(PATCH_UNSTAGED);
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
        })
        .await
    }

    /// We need to remove partially staged files, to avoid tasks from seeing
    /// them.
    #[tracing::instrument(name = "GitWorkflow::prepare", skip_all)]
    pub async fn hide_unstaged_changes(self: Arc<Self>) -> Result<()> {
        self.hide_unstaged_changes_inner().await
    }

    async fn hide_unstaged_changes_inner(self: Arc<Self>) -> Result<()> {}

    #[tracing::instrument(name = "GitWorkflow::prepare", skip_all)]
    pub async fn apply_modifications(self: Arc<Self>) -> Result<()> {
        self.apply_modifications_inner().await
    }

    async fn apply_modifications_inner(self: Arc<Self>) -> Result<()> {}

    #[tracing::instrument(name = "GitWorkflow::prepare", skip_all)]
    pub async fn restore_unstaged_changes(self: Arc<Self>) -> Result<()> {
        self.restore_unstaged_changes_inner().await
    }

    async fn restore_unstaged_changes_inner(self: Arc<Self>) -> Result<()> {}

    #[tracing::instrument(name = "GitWorkflow::prepare", skip_all)]
    pub async fn restore_original_state(self: Arc<Self>) -> Result<()> {
        self.restore_original_state_inner().await
    }

    async fn restore_original_state_inner(self: Arc<Self>) -> Result<()> {}

    #[tracing::instrument(name = "GitWorkflow::prepare", skip_all)]
    pub async fn cleanup(self: Arc<Self>) -> Result<()> {
        self.cleanup_inner().await
    }

    async fn cleanup_inner(self: Arc<Self>) -> Result<()> {}
}
