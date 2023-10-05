#![allow(unused)]

//! Utils for interacting with git.
use std::{path::PathBuf, sync::Arc};

use anyhow::{bail, Context, Result};
use futures::{try_join, Future};
use once_cell::sync::Lazy;
use regex::Regex;
use tokio::fs;
use tracing::{debug, info, warn};

use crate::util::{wrap, PrettyCmd};

const STASH: &str = "ddt-git-workflow automatic backup";

const PATCH_UNSTAGED: &str = "ddt-git-workflow_unstaged.patch";

static GIT_DIFF_ARGS: &[&str] = &[
    "--binary",          // support binary files
    "--unified=0",       // do not add lines around diff for consistent behaviour
    "--no-color",        // disable colors for consistent behaviour
    "--no-ext-diff",     // disable external diff tools for consistent behaviour
    "--src-prefix=a/",   // force prefix for consistent behaviour
    "--dst-prefix=b/",   // force prefix for consistent behaviour
    "--patch",           // output a patch that can be applied
    "--submodule=short", // always use the default short format for submodules
];

static GIT_APPLY_ARGS: &[&str] = &["-v", "--whitespace=nowarn", "--recount", "--unidiff-zero"];

/// Utility for git hooks, which cannot use commands like `git add`.
///
/// Any modification of the files in staging area (from hook) **must** be done
/// in the git workflow.

#[derive(Debug)]
pub struct GitWorkflow {
    matched_file_chunks: Arc<Vec<Vec<String>>>,
    git_dir: Arc<PathBuf>,
    git_config_dir: Arc<PathBuf>,

    allow_empty: bool,
    diff: Option<String>,
    diff_filter: Option<String>,

    merge_head_filename: PathBuf,
    merge_mode_filename: PathBuf,
    merge_msg_filename: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PrepareResult {
    partially_staged_files: Arc<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct MergeStatus {
    header: Option<Vec<u8>>,
    mode: Option<Vec<u8>>,
    msg: Option<Vec<u8>>,
}

const MERGE_HEAD: &str = "MERGE_HEAD";
const MERGE_MODE: &str = "MERGE_MODE";
const MERGE_MSG: &str = "MERGE_MSG";

/// Methods ported from lint-staged.
impl GitWorkflow {
    pub fn new(
        matched_file_chunks: Arc<Vec<Vec<String>>>,
        git_dir: Arc<PathBuf>,
        git_config_dir: Arc<PathBuf>,
        allow_empty: bool,
        diff: Option<String>,
        diff_filter: Option<String>,
    ) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            merge_head_filename: git_config_dir.join(MERGE_HEAD),
            merge_mode_filename: git_config_dir.join(MERGE_MODE),
            merge_msg_filename: git_config_dir.join(MERGE_MSG),

            matched_file_chunks,
            git_dir,
            git_config_dir,
            allow_empty,
            diff,
            diff_filter,
        }))
    }

    #[tracing::instrument(name = "GitWorkflow::backup_merge_status", skip_all)]
    pub async fn backup_merge_status(self: Arc<Self>) -> Result<MergeStatus> {
        wrap(async move { self.backup_merge_status_inner().await })
            .await
            .context("failed to backup merge status")
    }

    async fn backup_merge_status_inner(self: Arc<Self>) -> Result<MergeStatus> {
        debug!("Backing up merge state...");

        let (header, mode, msg) = try_join!(
            fs::read(&*self.merge_head_filename),
            fs::read(&*self.merge_mode_filename),
            fs::read(&*self.merge_msg_filename)
        )
        .context("failed to read merge file")?;

        debug!("Done backing up merge state!");

        Ok(MergeStatus {
            header: Some(header),
            mode: Some(mode),
            msg: Some(msg),
        })
    }

    #[tracing::instrument(name = "GitWorkflow::restore_merge_status", skip_all)]
    pub async fn restore_merge_status(self: Arc<Self>, status: MergeStatus) -> Result<()> {
        wrap(async move { self.restore_merge_status_inner(status).await })
            .await
            .context("failed to restore merge status")
    }

    async fn restore_merge_status_inner(self: Arc<Self>, status: MergeStatus) -> Result<()> {
        async fn w(path: PathBuf, content: Option<Vec<u8>>) -> Result<()> {
            if let Some(content) = content {
                fs::write(path, content)
                    .await
                    .context("failed to write merge status file")?;
            }

            Ok(())
        }

        try_join!(
            w(self.merge_head_filename.clone(), status.header),
            w(self.merge_mode_filename.clone(), status.mode),
            w(self.merge_msg_filename.clone(), status.msg),
        )?;

        Ok(())
    }

    /// Get a list of all files with both staged and unstaged modifications.
    /// Renames have special treatment, since the single status line includes
    /// both the "from" and "to" filenames, where "from" is no longer on disk.
    #[tracing::instrument(name = "GitWorkflow::get_partially_staged_files", skip_all)]
    pub async fn get_partially_staged_files(self: Arc<Self>) -> Result<Vec<String>> {
        wrap(async move { self.get_partially_staged_files_inner().await })
            .await
            .context("failed to get partially staged files")
    }

    async fn get_partially_staged_files_inner(self: Arc<Self>) -> Result<Vec<String>> {
        static SPLIT_RE: Lazy<Regex> =
            Lazy::new(|| Regex::new("\x00(?=[ AMDRCU?!]{2} |$)").unwrap());

        debug!("Getting partially staged files...");

        let status = self.exec_git(vec!["status".into(), "-z".into()]).await?;

        let res = SPLIT_RE
            .captures_iter(&status)
            .filter(|line| {
                let index = line.get(0).expect("index should exist").as_str();
                let working_tree = line.get(1).expect("working tree should exist").as_str();

                index != " " && working_tree != " " && index != "?" && working_tree != "?"
            })
            .map(|l| l.get(0).expect("index should exist").as_str()[3..].to_string())
            .collect::<Vec<_>>();

        debug!("Found partially staged files: {res:?}");

        Ok(res)
    }

    /// Create a diff of partially staged files and backup stash if enabled.
    #[tracing::instrument(name = "GitWorkflow::prepare", skip_all)]
    pub async fn prepare(self: Arc<Self>) -> Result<PrepareResult> {
        wrap(async move { self.prepare_inner().await })
            .await
            .context("failed to prepare a git workflow")
    }

    async fn prepare_inner(self: Arc<Self>) -> Result<PrepareResult> {
        debug!("Backing up original state...");

        let partially_staged_files = self.clone().get_partially_staged_files().await?;

        if !partially_staged_files.is_empty() {
            let unstage_patch = self
                .clone()
                .get_hidden_filepath(PATCH_UNSTAGED)
                .context("failed to get the path for the unstage patch file")?;
            let files = process_renames(&partially_staged_files, true);

            let mut args = vec![String::from("diff")];
            args.extend(GIT_DIFF_ARGS.iter().map(|v| v.to_string()));
            args.push("--output".into());

            args.push(unstage_patch.display().to_string());
            args.push("--".into());
            args.extend(files);

            self.exec_git(args).await?;
        }

        // TODO: https://github.com/okonet/lint-staged/blob/19a6527c8ac07dbafa2b8c1774e849d3cab635c3/lib/gitWorkflow.js#L210-L229

        Ok(PrepareResult {
            partially_staged_files: Arc::new(partially_staged_files),
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
        let files = process_renames(&partially_staged_files, false);

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
            self.clone().exec_git(args).await?;
        }

        debug!("Done adding task modifications to index!");

        let staged_files_after_add = self
            .clone()
            .exec_git(get_diff_command(
                self.diff.as_deref(),
                self.diff_filter.as_deref(),
            ))
            .await?;
        if staged_files_after_add.is_empty() && !self.allow_empty {
            // Tasks reverted all staged changes and the commit would be empty
            // Throw error to stop commit unless `--allow-empty` was used
            bail!("Prevented an empty git commit!")
        }

        Ok(())
    }

    #[tracing::instrument(name = "GitWorkflow::restore_unstaged_changes", skip_all)]
    pub async fn restore_unstaged_changes(self: Arc<Self>) -> Result<()> {
        wrap(async move { self.restore_unstaged_changes_inner().await })
            .await
            .context("failed to restore unstaged changes")
    }

    async fn restore_unstaged_changes_inner(self: Arc<Self>) -> Result<()> {
        debug!("Restoring unstaged changes...");

        let unstaged_patch = self.get_hidden_filepath(PATCH_UNSTAGED)?;

        let result = {
            let mut args = vec!["apply".into()];
            args.extend(GIT_APPLY_ARGS.iter().map(|v| v.to_string()));
            args.push(unstaged_patch.display().to_string());
            self.clone().exec_git(args).await
        };

        match result {
            Ok(_) => return Ok(()),
            Err(err) => {
                warn!("Error while restoring changes:'{:?}'", err);
                info!("Retrying with 3-way merge");

                let result = {
                    let mut args = vec!["apply".into()];
                    args.extend(GIT_APPLY_ARGS.iter().map(|v| v.to_string()));
                    args.push("--3way".into());
                    args.push(unstaged_patch.display().to_string());
                    self.exec_git(args).await
                };

                match result {
                    Ok(_) => Ok(()),
                    Err(_) => Err(err.context(
                        "Unstaged changes could not be restored due to a merge conflict!",
                    )),
                }
            }
        }
    }

    #[tracing::instrument(name = "GitWorkflow::restore_original_state", skip_all)]
    pub async fn restore_original_state(self: Arc<Self>, merge_status: MergeStatus) -> Result<()> {
        wrap(async move { self.restore_original_state_inner(merge_status).await })
            .await
            .context("failed to restore original state")
    }

    async fn restore_original_state_inner(
        self: Arc<Self>,
        merge_status: MergeStatus,
    ) -> Result<()> {
        debug!("Restoring original state...");

        self.clone()
            .exec_git(vec!["reset".into(), "--hard".into(), "HEAD".into()])
            .await?;

        {
            let stash_path = self.clone().get_backup_stash().await?;

            self.clone()
                .exec_git(vec![
                    "stash".into(),
                    "apply".into(),
                    "--quiet".into(),
                    "--index".into(),
                    stash_path,
                ])
                .await?;
        }

        // Restore meta information about ongoing git merge
        self.clone().restore_merge_status(merge_status).await?;

        // If stashing resurrected deleted files, clean them out

        // TODO(kdy1):
        //   await Promise.all(this.deletedFiles.map((file) => unlink(file)))

        {
            // Clean out patch
            let patch_file = self.get_hidden_filepath(PATCH_UNSTAGED)?;
            fs::remove_file(&patch_file)
                .await
                .context("failed to remove patch file")?
        };

        debug!("Done restoring original state!");

        Ok(())
    }

    #[tracing::instrument(name = "GitWorkflow::cleanup", skip_all)]
    pub async fn cleanup(self: Arc<Self>) -> Result<()> {
        wrap(async move { self.cleanup_inner().await })
            .await
            .context("failed to cleanup")
    }

    async fn cleanup_inner(self: Arc<Self>) -> Result<()> {
        debug!("Dropping backup stash...");

        let backup_stash = self.clone().get_backup_stash().await?;
        let args = vec![
            String::from("stash"),
            "drop".into(),
            "--quiet".into(),
            backup_stash,
        ];
        self.exec_git(args).await?;

        debug!("Done dropping backup stash!");

        Ok(())
    }

    #[tracing::instrument(name = "GitWorkflow::exec_git", skip_all)]
    async fn exec_git(self: Arc<Self>, args: Vec<String>) -> Result<String> {
        self.exec_git_inner(args).await
    }

    async fn exec_git_inner(self: Arc<Self>, args: Vec<String>) -> Result<String> {
        let output = PrettyCmd::new("Running git command", "git")
            .dir(&*self.git_dir)
            .args(&["-c", "submodule.recurse=false"])
            .args(args)
            .output()
            .await
            .context("git command failed")?;

        Ok(output)
    }

    fn get_hidden_filepath(&self, filename: &str) -> Result<PathBuf> {
        self.git_config_dir
            .join(filename)
            .canonicalize()
            .context("failed to get hidden filepath")
    }

    #[tracing::instrument(name = "GitWorkflow::get_backup_stash", skip_all)]
    async fn get_backup_stash(self: Arc<Self>) -> Result<String> {
        wrap(async move { self.get_backup_stash_inner().await })
            .await
            .context("failed to get backup stash")
    }

    async fn get_backup_stash_inner(self: Arc<Self>) -> Result<String> {
        let stashes = self.exec_git(vec!["stash".into(), "list".into()]).await?;

        let idx = stashes.lines().find(|line| line.contains(STASH));

        match idx {
            Some(v) => Ok(v.to_string()),
            None => bail!("ddt-stash automatic backup is missing!"),
        }
    }
}

/// In git status machine output, renames are presented as `to`NUL`from`
/// When diffing, both need to be taken into account, but in some cases on the
/// `to`.
///
/// Ported from https://github.com/okonet/lint-staged/blob/19a6527c8ac07dbafa2b8c1774e849d3cab635c3/lib/gitWorkflow.js#L29-L44
fn process_renames(files: &[String], include_rename_from: bool) -> Vec<String> {
    files.into_iter().fold(vec![], |mut flattened, file| {
        if let Some(idx) = file.find('\0') {
            let (to, from) = file.split_at(idx);

            if include_rename_from {
                flattened.push(from.to_string());
            }
            flattened.push(to.to_string());
        } else {
            flattened.push(file.to_string());
        }

        flattened
    })
}

/// Ported from https://github.com/okonet/lint-staged/blob/19a6527c8ac07dbafa2b8c1774e849d3cab635c3/lib/getDiffCommand.js#L1
fn get_diff_command(diff: Option<&str>, diff_filter: Option<&str>) -> Vec<String> {
    let diff_filter_arg = diff_filter.map_or("ACMR", |s| s.trim());

    let diff_args: Vec<&str> = diff.map_or(vec!["staged"], |s| s.trim().split(' ').collect());

    let mut args = vec![
        "diff".into(),
        "--name-only".into(),
        "-z".into(),
        format!("--diff-filter={diff_filter_arg}"),
    ];
    args.extend(diff_args.iter().map(|s| s.to_string()));

    args
}
