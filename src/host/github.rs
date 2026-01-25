use crate::errors::AppError;
use crate::host::{DownloadPolicy, HostClient, Release, ReleaseAsset};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, ACCEPT, AUTHORIZATION, USER_AGENT};
use reqwest::Method;
use reqwest::StatusCode;
use serde::Serialize;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::env;
use std::io::Read;
use std::time::Duration;

const DEFAULT_API_BASE: &str = "https://api.github.com";
const DEFAULT_USER_AGENT: &str = "brewdistillery";

enum DownloadFailure {
    Retryable(String),
    Fatal(AppError),
}

#[derive(Debug, Clone)]
pub struct GitHubClient {
    base_url: String,
    token: Option<String>,
    client: Client,
    download_policy: DownloadPolicy,
}

impl GitHubClient {
    pub fn from_env(
        api_base: Option<&str>,
        download_policy: DownloadPolicy,
    ) -> Result<Self, AppError> {
        let token = read_token();
        let base_url = api_base
            .map(|value| value.trim().trim_end_matches('/').to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| DEFAULT_API_BASE.to_string());

        let timeout_secs = download_policy.timeout_secs.max(1);
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .map_err(|err| AppError::Network(format!("failed to build HTTP client: {err}")))?;

        Ok(Self {
            base_url,
            token,
            client,
            download_policy,
        })
    }

    fn request(&self, path: &str) -> reqwest::blocking::RequestBuilder {
        self.request_with_method(Method::GET, path)
    }

    fn request_with_method(
        &self,
        method: Method,
        path: &str,
    ) -> reqwest::blocking::RequestBuilder {
        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        let builder = self
            .client
            .request(method, url)
            .header(USER_AGENT, DEFAULT_USER_AGENT)
            .header(ACCEPT, "application/vnd.github+json");
        if let Some(token) = &self.token {
            builder.header(AUTHORIZATION, format!("Bearer {token}"))
        } else {
            builder
        }
    }

    fn get_json<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T, AppError> {
        let response = self.request(path).send().map_err(|err| {
            AppError::Network(format!("GitHub API request failed: {err}"))
        })?;

        if !response.status().is_success() {
            return Err(map_github_error(path, response));
        }

        response.json().map_err(|err| {
            AppError::Network(format!("failed to parse GitHub API response: {err}"))
        })
    }

    fn require_token(&self) -> Result<&str, AppError> {
        self.token
            .as_deref()
            .filter(|token| !token.trim().is_empty())
            .ok_or_else(|| {
                AppError::Network(
                    "GitHub token missing; set GITHUB_TOKEN or GH_TOKEN to create the tap repo"
                        .to_string(),
                )
            })
    }

    fn authenticated_user(&self) -> Result<GitHubUser, AppError> {
        self.require_token()?;
        self.get_json("/user")
    }

    pub fn create_public_repo(&self, owner: &str, repo: &str) -> Result<CreatedRepo, AppError> {
        let owner = owner.trim();
        let repo = repo.trim();
        if owner.is_empty() {
            return Err(AppError::InvalidInput(
                "tap owner cannot be empty".to_string(),
            ));
        }
        if repo.is_empty() {
            return Err(AppError::InvalidInput(
                "tap repo cannot be empty".to_string(),
            ));
        }

        self.require_token()?;

        let user = self.authenticated_user()?;
        let path = if owner.eq_ignore_ascii_case(&user.login) {
            "/user/repos".to_string()
        } else {
            format!("/orgs/{owner}/repos")
        };

        let payload = CreateRepoRequest {
            name: repo.to_string(),
            private: false,
        };

        let response = self
            .request_with_method(Method::POST, &path)
            .json(&payload)
            .send()
            .map_err(|err| {
                AppError::Network(format!("GitHub API request failed: {err}"))
            })?;

        if response.status().is_success() {
            let created: GitHubRepo = response.json().map_err(|err| {
                AppError::Network(format!("failed to parse GitHub API response: {err}"))
            })?;
            return Ok(created.into());
        }

        Err(map_repo_create_error(
            owner,
            repo,
            &path,
            response,
        ))
    }

    fn get_release_by_tag(
        &self,
        owner: &str,
        repo: &str,
        tag: &str,
    ) -> Result<GitHubRelease, AppError> {
        let path = format!("/repos/{owner}/{repo}/releases/tags/{tag}");
        let response = self.request(&path).send().map_err(|err| {
            AppError::Network(format!("GitHub API request failed: {err}"))
        })?;

        if response.status() == StatusCode::NOT_FOUND {
            return Err(AppError::InvalidInput(format!(
                "GitHub release tag '{tag}' not found for {owner}/{repo}"
            )));
        }

        if !response.status().is_success() {
            return Err(map_github_error(&path, response));
        }

        response.json().map_err(|err| {
            AppError::Network(format!("failed to parse GitHub API response: {err}"))
        })
    }

    fn latest_release_impl(
        &self,
        owner: &str,
        repo: &str,
        include_prerelease: bool,
    ) -> Result<GitHubRelease, AppError> {
        if !include_prerelease {
            let path = format!("/repos/{owner}/{repo}/releases/latest");
            let response = self.request(&path).send().map_err(|err| {
                AppError::Network(format!("GitHub API request failed: {err}"))
            })?;

            if response.status() == StatusCode::NOT_FOUND {
                return Err(no_releases_error(owner, repo));
            }

            if !response.status().is_success() {
                return Err(map_github_error(&path, response));
            }

            return response.json().map_err(|err| {
                AppError::Network(format!("failed to parse GitHub API response: {err}"))
            });
        }

        let path = format!("/repos/{owner}/{repo}/releases?per_page=100");
        let releases: Vec<GitHubRelease> = self.get_json(&path)?;
        select_latest_release(releases).ok_or_else(|| no_releases_error(owner, repo))
    }
}

impl HostClient for GitHubClient {
    fn latest_release(
        &self,
        owner: &str,
        repo: &str,
        include_prerelease: bool,
    ) -> Result<Release, AppError> {
        let release = self.latest_release_impl(owner, repo, include_prerelease)?;
        ensure_release_allowed(&release, include_prerelease)?;
        Ok(release.into())
    }

    fn release_by_tag(
        &self,
        owner: &str,
        repo: &str,
        tag: &str,
        include_prerelease: bool,
    ) -> Result<Release, AppError> {
        let release = self.get_release_by_tag(owner, repo, tag)?;
        ensure_release_allowed(&release, include_prerelease)?;
        Ok(release.into())
    }

    fn download_sha256(&self, url: &str, max_bytes: Option<u64>) -> Result<String, AppError> {
        let max_retries = self.download_policy.max_retries.max(1);
        for attempt in 1..=max_retries {
            match self.download_sha256_once(url, max_bytes) {
                Ok(sha) => return Ok(sha),
                Err(DownloadFailure::Retryable(message)) => {
                    if attempt < max_retries {
                        std::thread::sleep(retry_delay(attempt, &self.download_policy));
                        continue;
                    }
                    return Err(AppError::Network(format!(
                        "{message} (after {max_retries} attempts)"
                    )));
                }
                Err(DownloadFailure::Fatal(err)) => return Err(err),
            }
        }

        Err(AppError::Network("failed to download asset".to_string()))
    }
}

impl GitHubClient {
    fn download_sha256_once(
        &self,
        url: &str,
        max_bytes: Option<u64>,
    ) -> Result<String, DownloadFailure> {
        let mut request = self
            .client
            .get(url)
            .header(USER_AGENT, DEFAULT_USER_AGENT);
        if let Some(token) = &self.token {
            request = request.header(AUTHORIZATION, format!("Bearer {token}"));
        }

        let mut response = request.send().map_err(|err| {
            DownloadFailure::Retryable(format!("failed to download asset: {err}"))
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let message = format!("failed to download asset {url}: HTTP {status}");
            if should_retry_status(status) {
                return Err(DownloadFailure::Retryable(message));
            }
            return Err(DownloadFailure::Fatal(AppError::Network(message)));
        }

        if let Some(limit) = max_bytes {
            if let Some(length) = response.content_length() {
                if length > limit {
                    return Err(DownloadFailure::Fatal(AppError::Network(format!(
                        "download exceeds size limit ({length} bytes > {limit} bytes): {url}"
                    ))));
                }
            }
        }

        let mut hasher = Sha256::new();
        let mut total: u64 = 0;
        let mut buffer = [0u8; 8192];

        loop {
            let read = response.read(&mut buffer).map_err(|err| {
                DownloadFailure::Retryable(format!("failed to read asset: {err}"))
            })?;
            if read == 0 {
                break;
            }
            total += read as u64;
            if let Some(limit) = max_bytes {
                if total > limit {
                    return Err(DownloadFailure::Fatal(AppError::Network(format!(
                        "download exceeds size limit ({total} bytes > {limit} bytes): {url}"
                    ))));
                }
            }
            hasher.update(&buffer[..read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }
}

fn should_retry_status(status: StatusCode) -> bool {
    status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
}

fn retry_delay(attempt: usize, policy: &DownloadPolicy) -> Duration {
    let shift = attempt.saturating_sub(1);
    let exp = 1u64 << shift;
    let base = policy.retry_base_delay_ms.max(1);
    let cap = policy.retry_max_delay_ms.max(base);
    let delay = base.saturating_mul(exp);
    Duration::from_millis(delay.min(cap))
}

fn read_token() -> Option<String> {
    env::var("GITHUB_TOKEN")
        .ok()
        .or_else(|| env::var("GH_TOKEN").ok())
        .and_then(|token| {
            let trimmed = token.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        })
}

fn map_github_error(path: &str, response: reqwest::blocking::Response) -> AppError {
    let status = response.status();
    let headers = response.headers().clone();
    let message = response
        .json::<GitHubError>()
        .map(|err| err.message)
        .unwrap_or_else(|_| "unknown error".to_string());

    map_github_error_message(path, status, &headers, &message)
}

fn map_repo_create_error(
    owner: &str,
    repo: &str,
    path: &str,
    response: reqwest::blocking::Response,
) -> AppError {
    let status = response.status();
    let headers = response.headers().clone();
    let message = response
        .json::<GitHubError>()
        .map(|err| err.message)
        .unwrap_or_else(|_| "unknown error".to_string());

    if status == StatusCode::UNPROCESSABLE_ENTITY {
        return AppError::InvalidInput(format!(
            "GitHub repo '{owner}/{repo}' cannot be created: {message}",
        ));
    }

    if status == StatusCode::NOT_FOUND {
        return AppError::Network(format!(
            "GitHub organization '{owner}' not found or token lacks access"
        ));
    }

    map_github_error_message(path, status, &headers, &message)
}

fn map_github_error_message(
    path: &str,
    status: StatusCode,
    headers: &HeaderMap,
    message: &str,
) -> AppError {
    if status == StatusCode::UNAUTHORIZED {
        return AppError::Network(
            "GitHub API authentication failed; set GITHUB_TOKEN or GH_TOKEN".to_string(),
        );
    }

    if status == StatusCode::FORBIDDEN {
        if is_rate_limited(headers, message) {
            return AppError::Network(rate_limit_message(headers));
        }
        return AppError::Network(format!(
            "GitHub API permission denied for {path}: {message}. Ensure the token has repo access",
        ));
    }

    AppError::Network(format!(
        "GitHub API request failed ({status}) for {path}: {message}"
    ))
}

fn is_rate_limited(headers: &HeaderMap, message: &str) -> bool {
    if message.to_ascii_lowercase().contains("rate limit") {
        return true;
    }
    headers
        .get("x-ratelimit-remaining")
        .and_then(|value| value.to_str().ok())
        .map(|value| value == "0")
        .unwrap_or(false)
}

fn rate_limit_message(headers: &HeaderMap) -> String {
    let reset = headers
        .get("x-ratelimit-reset")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok());
    if let Some(reset) = reset {
        format!("GitHub API rate limit exceeded; retry after {reset} (unix epoch)")
    } else {
        "GitHub API rate limit exceeded; retry later".to_string()
    }
}

fn no_releases_error(owner: &str, repo: &str) -> AppError {
    AppError::InvalidInput(format!("no GitHub releases found for {owner}/{repo}"))
}

fn ensure_release_allowed(
    release: &GitHubRelease,
    include_prerelease: bool,
) -> Result<(), AppError> {
    if release.draft {
        return Err(AppError::InvalidInput(format!(
            "GitHub release '{}' is a draft; publish it before releasing",
            release.tag_name
        )));
    }
    if release.prerelease && !include_prerelease {
        return Err(AppError::InvalidInput(format!(
            "GitHub release '{}' is a prerelease; re-run with --include-prerelease",
            release.tag_name
        )));
    }
    Ok(())
}

fn select_latest_release(releases: Vec<GitHubRelease>) -> Option<GitHubRelease> {
    // GitHub returns releases newest-first. We select the first non-draft
    // release so that --include-prerelease can pick the newest prerelease
    // when it appears ahead of the latest stable.
    releases.into_iter().find(|release| !release.draft)
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

#[derive(Debug, Deserialize)]
struct GitHubError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct GitHubUser {
    login: String,
}

#[derive(Debug, Deserialize)]
struct GitHubRepo {
    clone_url: String,
    ssh_url: Option<String>,
    html_url: Option<String>,
    full_name: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreateRepoRequest {
    name: String,
    private: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatedRepo {
    pub clone_url: String,
    pub ssh_url: Option<String>,
    pub html_url: Option<String>,
    pub full_name: Option<String>,
}

impl From<GitHubRepo> for CreatedRepo {
    fn from(value: GitHubRepo) -> Self {
        Self {
            clone_url: value.clone_url,
            ssh_url: value.ssh_url,
            html_url: value.html_url,
            full_name: value.full_name,
        }
    }
}

impl From<GitHubRelease> for Release {
    fn from(value: GitHubRelease) -> Self {
        Release {
            tag_name: value.tag_name,
            draft: value.draft,
            prerelease: value.prerelease,
            assets: value
                .assets
                .into_iter()
                .map(|asset| ReleaseAsset {
                    name: asset.name,
                    download_url: asset.browser_download_url,
                    size: Some(asset.size),
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_first_non_draft_release() {
        let releases = vec![
            GitHubRelease {
                tag_name: "v1.0.0".to_string(),
                draft: true,
                prerelease: false,
                assets: vec![],
            },
            GitHubRelease {
                tag_name: "v1.1.0".to_string(),
                draft: false,
                prerelease: true,
                assets: vec![],
            },
        ];

        let selected = select_latest_release(releases).unwrap();
        assert_eq!(selected.tag_name, "v1.1.0");
    }

    #[test]
    fn include_prerelease_selects_latest_prerelease_when_ordered_first() {
        let releases = vec![
            GitHubRelease {
                tag_name: "v2.0.0-rc.1".to_string(),
                draft: false,
                prerelease: true,
                assets: vec![],
            },
            GitHubRelease {
                tag_name: "v1.9.0".to_string(),
                draft: false,
                prerelease: false,
                assets: vec![],
            },
        ];

        let selected = select_latest_release(releases).unwrap();
        assert_eq!(selected.tag_name, "v2.0.0-rc.1");
        assert!(selected.prerelease);
    }

    #[test]
    fn formats_no_releases_error_message() {
        let err = no_releases_error("acme", "brewtool");
        assert_eq!(err.to_string(), "no GitHub releases found for acme/brewtool");
    }

    #[test]
    fn rejects_prerelease_when_flag_is_not_set() {
        let release = GitHubRelease {
            tag_name: "v1.2.0-beta.1".to_string(),
            draft: false,
            prerelease: true,
            assets: vec![],
        };

        let err = ensure_release_allowed(&release, false).unwrap_err();
        assert_eq!(
            err.to_string(),
            "GitHub release 'v1.2.0-beta.1' is a prerelease; re-run with --include-prerelease"
        );
    }

    #[test]
    fn detects_rate_limit_from_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("x-ratelimit-remaining", "0".parse().unwrap());
        assert!(is_rate_limited(&headers, "API rate limit exceeded"));
    }

    #[test]
    fn formats_rate_limit_message_with_reset() {
        let mut headers = HeaderMap::new();
        headers.insert("x-ratelimit-reset", "1700000000".parse().unwrap());
        let message = rate_limit_message(&headers);
        assert!(message.contains("1700000000"));
    }

    #[test]
    fn retries_on_server_errors_and_rate_limits() {
        assert!(should_retry_status(StatusCode::TOO_MANY_REQUESTS));
        assert!(should_retry_status(StatusCode::INTERNAL_SERVER_ERROR));
        assert!(!should_retry_status(StatusCode::BAD_REQUEST));
    }

    #[test]
    fn retry_delay_grows_and_caps() {
        let policy = DownloadPolicy::default();
        let first = retry_delay(1, &policy);
        let second = retry_delay(2, &policy);
        let third = retry_delay(3, &policy);
        assert!(second > first);
        assert!(third >= second);
        assert!(third <= Duration::from_millis(policy.retry_max_delay_ms));
    }
}
