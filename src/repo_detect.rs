use crate::errors::AppError;
use serde_json::Value as JsonValue;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value as TomlValue;

#[derive(Debug, Clone, Default)]
pub struct RepoInfo {
    pub git_root: Option<PathBuf>,
    pub metadata: Option<ProjectMetadata>,
    pub conflicts: Vec<MetadataConflict>,
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
pub enum ConflictPolicy {
    Error,
    Allow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataConflict {
    pub field: String,
    pub details: String,
}

#[derive(Debug, Clone, Default)]
struct ResolvedMetadata {
    metadata: Option<ProjectMetadata>,
    conflicts: Vec<MetadataConflict>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataSource {
    Cargo,
    PackageJson,
    PyProject,
    GoMod,
    GitRemote,
    Mixed,
    Unknown,
}

impl Default for MetadataSource {
    fn default() -> Self {
        MetadataSource::Unknown
    }
}

impl MetadataSource {
    fn label(self) -> &'static str {
        match self {
            MetadataSource::Cargo => "Cargo.toml",
            MetadataSource::PackageJson => "package.json",
            MetadataSource::PyProject => "pyproject.toml",
            MetadataSource::GoMod => "go.mod",
            MetadataSource::GitRemote => "git remote",
            MetadataSource::Mixed => "multiple sources",
            MetadataSource::Unknown => "unknown",
        }
    }
}

pub fn detect_repo(start: &Path, policy: ConflictPolicy) -> Result<RepoInfo, AppError> {
    let git_root = find_git_root(start);
    let root = git_root.as_deref().unwrap_or(start);
    let resolved = detect_metadata(root, policy)?;

    Ok(RepoInfo {
        git_root,
        metadata: resolved.metadata,
        conflicts: resolved.conflicts,
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

fn detect_metadata(root: &Path, policy: ConflictPolicy) -> Result<ResolvedMetadata, AppError> {
    let mut metas = Vec::new();
    if let Some(meta) = detect_cargo_metadata(root)? {
        metas.push(meta);
    }
    if let Some(meta) = detect_package_json_metadata(root)? {
        metas.push(meta);
    }
    if let Some(meta) = detect_pyproject_metadata(root)? {
        metas.push(meta);
    }
    if let Some(meta) = detect_go_mod_metadata(root)? {
        metas.push(meta);
    }

    if metas.is_empty() {
        if let Some(meta) = metadata_from_git_remote(root)? {
            metas.push(meta);
        }
    }

    if metas.is_empty() {
        return Ok(ResolvedMetadata::default());
    }

    let mut resolved = resolve_metadata(&metas);

    if !resolved.conflicts.is_empty() && policy == ConflictPolicy::Error {
        return Err(conflict_error(&resolved.conflicts));
    }

    let conflict_fields = resolved
        .conflicts
        .iter()
        .map(|conflict| conflict.field.as_str())
        .collect::<HashSet<_>>();

    if let Some(meta) = resolved.metadata.as_mut() {
        if meta.license.is_none() && !conflict_fields.contains("license") {
            meta.license = detect_license_from_files(root);
        }
        if meta.homepage.is_none() && !conflict_fields.contains("homepage") {
            meta.homepage = detect_homepage_from_git(root)?;
        }
        if meta.name.is_none() && !conflict_fields.contains("name") {
            meta.name = detect_name_from_git(root)?;
        }
    }

    Ok(resolved)
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
    let project_name = name.as_deref().map(unscoped_package_name);

    let mut bins = Vec::new();
    match value.get("bin") {
        Some(JsonValue::String(_)) => {
            if let Some(name) = project_name.as_deref() {
                bins.push(name.to_string());
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
        name: project_name,
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

fn resolve_metadata(metas: &[ProjectMetadata]) -> ResolvedMetadata {
    let source = if metas.len() == 1 {
        metas[0].source
    } else {
        MetadataSource::Mixed
    };

    let mut conflicts = Vec::new();

    let (name, conflict) = resolve_string_field("name", metas, |meta| meta.name.as_deref());
    push_conflict(&mut conflicts, conflict);

    let (description, conflict) =
        resolve_string_field("description", metas, |meta| meta.description.as_deref());
    push_conflict(&mut conflicts, conflict);

    let (homepage, conflict) =
        resolve_string_field("homepage", metas, |meta| meta.homepage.as_deref());
    push_conflict(&mut conflicts, conflict);

    let (license, conflict) =
        resolve_string_field("license", metas, |meta| meta.license.as_deref());
    push_conflict(&mut conflicts, conflict);

    let (version, conflict) =
        resolve_string_field("version", metas, |meta| meta.version.as_deref());
    push_conflict(&mut conflicts, conflict);

    let (bin, conflict) = resolve_bins_field(metas);
    push_conflict(&mut conflicts, conflict);

    ResolvedMetadata {
        metadata: Some(ProjectMetadata {
            name,
            description,
            homepage,
            license,
            version,
            bin,
            source,
        }),
        conflicts,
    }
}

fn resolve_string_field<F>(
    field: &str,
    metas: &[ProjectMetadata],
    getter: F,
) -> (Option<String>, Option<MetadataConflict>)
where
    F: Fn(&ProjectMetadata) -> Option<&str>,
{
    let mut values: Vec<(MetadataSource, String)> = Vec::new();
    for meta in metas {
        if let Some(value) = getter(meta) {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                continue;
            }
            if !values.iter().any(|(_, existing)| existing == trimmed) {
                values.push((meta.source, trimmed.to_string()));
            }
        }
    }

    if values.len() > 1 {
        return (
            None,
            Some(MetadataConflict {
                field: field.to_string(),
                details: format_conflicts(&values),
            }),
        );
    }

    (values.first().map(|(_, value)| value.clone()), None)
}

fn resolve_bins_field(metas: &[ProjectMetadata]) -> (Vec<String>, Option<MetadataConflict>) {
    let mut values: Vec<(MetadataSource, Vec<String>)> = Vec::new();
    for meta in metas {
        if meta.bin.is_empty() {
            continue;
        }
        let mut bins = meta.bin.clone();
        normalize_bins(&mut bins);
        if !values.iter().any(|(_, existing)| *existing == bins) {
            values.push((meta.source, bins));
        }
    }

    if values.len() > 1 {
        return (
            Vec::new(),
            Some(MetadataConflict {
                field: "bin".to_string(),
                details: format_bins_conflicts(&values),
            }),
        );
    }

    (
        values
            .first()
            .map(|(_, bins)| bins.clone())
            .unwrap_or_default(),
        None,
    )
}

fn push_conflict(conflicts: &mut Vec<MetadataConflict>, conflict: Option<MetadataConflict>) {
    if let Some(conflict) = conflict {
        conflicts.push(conflict);
    }
}

fn conflict_error(conflicts: &[MetadataConflict]) -> AppError {
    let details = conflicts
        .iter()
        .map(|conflict| format!("{}: {}", conflict.field, conflict.details))
        .collect::<Vec<_>>()
        .join("; ");
    AppError::InvalidInput(format!(
        "conflicting metadata detected: {details}; run without --non-interactive or set explicit values"
    ))
}

fn format_conflicts(values: &[(MetadataSource, String)]) -> String {
    values
        .iter()
        .map(|(source, value)| format!("{}='{}'", source.label(), value))
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_bins_conflicts(values: &[(MetadataSource, Vec<String>)]) -> String {
    values
        .iter()
        .map(|(source, bins)| format!("{}=[{}]", source.label(), bins.join(", ")))
        .collect::<Vec<_>>()
        .join(", ")
}

fn metadata_from_git_remote(root: &Path) -> Result<Option<ProjectMetadata>, AppError> {
    let remote = match select_github_remote_url(root, true)? {
        Some(remote) => remote,
        None => return Ok(None),
    };
    let name = repo_name_from_remote(&remote);
    let homepage = github_https_from_remote(&remote);
    if name.is_none() && homepage.is_none() {
        return Ok(None);
    }

    Ok(Some(ProjectMetadata {
        name,
        description: None,
        homepage,
        license: None,
        version: None,
        bin: Vec::new(),
        source: MetadataSource::GitRemote,
    }))
}

fn detect_homepage_from_git(root: &Path) -> Result<Option<String>, AppError> {
    let remote = select_github_remote_url(root, false)?;
    Ok(remote.and_then(|remote| github_https_from_remote(&remote)))
}

fn detect_name_from_git(root: &Path) -> Result<Option<String>, AppError> {
    let remote = select_github_remote_url(root, false)?;
    Ok(remote.and_then(|remote| repo_name_from_remote(&remote)))
}

#[derive(Debug, Clone)]
struct GitRemote {
    name: String,
    url: String,
}

fn select_github_remote_url(root: &Path, require: bool) -> Result<Option<String>, AppError> {
    let remotes = load_git_remotes(root)?;
    if remotes.is_empty() {
        return if require {
            Err(AppError::GitState(
                "no GitHub remote found; specify --host-owner/--host-repo".to_string(),
            ))
        } else {
            Ok(None)
        };
    }

    if let Some(origin) = remotes.iter().find(|remote| remote.name == "origin") {
        if is_github_url(&origin.url) {
            return github_https_from_remote(&origin.url)
                .map(|_| origin.url.clone())
                .ok_or_else(|| {
                    AppError::GitState(
                        "unable to parse GitHub remote URL; specify --host-owner/--host-repo"
                            .to_string(),
                    )
                })
                .map(Some);
        }
    }

    let mut github_remotes = Vec::new();
    let mut has_unparsable = false;
    for remote in &remotes {
        if is_github_url(&remote.url) {
            if github_https_from_remote(&remote.url).is_some() {
                github_remotes.push(remote);
            } else {
                has_unparsable = true;
            }
        }
    }

    if github_remotes.len() == 1 {
        return Ok(Some(github_remotes[0].url.clone()));
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

    if require {
        Err(AppError::GitState(
            "no GitHub remote found; specify --host-owner/--host-repo".to_string(),
        ))
    } else {
        Ok(None)
    }
}

fn load_git_remotes(root: &Path) -> Result<Vec<GitRemote>, AppError> {
    let config_path = match git_config_path(root) {
        Some(path) => path,
        None => return Ok(Vec::new()),
    };
    let content = fs::read_to_string(config_path)?;
    let mut remotes: Vec<GitRemote> = Vec::new();
    let mut current_remote: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("[remote \"") {
            if let Some(remote) = rest.strip_suffix("\"]") {
                current_remote = Some(remote.to_string());
                continue;
            }
        }

        if trimmed.starts_with('[') {
            current_remote = None;
            continue;
        }

        let Some(remote_name) = current_remote.as_deref() else {
            continue;
        };

        if let Some(url) = parse_url_line(trimmed) {
            if remotes.iter().any(|remote| remote.name == remote_name) {
                continue;
            }
            remotes.push(GitRemote {
                name: remote_name.to_string(),
                url: url.to_string(),
            });
        }
    }

    Ok(remotes)
}

fn parse_url_line(line: &str) -> Option<&str> {
    if let Some(rest) = line.strip_prefix("url") {
        let rest = rest.trim_start();
        if let Some(rest) = rest.strip_prefix('=') {
            let url = rest.trim();
            if url.is_empty() {
                None
            } else {
                Some(url)
            }
        } else {
            None
        }
    } else {
        None
    }
}

fn git_config_path(root: &Path) -> Option<PathBuf> {
    let git_path = root.join(".git");
    if git_path.is_dir() {
        return Some(git_path.join("config"));
    }

    if git_path.is_file() {
        let content = fs::read_to_string(&git_path).ok()?;
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("gitdir:") {
                let path = rest.trim();
                if path.is_empty() {
                    continue;
                }
                let gitdir = PathBuf::from(path);
                let resolved = if gitdir.is_absolute() {
                    gitdir
                } else {
                    root.join(gitdir)
                };
                return Some(resolved.join("config"));
            }
        }
    }

    None
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

fn repo_name_from_remote(remote: &str) -> Option<String> {
    let https = github_https_from_remote(remote)?;
    let name = https.rsplit('/').next()?;
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
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
    use std::collections::HashSet;
    use std::path::Path;
    use tempfile::tempdir;

    fn detect_metadata_strict(root: &Path) -> ProjectMetadata {
        detect_metadata(root, ConflictPolicy::Error)
            .unwrap()
            .metadata
            .expect("metadata")
    }

    fn detect_metadata_allow(root: &Path) -> ResolvedMetadata {
        detect_metadata(root, ConflictPolicy::Allow).unwrap()
    }

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

        let meta = detect_metadata_strict(dir.path());
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

        let meta = detect_metadata_strict(dir.path());
        assert_eq!(meta.name.as_deref(), Some("brewtool"));
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

        let meta = detect_metadata_strict(dir.path());
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

        let meta = detect_metadata_strict(dir.path());
        assert_eq!(meta.name.as_deref(), Some("brewgo"));
        assert_eq!(meta.bin, vec!["brewgo".to_string()]);
        assert_eq!(meta.source, MetadataSource::GoMod);
    }

    #[test]
    fn infers_license_from_filename() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"brew\"\n").unwrap();
        fs::write(dir.path().join("LICENSE-MIT"), "MIT License").unwrap();

        let meta = detect_metadata_strict(dir.path());
        assert_eq!(meta.license.as_deref(), Some("MIT"));
    }

    #[test]
    fn uses_git_remote_for_homepage_fallback() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".git")).unwrap();
        fs::write(
            dir.path().join(".git/config"),
            r#"[remote "origin"]
    url = git@github.com:acme/brewtool.git
"#,
        )
        .unwrap();

        fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"brew\"\n").unwrap();

        let meta = detect_metadata_strict(dir.path());
        assert_eq!(
            meta.homepage.as_deref(),
            Some("https://github.com/acme/brewtool")
        );
    }

    #[test]
    fn falls_back_to_git_remote_when_no_manifest() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".git")).unwrap();
        fs::write(
            dir.path().join(".git/config"),
            r#"[remote "origin"]
    url = https://github.com/acme/brewtool.git
"#,
        )
        .unwrap();

        let meta = detect_metadata_strict(dir.path());
        assert_eq!(meta.name.as_deref(), Some("brewtool"));
        assert_eq!(
            meta.homepage.as_deref(),
            Some("https://github.com/acme/brewtool")
        );
        assert_eq!(meta.source, MetadataSource::GitRemote);
    }

    #[test]
    fn errors_on_conflicting_metadata() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"brewtool\"\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{ "name": "other", "bin": { "other": "bin/other" } }"#,
        )
        .unwrap();

        let err = detect_metadata(dir.path(), ConflictPolicy::Error).unwrap_err();
        assert!(matches!(err, AppError::InvalidInput(_)));
        assert!(err
            .to_string()
            .starts_with("conflicting metadata detected:"));
    }

    #[test]
    fn allows_conflicts_when_policy_allows() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"brewtool\"\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{ "name": "other", "bin": { "other": "bin/other" } }"#,
        )
        .unwrap();

        let resolved = detect_metadata_allow(dir.path());
        let meta = resolved.metadata.expect("metadata");

        assert!(meta.name.is_none());
        assert!(meta.bin.is_empty());

        let fields = resolved
            .conflicts
            .iter()
            .map(|conflict| conflict.field.as_str())
            .collect::<HashSet<_>>();
        assert!(fields.contains("name"));
        assert!(fields.contains("bin"));
    }

    #[test]
    fn prefers_origin_github_remote() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".git")).unwrap();
        fs::write(
            dir.path().join(".git/config"),
            r#"[remote "origin"]
    url = git@github.com:acme/brewtool.git
[remote "upstream"]
    url = https://github.com/acme/other.git
"#,
        )
        .unwrap();

        let remote = select_github_remote_url(dir.path(), true).unwrap();
        assert_eq!(
            remote.as_deref(),
            Some("git@github.com:acme/brewtool.git")
        );
    }

    #[test]
    fn errors_on_multiple_github_remotes_without_origin() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".git")).unwrap();
        fs::write(
            dir.path().join(".git/config"),
            r#"[remote "origin"]
    url = https://gitlab.com/acme/brewtool.git
[remote "upstream"]
    url = https://github.com/acme/brewtool.git
[remote "fork"]
    url = git@github.com:acme/brewtool-fork.git
"#,
        )
        .unwrap();

        let err = select_github_remote_url(dir.path(), true).unwrap_err();
        assert!(matches!(err, AppError::GitState(_)));
        assert_eq!(
            err.to_string(),
            "multiple git remotes found; specify --host-owner/--host-repo"
        );
    }

    #[test]
    fn errors_on_unparsable_github_remote() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".git")).unwrap();
        fs::write(
            dir.path().join(".git/config"),
            r#"[remote "origin"]
    url = ssh://github.com/acme/brewtool.git
"#,
        )
        .unwrap();

        let err = select_github_remote_url(dir.path(), true).unwrap_err();
        assert!(matches!(err, AppError::GitState(_)));
        assert_eq!(
            err.to_string(),
            "unable to parse GitHub remote URL; specify --host-owner/--host-repo"
        );
    }

    #[test]
    fn returns_none_when_no_github_remote_and_optional() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".git")).unwrap();
        fs::write(
            dir.path().join(".git/config"),
            r#"[remote "origin"]
    url = https://gitlab.com/acme/brewtool.git
"#,
        )
        .unwrap();

        let remote = select_github_remote_url(dir.path(), false).unwrap();
        assert!(remote.is_none());
    }
}
