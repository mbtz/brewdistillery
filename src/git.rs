use crate::errors::AppError;
use std::collections::BTreeMap;
use std::path::Path;
use std::process::{Command, Output};

#[derive(Debug, Clone, Default)]
struct RemoteEntry {
    name: String,
    urls: Vec<String>,
}

pub fn git_clone(remote: &str, dest: &Path) -> Result<(), AppError> {
    let remote = remote.trim();
    if remote.is_empty() {
        return Err(AppError::GitState(
            "tap remote is empty; set tap.remote or --tap-remote".to_string(),
        ));
    }

    let output = Command::new("git")
        .arg("clone")
        .arg(remote)
        .arg(dest)
        .output()
        .map_err(|err| AppError::GitState(format!("failed to run git clone: {err}")))?;

    if output.status.success() {
        return Ok(());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut message = format!("failed to clone tap repo from {remote}");
    if !stdout.trim().is_empty() || !stderr.trim().is_empty() {
        message.push_str(":\n");
        if !stdout.trim().is_empty() {
            message.push_str(stdout.trim());
            message.push('\n');
        }
        if !stderr.trim().is_empty() {
            message.push_str(stderr.trim());
        }
    }
    Err(AppError::GitState(message))
}

pub fn ensure_clean_repo(repo: &Path, label: &str) -> Result<(), AppError> {
    let output = run_git(repo, &["status", "--porcelain"])?;
    let status = String::from_utf8_lossy(&output.stdout);
    if status.trim().is_empty() {
        return Ok(());
    }
    Err(AppError::GitState(format!(
        "{label} has uncommitted changes; re-run with --allow-dirty to continue"
    )))
}

pub fn create_tag(repo: &Path, tag: &str) -> Result<(), AppError> {
    let exists = run_git(repo, &["tag", "--list", tag])?;
    if !String::from_utf8_lossy(&exists.stdout).trim().is_empty() {
        return Err(AppError::GitState(format!(
            "tag '{tag}' already exists; re-run with --skip-tag or choose a new version"
        )));
    }
    run_git(repo, &["tag", tag])?;
    Ok(())
}

pub fn commit_paths(repo: &Path, paths: &[&Path], message: &str) -> Result<(), AppError> {
    for path in paths {
        let relative = path
            .strip_prefix(repo)
            .map(|value| value.to_path_buf())
            .map_err(|_| {
                AppError::GitState(format!(
                    "path {} is not inside repo {}",
                    path.display(),
                    repo.display()
                ))
            })?;

        let relative = relative.to_str().ok_or_else(|| {
            AppError::GitState("path contains invalid UTF-8 for git add".to_string())
        })?;

        run_git(repo, &["add", relative])?;
    }

    let diff = run_git(repo, &["diff", "--cached", "--name-only"])?;
    if String::from_utf8_lossy(&diff.stdout).trim().is_empty() {
        return Ok(());
    }

    run_git(repo, &["commit", "-m", message])?;
    Ok(())
}

pub fn push_head(repo: &Path, configured_remote_url: Option<&str>) -> Result<(), AppError> {
    let remote = select_git_remote(repo, configured_remote_url)?;
    run_git(repo, &["push", &remote, "HEAD"])?;
    Ok(())
}

pub fn push_tag(
    repo: &Path,
    tag: &str,
    configured_remote_url: Option<&str>,
) -> Result<(), AppError> {
    let remote = select_git_remote(repo, configured_remote_url)?;
    run_git(repo, &["push", &remote, tag])?;
    Ok(())
}

pub fn run_git(repo: &Path, args: &[&str]) -> Result<Output, AppError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .map_err(|err| AppError::GitState(format!("failed to run git: {err}")))?;

    if output.status.success() {
        return Ok(output);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut message = format!("git command failed: git {}", args.join(" "));
    if !stdout.trim().is_empty() || !stderr.trim().is_empty() {
        message.push_str(":\n");
        if !stdout.trim().is_empty() {
            message.push_str(stdout.trim());
            message.push('\n');
        }
        if !stderr.trim().is_empty() {
            message.push_str(stderr.trim());
        }
    }
    Err(AppError::GitState(message))
}

pub fn select_git_remote(
    repo: &Path,
    configured_remote_url: Option<&str>,
) -> Result<String, AppError> {
    let remotes = list_git_remotes(repo)?;
    if remotes.is_empty() {
        return Err(AppError::GitState("git remote not configured".to_string()));
    }

    let configured = configured_remote_url
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if let Some(configured) = configured {
        let matches = remotes
            .iter()
            .filter(|remote| remote.urls.iter().any(|url| url == configured))
            .collect::<Vec<_>>();

        if matches.len() == 1 {
            return Ok(matches[0].name.clone());
        }
    }

    if remotes.iter().any(|remote| remote.name == "origin") {
        return Ok("origin".to_string());
    }

    if remotes.len() == 1 {
        return Ok(remotes[0].name.clone());
    }

    Err(AppError::GitState(
        "multiple git remotes found; configure origin or set a matching remote URL".to_string(),
    ))
}

fn list_git_remotes(repo: &Path) -> Result<Vec<RemoteEntry>, AppError> {
    let output = run_git(repo, &["remote", "-v"])?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut by_name: BTreeMap<String, RemoteEntry> = BTreeMap::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let mut parts = trimmed.split_whitespace();
        let Some(name) = parts.next() else {
            continue;
        };
        let Some(url) = parts.next() else {
            continue;
        };

        let entry = by_name
            .entry(name.to_string())
            .or_insert_with(|| RemoteEntry {
                name: name.to_string(),
                urls: Vec::new(),
            });

        if !entry.urls.iter().any(|existing| existing == url) {
            entry.urls.push(url.to_string());
        }
    }

    Ok(by_name.into_values().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn init_repo() -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempdir().expect("tempdir");
        let repo = dir.path().join("repo");
        std::fs::create_dir_all(&repo).expect("create repo dir");
        let status = Command::new("git")
            .args(["init", "-q"])
            .arg(&repo)
            .status()
            .expect("git init");
        assert!(status.success(), "git init should succeed");
        (dir, repo)
    }

    fn add_remote(repo: &Path, name: &str, url: &str) {
        let status = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(["remote", "add", name, url])
            .status()
            .expect("git remote add");
        assert!(status.success(), "git remote add should succeed");
    }

    #[test]
    fn selects_configured_remote_when_present() {
        let (_dir, repo) = init_repo();
        add_remote(&repo, "origin", "https://example.com/origin.git");
        add_remote(&repo, "upstream", "https://example.com/upstream.git");

        let remote = select_git_remote(&repo, Some("https://example.com/upstream.git"))
            .expect("select remote");
        assert_eq!(remote, "upstream");
    }

    #[test]
    fn falls_back_to_origin_without_configured_match() {
        let (_dir, repo) = init_repo();
        add_remote(&repo, "origin", "https://example.com/origin.git");
        add_remote(&repo, "upstream", "https://example.com/upstream.git");

        let remote =
            select_git_remote(&repo, Some("https://example.com/other.git")).expect("select remote");
        assert_eq!(remote, "origin");
    }

    #[test]
    fn falls_back_to_single_remote() {
        let (_dir, repo) = init_repo();
        add_remote(&repo, "upstream", "https://example.com/upstream.git");

        let remote = select_git_remote(&repo, None).expect("select remote");
        assert_eq!(remote, "upstream");
    }

    #[test]
    fn errors_when_multiple_remotes_and_no_origin() {
        let (_dir, repo) = init_repo();
        add_remote(&repo, "upstream", "https://example.com/upstream.git");
        add_remote(&repo, "mirror", "https://example.com/mirror.git");

        let err = select_git_remote(&repo, None).expect_err("should error");
        match err {
            AppError::GitState(message) => assert!(
                message.contains("multiple git remotes found"),
                "unexpected message: {message}"
            ),
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
