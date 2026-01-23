use crate::errors::AppError;

pub mod github;

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
