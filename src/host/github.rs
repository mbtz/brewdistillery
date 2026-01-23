use crate::errors::AppError;
use crate::host::{HostClient, Release, ReleaseAsset};
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use reqwest::StatusCode;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::env;
use std::io::Read;
use std::time::Duration;

const DEFAULT_API_BASE: &str = "https://api.github.com";
const DEFAULT_USER_AGENT: &str = "brewdistillery";

#[derive(Debug, Clone)]
pub struct GitHubClient {
    base_url: String,
    token: Option<String>,
    client: Client,
}

impl GitHubClient {
    pub fn from_env(api_base: Option<&str>) -> Result<Self, AppError> {
        let token = read_token();
        let base_url = api_base
            .map(|value| value.trim().trim_end_matches('/').to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| DEFAULT_API_BASE.to_string());

        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|err| AppError::Network(format!("failed to build HTTP client: {err}")))?;

        Ok(Self {
            base_url,
            token,
            client,
        })
    }

    fn request(&self, path: &str) -> reqwest::blocking::RequestBuilder {
        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        let builder = self
            .client
            .get(url)
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
                return Err(AppError::InvalidInput(format!(
                    "no GitHub releases found for {owner}/{repo}"
                )));
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
        select_latest_release(releases).ok_or_else(|| {
            AppError::InvalidInput(format!(
                "no GitHub releases found for {owner}/{repo}"
            ))
        })
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
        let mut request = self
            .client
            .get(url)
            .header(USER_AGENT, DEFAULT_USER_AGENT);
        if let Some(token) = &self.token {
            request = request.header(AUTHORIZATION, format!("Bearer {token}"));
        }

        let mut response = request
            .send()
            .map_err(|err| AppError::Network(format!("failed to download asset: {err}")))?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "failed to download asset {url}: HTTP {}",
                response.status()
            )));
        }

        if let Some(limit) = max_bytes {
            if let Some(length) = response.content_length() {
                if length > limit {
                    return Err(AppError::Network(format!(
                        "download exceeds size limit ({length} bytes > {limit} bytes): {url}"
                    )));
                }
            }
        }

        let mut hasher = Sha256::new();
        let mut total: u64 = 0;
        let mut buffer = [0u8; 8192];

        loop {
            let read = response
                .read(&mut buffer)
                .map_err(|err| AppError::Network(format!("failed to read asset: {err}")))?;
            if read == 0 {
                break;
            }
            total += read as u64;
            if let Some(limit) = max_bytes {
                if total > limit {
                    return Err(AppError::Network(format!(
                        "download exceeds size limit ({total} bytes > {limit} bytes): {url}"
                    )));
                }
            }
            hasher.update(&buffer[..read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }
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
    let message = response
        .json::<GitHubError>()
        .map(|err| err.message)
        .unwrap_or_else(|_| "unknown error".to_string());

    AppError::Network(format!(
        "GitHub API request failed ({status}) for {path}: {message}"
    ))
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
    for release in releases {
        if release.draft {
            continue;
        }
        return Some(release);
    }
    None
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
}
