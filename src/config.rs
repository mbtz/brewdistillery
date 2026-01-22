use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub schema_version: Option<u32>,

    #[serde(flatten)]
    pub extra: BTreeMap<String, toml::Value>,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, AppError> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)?;
        toml::from_str(&content).map_err(|err| {
            AppError::InvalidInput(format!("invalid config at {}: {err}", path.display()))
        })
    }
}
