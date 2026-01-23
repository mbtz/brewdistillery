use crate::errors::AppError;
use similar::TextDiff;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::{NamedTempFile, PersistError};

#[derive(Debug, Clone)]
pub struct PlannedWrite {
    pub path: PathBuf,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct RepoPlan {
    pub label: String,
    pub repo_root: PathBuf,
    pub writes: Vec<PlannedWrite>,
}

#[derive(Debug, Clone)]
pub struct PreviewOutput {
    pub summary: String,
    pub diff: String,
    pub changed_files: Vec<PathBuf>,
}

pub fn preview_and_apply(plans: &[RepoPlan], dry_run: bool) -> Result<PreviewOutput, AppError> {
    let (summary, diff, changed_files) = build_preview(plans)?;

    if !dry_run {
        for plan in plans {
            for write in &plan.writes {
                write_atomic(&write.path, &write.content)?;
            }
        }
    }

    Ok(PreviewOutput {
        summary,
        diff,
        changed_files,
    })
}

fn build_preview(plans: &[RepoPlan]) -> Result<(String, String, Vec<PathBuf>), AppError> {
    let mut summary = String::new();
    let mut diff_output = String::new();
    let mut changed_files = Vec::new();

    for plan in plans {
        let repo_header = format!(
            "Repo: {} ({})\n",
            plan.label,
            plan.repo_root.display()
        );
        summary.push_str(&repo_header);

        for write in &plan.writes {
            let relative = relative_path(&plan.repo_root, &write.path);
            let existing = read_optional(&write.path)?;
            let old_content = existing.as_deref().unwrap_or("");

            if old_content == write.content {
                continue;
            }

            let status = if existing.is_some() { "modified" } else { "new" };
            summary.push_str(&format!("  - {} ({})\n", relative, status));
            changed_files.push(write.path.clone());

            let diff = TextDiff::from_lines(old_content, &write.content);
            let file_header = (
                format!("a/{}", relative),
                format!("b/{}", relative),
            );
            let unified = diff
                .unified_diff()
                .context_radius(3)
                .header(&file_header.0, &file_header.1)
                .to_string();
            diff_output.push_str(&unified);
        }
    }

    Ok((summary, diff_output, changed_files))
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .ok()
        .and_then(|rel| rel.to_str())
        .map(|rel| rel.to_string())
        .unwrap_or_else(|| path.display().to_string())
}

fn read_optional(path: &Path) -> Result<Option<String>, AppError> {
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(fs::read_to_string(path)?))
}

fn write_atomic(path: &Path, content: &str) -> Result<(), AppError> {
    let parent = path.parent().ok_or_else(|| {
        AppError::InvalidInput(format!("invalid path for write: {}", path.display()))
    })?;
    fs::create_dir_all(parent)?;

    let mut temp = NamedTempFile::new_in(parent)?;
    temp.write_all(content.as_bytes())?;
    temp.flush()?;
    temp.as_file().sync_all()?;

    persist_atomic(temp, path)?;
    Ok(())
}

fn persist_atomic(temp: NamedTempFile, path: &Path) -> Result<(), AppError> {
    match temp.persist(path) {
        Ok(_) => Ok(()),
        Err(PersistError { error, file }) => {
            if path.exists() {
                fs::remove_file(path)?;
                file.persist(path)
                    .map(|_| ())
                    .map_err(|err| AppError::Io(err.error))
            } else {
                Err(AppError::Io(error))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn builds_diff_for_modified_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("Formula/test.rb");
        fs::create_dir_all(file_path.parent().unwrap()).unwrap();
        fs::write(&file_path, "old\n").unwrap();

        let plan = RepoPlan {
            label: "tap".to_string(),
            repo_root: dir.path().to_path_buf(),
            writes: vec![PlannedWrite {
                path: file_path.clone(),
                content: "new\n".to_string(),
            }],
        };

        let preview = preview_and_apply(&[plan], true).unwrap();
        assert!(preview.summary.contains("Formula/test.rb"));
        assert!(preview.diff.contains("-old"));
        assert!(preview.diff.contains("+new"));
        assert!(!preview.changed_files.is_empty());
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "old\n");
    }

    #[test]
    fn applies_writes_when_not_dry_run() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("Formula/test.rb");

        let plan = RepoPlan {
            label: "tap".to_string(),
            repo_root: dir.path().to_path_buf(),
            writes: vec![PlannedWrite {
                path: file_path.clone(),
                content: "fresh\n".to_string(),
            }],
        };

        let preview = preview_and_apply(&[plan], false).unwrap();
        assert!(preview.summary.contains("Formula/test.rb"));
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "fresh\n");
    }
}
