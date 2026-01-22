use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub schema_version: Option<u32>,

    #[serde(default)]
    pub project: ProjectConfig,

    #[serde(default)]
    pub cli: RepoConfig,

    #[serde(default)]
    pub tap: TapConfig,

    #[serde(default)]
    pub artifact: ArtifactConfig,

    #[serde(default)]
    pub release: ReleaseConfig,

    #[serde(default)]
    pub version_update: VersionUpdateConfig,

    #[serde(default)]
    pub host: HostConfig,

    #[serde(default)]
    pub template: TemplateConfig,

    #[serde(flatten)]
    pub extra: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: Option<String>,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub license: Option<String>,

    #[serde(default)]
    pub bin: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepoConfig {
    pub owner: Option<String>,
    pub repo: Option<String>,
    pub remote: Option<String>,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TapConfig {
    pub owner: Option<String>,
    pub repo: Option<String>,
    pub remote: Option<String>,
    pub path: Option<PathBuf>,
    pub formula: Option<String>,
    pub formula_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArtifactConfig {
    pub owner: Option<String>,
    pub repo: Option<String>,
    pub strategy: Option<String>,
    pub asset_template: Option<String>,
    pub asset_name: Option<String>,

    #[serde(default)]
    pub targets: BTreeMap<String, ArtifactTarget>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArtifactTarget {
    pub asset_template: Option<String>,
    pub asset_name: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReleaseConfig {
    pub tag_format: Option<String>,
    pub tarball_url_template: Option<String>,
    pub commit_message_template: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VersionUpdateConfig {
    pub mode: Option<String>,
    pub cargo_package: Option<String>,
    pub regex_file: Option<PathBuf>,
    pub regex_pattern: Option<String>,
    pub regex_replacement: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostConfig {
    pub provider: Option<String>,
    pub api_base: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub path: Option<PathBuf>,
    pub install_block: Option<String>,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, AppError> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)?;
        let config = Self::from_str(&content, path)?;
        config.validate(path)?;
        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<(), AppError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        self.validate(path)?;
        let content = toml::to_string_pretty(self).map_err(|err| {
            AppError::InvalidInput(format!("failed to serialize config {}: {err}", path.display()))
        })?;

        let tmp_path = path.with_extension("toml.tmp");
        fs::write(&tmp_path, format!("{content}\n"))?;
        fs::rename(tmp_path, path)?;
        Ok(())
    }

    fn from_str(content: &str, path: &Path) -> Result<Self, AppError> {
        toml::from_str(content).map_err(|err| {
            AppError::InvalidInput(format!("invalid config at {}: {err}", path.display()))
        })
    }

    fn validate(&self, path: &Path) -> Result<(), AppError> {
        if let Some(schema_version) = self.schema_version {
            if schema_version == 0 {
                return Err(AppError::InvalidInput(format!(
                    "invalid config at {}: schema_version must be >= 1",
                    path.display()
                )));
            }
        }

        if self.project.bin.iter().any(|bin| bin.trim().is_empty()) {
            return Err(AppError::InvalidInput(format!(
                "invalid config at {}: project.bin entries cannot be empty",
                path.display()
            )));
        }

        if let Some(strategy) = &self.artifact.strategy {
            let normalized = strategy.trim();
            if !matches!(normalized, "release-asset" | "source-tarball") {
                return Err(AppError::InvalidInput(format!(
                    "invalid config at {}: artifact.strategy must be 'release-asset' or 'source-tarball'",
                    path.display()
                )));
            }
        }

        if let Some(mode) = &self.version_update.mode {
            let normalized = mode.trim();
            if !matches!(normalized, "none" | "cargo" | "regex") {
                return Err(AppError::InvalidInput(format!(
                    "invalid config at {}: version_update.mode must be 'none', 'cargo', or 'regex'",
                    path.display()
                )));
            }
        }

        if let Some(provider) = &self.host.provider {
            let normalized = provider.trim();
            if !matches!(normalized, "github") {
                return Err(AppError::InvalidInput(format!(
                    "invalid config at {}: host.provider must be 'github'",
                    path.display()
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_config() {
        let raw = r#"schema_version = 1

[project]
name = "brew"
bin = ["brew"]
"#;

        let config = Config::from_str(raw, Path::new("config.toml")).unwrap();
        assert_eq!(config.schema_version, Some(1));
        assert_eq!(config.project.name.as_deref(), Some("brew"));
        assert_eq!(config.project.bin, vec!["brew".to_string()]);
    }

    #[test]
    fn rejects_unknown_artifact_strategy() {
        let raw = r#"schema_version = 1

[artifact]
strategy = "mystery"
"#;

        let err = Config::from_str(raw, Path::new("config.toml"))
            .and_then(|config| config.validate(Path::new("config.toml")))
            .unwrap_err();

        assert!(matches!(err, AppError::InvalidInput(_)));
    }
}
