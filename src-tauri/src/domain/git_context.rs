//! Platform-neutral Git repository context used only for presentation.

use std::path::{Path, PathBuf};

/// A repository root and the symbolic branch associated with a working directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitContext {
    pub repository_root: PathBuf,
    pub branch: GitBranch,
}

/// Detached HEAD is explicit; an empty branch name is never used as a sentinel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitBranch {
    Named(String),
    DetachedHead,
}

/// Read-only lookup from an observed process working directory.
///
/// No context is a normal result for non-repositories, unavailable Git, and
/// inaccessible or stale paths. Implementations must not execute project code.
pub trait GitContextProvider: Send + Sync {
    fn context_for(&self, working_directory: &Path) -> Option<GitContext>;
}
