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
        return Err(AppError::GitState(
            "no GitHub remote found; specify --host-owner/--host-repo".to_string(),
        ));
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
        if matches.len() > 1 {
            return Err(AppError::GitState(
                "multiple git remotes found; specify --host-owner/--host-repo".to_string(),
            ));
        }
    }

    if let Some(origin) = remotes.iter().find(|remote| remote.name == "origin") {
        let github_urls = origin
            .urls
            .iter()
            .filter(|url| is_github_url(url))
            .collect::<Vec<_>>();
        if !github_urls.is_empty() {
            if github_urls
                .iter()
                .any(|url| github_https_from_remote(url).is_some())
            {
                return Ok(origin.name.clone());
            }
            return Err(AppError::GitState(
                "unable to parse GitHub remote URL; specify --host-owner/--host-repo".to_string(),
            ));
        }
    }

    let mut github_remotes = Vec::new();
    let mut has_unparsable = false;
    for remote in &remotes {
        let mut github_urls = remote
            .urls
            .iter()
            .filter(|url| is_github_url(url))
            .collect::<Vec<_>>();
        github_urls.sort();
        github_urls.dedup();
        if github_urls.is_empty() {
            continue;
        }

        let parseable = github_urls
            .iter()
            .any(|url| github_https_from_remote(url).is_some());
        if parseable {
            github_remotes.push(remote);
        } else {
            has_unparsable = true;
        }
    }

    if github_remotes.len() == 1 {
        return Ok(github_remotes[0].name.clone());
    }

    if github_remotes.len() > 1 {
        return Err(AppError::GitState(
            "multiple git remotes found; specify --host-owner/--host-repo".to_string(),
        ));
    }

    if has_unparsable {
        return Err(AppError::GitState(
            "unable to parse GitHub remote URL; specify --host-owner/--host-repo".to_string(),
        ));
    }

    Err(AppError::GitState(
        "no GitHub remote found; specify --host-owner/--host-repo".to_string(),
    ))
}

fn github_https_from_remote(remote: &str) -> Option<String> {
    let trimmed = remote.trim();
    let path = if let Some(rest) = trimmed.strip_prefix("git@github.com:") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("ssh://git@github.com/") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("https://github.com/") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("http://github.com/") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("git://github.com/") {
        rest
    } else {
        return None;
    };

    let cleaned = path.trim_end_matches(".git").trim_end_matches('/');
    if cleaned.is_empty() {
        return None;
    }

    Some(format!("https://github.com/{cleaned}"))
}

fn is_github_url(remote: &str) -> bool {
    remote.contains("github.com")
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

    fn make_initial_commit(repo: &Path) {
        let config_email = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(["config", "user.email", "codex@example.com"])
            .status()
            .expect("git config user.email");
        assert!(
            config_email.success(),
            "git config user.email should succeed"
        );

        let config_name = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(["config", "user.name", "Codex"])
            .status()
            .expect("git config user.name");
        assert!(config_name.success(), "git config user.name should succeed");

        let config_sign = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(["config", "commit.gpgsign", "false"])
            .status()
            .expect("git config commit.gpgsign");
        assert!(
            config_sign.success(),
            "git config commit.gpgsign should succeed"
        );

        let readme = repo.join("README.md");
        std::fs::write(&readme, "init\n").expect("write README");

        let add_status = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(["add", "README.md"])
            .status()
            .expect("git add");
        assert!(add_status.success(), "git add should succeed");

        let commit_status = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(["-c", "commit.gpgsign=false", "commit", "-m", "init"])
            .status()
            .expect("git commit");
        assert!(commit_status.success(), "git commit should succeed");
    }

    #[test]
    fn selects_configured_remote_when_present() {
        let (_dir, repo) = init_repo();
        add_remote(&repo, "origin", "https://github.com/acme/origin.git");
        add_remote(&repo, "upstream", "https://github.com/acme/upstream.git");

        let remote = select_git_remote(&repo, Some("https://github.com/acme/upstream.git"))
            .expect("select remote");
        assert_eq!(remote, "upstream");
    }

    #[test]
    fn falls_back_to_origin_without_configured_match() {
        let (_dir, repo) = init_repo();
        add_remote(&repo, "origin", "https://github.com/acme/origin.git");
        add_remote(&repo, "upstream", "https://github.com/acme/upstream.git");

        let remote = select_git_remote(&repo, Some("https://github.com/acme/other.git"))
            .expect("select remote");
        assert_eq!(remote, "origin");
    }

    #[test]
    fn falls_back_to_single_remote() {
        let (_dir, repo) = init_repo();
        add_remote(&repo, "upstream", "https://github.com/acme/upstream.git");

        let remote = select_git_remote(&repo, None).expect("select remote");
        assert_eq!(remote, "upstream");
    }

    #[test]
    fn errors_when_multiple_remotes_and_no_origin() {
        let (_dir, repo) = init_repo();
        add_remote(&repo, "upstream", "https://github.com/acme/upstream.git");
        add_remote(&repo, "mirror", "https://github.com/acme/mirror.git");

        let err = select_git_remote(&repo, None).expect_err("should error");
        assert_eq!(
            err.to_string(),
            "multiple git remotes found; specify --host-owner/--host-repo"
        );
    }

    #[test]
    fn ignores_non_github_origin_when_single_github_remote_exists() {
        let (_dir, repo) = init_repo();
        add_remote(&repo, "origin", "https://gitlab.com/acme/origin.git");
        add_remote(&repo, "upstream", "https://github.com/acme/upstream.git");

        let remote = select_git_remote(&repo, None).expect("select remote");
        assert_eq!(remote, "upstream");
    }

    #[test]
    fn errors_when_tag_already_exists() {
        let (_dir, repo) = init_repo();
        make_initial_commit(&repo);

        create_tag(&repo, "v1.2.3").expect("create initial tag");
        let err = create_tag(&repo, "v1.2.3").expect_err("tag should already exist");
        assert_eq!(
            err.to_string(),
            "tag 'v1.2.3' already exists; re-run with --skip-tag or choose a new version"
        );
    }
}
