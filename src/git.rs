//! Utils for interacting with git.
use anyhow::Result;

/// Utility for git hooks, which cannot use commands like `git add`.
#[derive(Debug)]
pub struct GitWorkflow {}

/// Methods ported from lint-staged.
impl GitWorkflow {
    /// Create a diff of partially staged files and backup stash if enabled.
    async fn prepare(&self) -> Result<()> {}

    /// We need to remove partially staged files, to avoid tasks from seeing
    /// them.
    pub async fn hide_unstaged_changes(&self, partially_staged_files: Vec<String>) -> Result<()> {}

    pub async fn apply_modifications(&self, partially_staged_files: Vec<String>) -> Result<()> {}
}
