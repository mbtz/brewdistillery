use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const SCHEMA_VERSION_V1: u32 = 1;

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

    #[serde(default, flatten)]
    pub extra: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepoConfig {
    pub owner: Option<String>,
    pub repo: Option<String>,
    pub remote: Option<String>,
    pub path: Option<PathBuf>,

    #[serde(default, flatten)]
    pub extra: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TapConfig {
    pub owner: Option<String>,
    pub repo: Option<String>,
    pub remote: Option<String>,
    pub path: Option<PathBuf>,
    pub formula: Option<String>,
    pub formula_path: Option<PathBuf>,

    #[serde(default, flatten)]
    pub extra: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArtifactConfig {
    pub owner: Option<String>,
    pub repo: Option<String>,
    pub strategy: Option<String>,
    pub asset_template: Option<String>,
    pub asset_name: Option<String>,
    pub checksum_max_bytes: Option<u64>,
    pub checksum_timeout_secs: Option<u64>,
    pub checksum_max_retries: Option<u32>,
    pub checksum_retry_base_delay_ms: Option<u64>,
    pub checksum_retry_max_delay_ms: Option<u64>,

    #[serde(default)]
    pub targets: BTreeMap<String, ArtifactTarget>,

    #[serde(default, flatten)]
    pub extra: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArtifactTarget {
    pub asset_template: Option<String>,
    pub asset_name: Option<String>,

    #[serde(default, flatten)]
    pub extra: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReleaseConfig {
    pub tag_format: Option<String>,
    pub tarball_url_template: Option<String>,
    pub commit_message_template: Option<String>,

    #[serde(default, flatten)]
    pub extra: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VersionUpdateConfig {
    pub mode: Option<String>,
    pub cargo_package: Option<String>,
    pub regex_file: Option<PathBuf>,
    pub regex_pattern: Option<String>,
    pub regex_replacement: Option<String>,

    #[serde(default, flatten)]
    pub extra: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostConfig {
    pub provider: Option<String>,
    pub api_base: Option<String>,

    #[serde(default, flatten)]
    pub extra: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub path: Option<PathBuf>,
    pub install_block: Option<String>,

    #[serde(default, flatten)]
    pub extra: BTreeMap<String, toml::Value>,
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

        let mut to_save = self.clone();
        if to_save.schema_version.is_none() {
            to_save.schema_version = Some(SCHEMA_VERSION_V1);
        }

        to_save.validate(path)?;
        let content = toml::to_string_pretty(&to_save).map_err(|err| {
            AppError::InvalidInput(format!(
                "failed to serialize config {}: {err}",
                path.display()
            ))
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

        if matches!(self.artifact.checksum_max_bytes, Some(0)) {
            return Err(AppError::InvalidInput(format!(
                "invalid config at {}: artifact.checksum_max_bytes must be > 0",
                path.display()
            )));
        }

        if matches!(self.artifact.checksum_timeout_secs, Some(0)) {
            return Err(AppError::InvalidInput(format!(
                "invalid config at {}: artifact.checksum_timeout_secs must be > 0",
                path.display()
            )));
        }

        if matches!(self.artifact.checksum_max_retries, Some(0)) {
            return Err(AppError::InvalidInput(format!(
                "invalid config at {}: artifact.checksum_max_retries must be > 0",
                path.display()
            )));
        }

        if matches!(self.artifact.checksum_retry_base_delay_ms, Some(0)) {
            return Err(AppError::InvalidInput(format!(
                "invalid config at {}: artifact.checksum_retry_base_delay_ms must be > 0",
                path.display()
            )));
        }

        if matches!(self.artifact.checksum_retry_max_delay_ms, Some(0)) {
            return Err(AppError::InvalidInput(format!(
                "invalid config at {}: artifact.checksum_retry_max_delay_ms must be > 0",
                path.display()
            )));
        }

        if let (Some(base), Some(max)) = (
            self.artifact.checksum_retry_base_delay_ms,
            self.artifact.checksum_retry_max_delay_ms,
        ) {
            if max < base {
                return Err(AppError::InvalidInput(format!(
                    "invalid config at {}: artifact.checksum_retry_max_delay_ms must be >= artifact.checksum_retry_base_delay_ms",
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
    use std::fs;
    use tempfile::tempdir;
    use toml::Value as TomlValue;

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

    #[test]
    fn rejects_zero_checksum_limits() {
        let raw = r#"schema_version = 1

[artifact]
strategy = "release-asset"
checksum_max_bytes = 0
"#;

        let err = Config::from_str(raw, Path::new("config.toml"))
            .and_then(|config| config.validate(Path::new("config.toml")))
            .unwrap_err();

        assert!(matches!(err, AppError::InvalidInput(_)));
        assert!(err.to_string().contains("checksum_max_bytes must be > 0"));
    }

    #[test]
    fn rejects_retry_cap_less_than_base_delay() {
        let raw = r#"schema_version = 1

[artifact]
strategy = "release-asset"
checksum_retry_base_delay_ms = 500
checksum_retry_max_delay_ms = 100
"#;

        let err = Config::from_str(raw, Path::new("config.toml"))
            .and_then(|config| config.validate(Path::new("config.toml")))
            .unwrap_err();

        assert!(matches!(err, AppError::InvalidInput(_)));
        assert!(err.to_string().contains("checksum_retry_max_delay_ms must be >="));
    }

    #[test]
    fn saves_with_schema_version_when_missing() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let raw = r#"[project]
name = "brew"
bin = ["brew"]
"#;

        let config = Config::from_str(raw, &path).unwrap();
        assert_eq!(config.schema_version, None);

        config.save(&path).unwrap();

        let saved = fs::read_to_string(&path).unwrap();
        let value: TomlValue = toml::from_str(&saved).unwrap();
        assert_eq!(
            value
                .get("schema_version")
                .and_then(|entry| entry.as_integer()),
            Some(SCHEMA_VERSION_V1 as i64)
        );
    }

    #[test]
    fn preserves_unknown_fields_across_sections() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let raw = r#"mystery = "keep"

[project]
name = "brew"
bin = ["brew"]
custom_project = "value"

[artifact]
strategy = "release-asset"
custom_artifact = true

[artifact.targets."darwin-arm64"]
asset_template = "brew-{version}-darwin-arm64.tar.gz"
custom_target = "target-extra"
"#;

        let config = Config::from_str(raw, &path).unwrap();
        config.save(&path).unwrap();

        let saved = fs::read_to_string(&path).unwrap();
        let value: TomlValue = toml::from_str(&saved).unwrap();

        assert_eq!(
            value.get("mystery").and_then(|entry| entry.as_str()),
            Some("keep")
        );
        assert_eq!(
            value
                .get("project")
                .and_then(|entry| entry.get("custom_project"))
                .and_then(|entry| entry.as_str()),
            Some("value")
        );
        assert_eq!(
            value
                .get("artifact")
                .and_then(|entry| entry.get("custom_artifact"))
                .and_then(|entry| entry.as_bool()),
            Some(true)
        );
        assert_eq!(
            value
                .get("artifact")
                .and_then(|entry| entry.get("targets"))
                .and_then(|entry| entry.get("darwin-arm64"))
                .and_then(|entry| entry.get("custom_target"))
                .and_then(|entry| entry.as_str()),
            Some("target-extra")
        );
    }
}
