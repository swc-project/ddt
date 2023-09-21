//! Utils for interacting with git.

#[derive(Debug)]
pub struct GitWorkflow {}

/// Methods ported from lint-staged.
impl GitWorkflow {
    /// We need to remove partially staged files, to avoid tasks from seeing
    /// them.
    pub async fn hide_unstaged_changes(&self, partially_staged_files: Vec<String>) -> Result<()> {}

    pub async fn apply_modifications(&self, partially_staged_files: Vec<String>) -> Result<()> {}
}
