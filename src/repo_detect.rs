use crate::errors::AppError;
use serde_json::Value as JsonValue;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value as TomlValue;

#[derive(Debug, Clone, Default)]
pub struct RepoInfo {
    pub git_root: Option<PathBuf>,
    pub metadata: Option<ProjectMetadata>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub version: Option<String>,
    pub bin: Vec<String>,
    pub source: MetadataSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataSource {
    Cargo,
    PackageJson,
    PyProject,
    GoMod,
    Unknown,
}

impl Default for MetadataSource {
    fn default() -> Self {
        MetadataSource::Unknown
    }
}

pub fn detect_repo(start: &Path) -> Result<RepoInfo, AppError> {
    let git_root = find_git_root(start);
    let root = git_root.as_deref().unwrap_or(start);
    let metadata = detect_metadata(root)?;

    Ok(RepoInfo {
        git_root,
        metadata,
    })
}

fn find_git_root(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(path) = current {
        if path.join(".git").exists() {
            return Some(path.to_path_buf());
        }

        current = path.parent();
    }
    None
}

fn detect_metadata(root: &Path) -> Result<Option<ProjectMetadata>, AppError> {
    let mut metadata = if let Some(meta) = detect_cargo_metadata(root)? {
        Some(meta)
    } else if let Some(meta) = detect_package_json_metadata(root)? {
        Some(meta)
    } else if let Some(meta) = detect_pyproject_metadata(root)? {
        Some(meta)
    } else {
        detect_go_mod_metadata(root)?
    };

    if let Some(meta) = metadata.as_mut() {
        if meta.license.is_none() {
            meta.license = detect_license_from_files(root);
        }
    }

    Ok(metadata)
}

fn detect_cargo_metadata(root: &Path) -> Result<Option<ProjectMetadata>, AppError> {
    let path = root.join("Cargo.toml");
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path)?;
    let value: TomlValue = toml::from_str(&raw).map_err(|err| {
        AppError::InvalidInput(format!("invalid Cargo.toml at {}: {err}", path.display()))
    })?;

    let package = value
        .get("package")
        .and_then(|entry| entry.as_table())
        .or_else(|| {
            value
                .get("workspace")
                .and_then(|entry| entry.as_table())
                .and_then(|table| table.get("package"))
                .and_then(|entry| entry.as_table())
        });

    let package = match package {
        Some(package) => package,
        None => return Ok(None),
    };

    let name = table_get_string(package, "name");
    let mut bins = Vec::new();
    if let Some(bin_entries) = value.get("bin").and_then(|entry| entry.as_array()) {
        for entry in bin_entries {
            if let Some(table) = entry.as_table() {
                if let Some(bin_name) = table_get_string(table, "name") {
                    bins.push(bin_name);
                }
            }
        }
    }

    if bins.is_empty() {
        if let Some(name) = name.clone() {
            bins.push(name);
        }
    }
    normalize_bins(&mut bins);

    Ok(Some(ProjectMetadata {
        name,
        description: table_get_string(package, "description"),
        homepage: table_get_string(package, "homepage"),
        license: table_get_string(package, "license"),
        version: table_get_string(package, "version"),
        bin: bins,
        source: MetadataSource::Cargo,
    }))
}

fn detect_package_json_metadata(root: &Path) -> Result<Option<ProjectMetadata>, AppError> {
    let path = root.join("package.json");
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path)?;
    let value: JsonValue = serde_json::from_str(&raw).map_err(|err| {
        AppError::InvalidInput(format!("invalid package.json at {}: {err}", path.display()))
    })?;

    let name = value
        .get("name")
        .and_then(|entry| entry.as_str())
        .map(|entry| entry.to_string());

    let mut bins = Vec::new();
    match value.get("bin") {
        Some(JsonValue::String(_)) => {
            if let Some(name) = name.as_deref() {
                bins.push(unscoped_package_name(name));
            }
        }
        Some(JsonValue::Object(map)) => {
            for key in map.keys() {
                if !key.trim().is_empty() {
                    bins.push(key.to_string());
                }
            }
        }
        _ => {}
    }

    normalize_bins(&mut bins);

    Ok(Some(ProjectMetadata {
        name,
        description: value
            .get("description")
            .and_then(|entry| entry.as_str())
            .map(|entry| entry.to_string()),
        homepage: value
            .get("homepage")
            .and_then(|entry| entry.as_str())
            .map(|entry| entry.to_string()),
        license: value
            .get("license")
            .and_then(|entry| entry.as_str())
            .map(|entry| entry.to_string()),
        version: value
            .get("version")
            .and_then(|entry| entry.as_str())
            .map(|entry| entry.to_string()),
        bin: bins,
        source: MetadataSource::PackageJson,
    }))
}

fn detect_pyproject_metadata(root: &Path) -> Result<Option<ProjectMetadata>, AppError> {
    let path = root.join("pyproject.toml");
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path)?;
    let value: TomlValue = toml::from_str(&raw).map_err(|err| {
        AppError::InvalidInput(format!("invalid pyproject.toml at {}: {err}", path.display()))
    })?;

    if let Some(project) = value.get("project").and_then(|entry| entry.as_table()) {
        let mut bins = Vec::new();
        if let Some(scripts) = project.get("scripts").and_then(|entry| entry.as_table()) {
            for key in scripts.keys() {
                if !key.trim().is_empty() {
                    bins.push(key.to_string());
                }
            }
        }

        let homepage = project
            .get("urls")
            .and_then(|entry| entry.as_table())
            .and_then(|table| {
                table_get_string(table, "Homepage")
                    .or_else(|| table_get_string(table, "homepage"))
                    .or_else(|| table_get_string(table, "Home"))
                    .or_else(|| table_get_string(table, "home"))
                    .or_else(|| table_get_string(table, "Repository"))
            });

        let license = match project.get("license") {
            Some(TomlValue::String(value)) => Some(value.to_string()),
            Some(TomlValue::Table(table)) => table_get_string(table, "text")
                .or_else(|| table_get_string(table, "file")),
            _ => None,
        };

        normalize_bins(&mut bins);

        return Ok(Some(ProjectMetadata {
            name: table_get_string(project, "name"),
            description: table_get_string(project, "description"),
            homepage,
            license,
            version: table_get_string(project, "version"),
            bin: bins,
            source: MetadataSource::PyProject,
        }));
    }

    if let Some(poetry) = value
        .get("tool")
        .and_then(|entry| entry.as_table())
        .and_then(|table| table.get("poetry"))
        .and_then(|entry| entry.as_table())
    {
        let mut bins = Vec::new();
        if let Some(scripts) = poetry.get("scripts").and_then(|entry| entry.as_table()) {
            for key in scripts.keys() {
                if !key.trim().is_empty() {
                    bins.push(key.to_string());
                }
            }
        }

        normalize_bins(&mut bins);

        return Ok(Some(ProjectMetadata {
            name: table_get_string(poetry, "name"),
            description: table_get_string(poetry, "description"),
            homepage: table_get_string(poetry, "homepage"),
            license: table_get_string(poetry, "license"),
            version: table_get_string(poetry, "version"),
            bin: bins,
            source: MetadataSource::PyProject,
        }));
    }

    Ok(None)
}

fn detect_go_mod_metadata(root: &Path) -> Result<Option<ProjectMetadata>, AppError> {
    let path = root.join("go.mod");
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path)?;
    let mut module_path = None;
    for line in raw.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("module ") {
            module_path = Some(rest.trim().to_string());
            break;
        }
    }

    let module_path = match module_path {
        Some(module_path) if !module_path.is_empty() => module_path,
        _ => return Ok(None),
    };

    let name = module_path
        .split('/')
        .last()
        .map(|segment| segment.to_string());

    let mut bins: Vec<String> = name.clone().into_iter().collect();
    normalize_bins(&mut bins);

    Ok(Some(ProjectMetadata {
        name,
        description: None,
        homepage: None,
        license: None,
        version: None,
        bin: bins,
        source: MetadataSource::GoMod,
    }))
}

fn table_get_string(table: &toml::value::Table, key: &str) -> Option<String> {
    table.get(key).and_then(|value| value.as_str()).map(|v| v.to_string())
}

fn unscoped_package_name(name: &str) -> String {
    name.split('/').last().unwrap_or(name).trim().to_string()
}

fn normalize_bins(bins: &mut Vec<String>) {
    bins.retain(|entry| !entry.trim().is_empty());
    bins.sort();
    bins.dedup();
}

fn detect_license_from_files(root: &Path) -> Option<String> {
    let candidates = [
        "LICENSE",
        "LICENSE.txt",
        "LICENSE.md",
        "COPYING",
        "COPYING.txt",
        "COPYING.md",
        "LICENSE-MIT",
        "LICENSE-APACHE",
        "LICENSE-APACHE-2.0",
        "LICENSE-BSD",
        "LICENSE-BSD-3-CLAUSE",
        "LICENSE-GPL",
        "LICENSE-GPL-3.0",
    ];

    let mut detected = None;
    for candidate in candidates {
        let path = root.join(candidate);
        if !path.exists() {
            continue;
        }

        let normalized = candidate.to_ascii_uppercase();
        let license = if normalized.contains("MIT") {
            Some("MIT")
        } else if normalized.contains("APACHE") {
            Some("Apache-2.0")
        } else if normalized.contains("BSD-3") || normalized.contains("BSD3") {
            Some("BSD-3-Clause")
        } else if normalized.contains("BSD") {
            Some("BSD-2-Clause")
        } else if normalized.contains("GPL-3") {
            Some("GPL-3.0-only")
        } else if normalized.contains("GPL") {
            Some("GPL-2.0-only")
        } else {
            None
        };

        if let Some(license) = license {
            if let Some(existing) = detected.as_deref() {
                if existing != license {
                    return None;
                }
            } else {
                detected = Some(license.to_string());
            }
        }
    }

    detected
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn detects_cargo_metadata() {
        let dir = tempdir().unwrap();
        let cargo = r#"[package]
name = "brewtool"
description = "Brew tool"
homepage = "https://example.com"
license = "MIT"
version = "1.2.3"

[[bin]]
name = "brewtool"

[[bin]]
name = "brewctl"
"#;

        fs::write(dir.path().join("Cargo.toml"), cargo).unwrap();

        let meta = detect_metadata(dir.path()).unwrap().unwrap();
        assert_eq!(meta.name.as_deref(), Some("brewtool"));
        assert_eq!(meta.description.as_deref(), Some("Brew tool"));
        assert_eq!(meta.homepage.as_deref(), Some("https://example.com"));
        assert_eq!(meta.license.as_deref(), Some("MIT"));
        assert_eq!(meta.version.as_deref(), Some("1.2.3"));
        assert_eq!(meta.bin, vec!["brewctl".to_string(), "brewtool".to_string()]);
        assert_eq!(meta.source, MetadataSource::Cargo);
    }

    #[test]
    fn detects_package_json_bins() {
        let dir = tempdir().unwrap();
        let package_json = r#"{
  "name": "@acme/brewtool",
  "version": "0.9.0",
  "bin": {
    "brewtool": "bin/brewtool.js",
    "brewctl": "bin/brewctl.js"
  }
}"#;

        fs::write(dir.path().join("package.json"), package_json).unwrap();

        let meta = detect_metadata(dir.path()).unwrap().unwrap();
        assert_eq!(meta.name.as_deref(), Some("@acme/brewtool"));
        assert_eq!(meta.bin, vec!["brewctl".to_string(), "brewtool".to_string()]);
        assert_eq!(meta.source, MetadataSource::PackageJson);
    }

    #[test]
    fn detects_pyproject_metadata() {
        let dir = tempdir().unwrap();
        let pyproject = r#"[project]
name = "brewpy"
description = "Brew Python"
version = "0.4.0"
license = { text = "MIT" }

[project.urls]
Homepage = "https://example.com"

[project.scripts]
brewpy = "brewpy:main"
"#;

        fs::write(dir.path().join("pyproject.toml"), pyproject).unwrap();

        let meta = detect_metadata(dir.path()).unwrap().unwrap();
        assert_eq!(meta.name.as_deref(), Some("brewpy"));
        assert_eq!(meta.homepage.as_deref(), Some("https://example.com"));
        assert_eq!(meta.license.as_deref(), Some("MIT"));
        assert_eq!(meta.bin, vec!["brewpy".to_string()]);
        assert_eq!(meta.source, MetadataSource::PyProject);
    }

    #[test]
    fn detects_go_mod_metadata() {
        let dir = tempdir().unwrap();
        let go_mod = "module github.com/acme/brewgo\n";
        fs::write(dir.path().join("go.mod"), go_mod).unwrap();

        let meta = detect_metadata(dir.path()).unwrap().unwrap();
        assert_eq!(meta.name.as_deref(), Some("brewgo"));
        assert_eq!(meta.bin, vec!["brewgo".to_string()]);
        assert_eq!(meta.source, MetadataSource::GoMod);
    }

    #[test]
    fn infers_license_from_filename() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"brew\"\n").unwrap();
        fs::write(dir.path().join("LICENSE-MIT"), "MIT License").unwrap();

        let meta = detect_metadata(dir.path()).unwrap().unwrap();
        assert_eq!(meta.license.as_deref(), Some("MIT"));
    }
}
