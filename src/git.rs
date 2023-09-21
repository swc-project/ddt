//! Utils for interacting with git.
use anyhow::Result;

/// Utility for git hooks, which cannot use commands like `git add`.
#[derive(Debug)]
pub struct GitWorkflow {}

/// Methods ported from lint-staged.
impl GitWorkflow {
    /// Create a diff of partially staged files and backup stash if enabled.
    #[tracing::instrument(name = "GitWorkflow::prepare", skip_all)]
    pub async fn prepare(&self) -> Result<()> {
        self.prepare_inner().await
    }

    async fn prepare_inner(&self) -> Result<()> {}

    /// We need to remove partially staged files, to avoid tasks from seeing
    /// them.
    #[tracing::instrument(name = "GitWorkflow::prepare", skip_all)]
    pub async fn hide_unstaged_changes(&self) -> Result<()> {
        self.hide_unstaged_changes_inner().await
    }

    async fn hide_unstaged_changes_inner(&self) -> Result<()> {}

    #[tracing::instrument(name = "GitWorkflow::prepare", skip_all)]
    pub async fn apply_modifications(&self) -> Result<()> {
        self.apply_modifications_inner().await
    }

    async fn apply_modifications_inner(&self) -> Result<()> {}

    #[tracing::instrument(name = "GitWorkflow::prepare", skip_all)]
    pub async fn restore_unstaged_changes(&self) -> Result<()> {
        self.restore_unstaged_changes_inner().await
    }

    async fn restore_unstaged_changes_inner(&self) -> Result<()> {}

    #[tracing::instrument(name = "GitWorkflow::prepare", skip_all)]
    pub async fn restore_original_state(&self) -> Result<()> {
        self.restore_original_state_inner().await
    }

    async fn restore_original_state_inner(&self) -> Result<()> {}

    #[tracing::instrument(name = "GitWorkflow::prepare", skip_all)]
    pub async fn cleanup(&self) -> Result<()> {
        self.cleanup_inner().await
    }

    async fn cleanup_inner(&self) -> Result<()> {}
}
