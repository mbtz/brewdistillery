use crate::errors::AppError;

pub mod github;

pub const DEFAULT_CHECKSUM_MAX_BYTES: u64 = 200 * 1024 * 1024;
pub const DEFAULT_CHECKSUM_TIMEOUT_SECS: u64 = 60;
pub const DEFAULT_CHECKSUM_MAX_RETRIES: usize = 3;
pub const DEFAULT_CHECKSUM_RETRY_BASE_DELAY_MS: u64 = 250;
pub const DEFAULT_CHECKSUM_RETRY_MAX_DELAY_MS: u64 = 2000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DownloadPolicy {
    pub timeout_secs: u64,
    pub max_retries: usize,
    pub retry_base_delay_ms: u64,
    pub retry_max_delay_ms: u64,
}

impl Default for DownloadPolicy {
    fn default() -> Self {
        Self {
            timeout_secs: DEFAULT_CHECKSUM_TIMEOUT_SECS,
            max_retries: DEFAULT_CHECKSUM_MAX_RETRIES,
            retry_base_delay_ms: DEFAULT_CHECKSUM_RETRY_BASE_DELAY_MS,
            retry_max_delay_ms: DEFAULT_CHECKSUM_RETRY_MAX_DELAY_MS,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    pub tag_name: String,
    pub draft: bool,
    pub prerelease: bool,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseAsset {
    pub name: String,
    pub download_url: String,
    pub size: Option<u64>,
}

pub trait HostClient {
    fn latest_release(
        &self,
        owner: &str,
        repo: &str,
        include_prerelease: bool,
    ) -> Result<Release, AppError>;

    fn release_by_tag(
        &self,
        owner: &str,
        repo: &str,
        tag: &str,
        include_prerelease: bool,
    ) -> Result<Release, AppError>;

    fn download_sha256(&self, url: &str, max_bytes: Option<u64>) -> Result<String, AppError>;
}
