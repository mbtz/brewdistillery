use crate::errors::AppError;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct RepoInfo {
    pub git_root: Option<PathBuf>,
}

pub fn detect_repo(start: &Path) -> Result<RepoInfo, AppError> {
    Ok(RepoInfo {
        git_root: find_git_root(start),
    })
}

fn find_git_root(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(path) = current {
        if path.join(".git").exists() {
            return Some(path.to_path_buf());
        }

        current = path.parent();
    }
    None
}
