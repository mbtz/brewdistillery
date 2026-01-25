use crate::config::VersionUpdateConfig;
use crate::errors::AppError;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionUpdateChange {
    pub path: PathBuf,
    pub content: String,
}

pub fn plan_version_update(
    config: &VersionUpdateConfig,
    repo_root: &Path,
    version: &str,
) -> Result<Vec<VersionUpdateChange>, AppError> {
    let mode = config
        .mode
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("none");

    match mode {
        "none" => Ok(Vec::new()),
        "cargo" => update_cargo_version(config, repo_root, version),
        "regex" => update_regex_version(config, repo_root, version),
        _ => Ok(Vec::new()),
    }
}

pub fn apply_version_update(
    config: &VersionUpdateConfig,
    repo_root: &Path,
    version: &str,
    dry_run: bool,
) -> Result<Vec<PathBuf>, AppError> {
    let changes = plan_version_update(config, repo_root, version)?;

    if !dry_run {
        for change in &changes {
            fs::write(&change.path, &change.content)?;
        }
    }

    Ok(changes.into_iter().map(|change| change.path).collect())
}

fn update_cargo_version(
    config: &VersionUpdateConfig,
    repo_root: &Path,
    version: &str,
) -> Result<Vec<VersionUpdateChange>, AppError> {
    let manifest = repo_root.join("Cargo.toml");
    if !manifest.exists() {
        return Err(AppError::InvalidInput(format!(
            "Cargo.toml not found at {}",
            manifest.display()
        )));
    }

    let raw = fs::read_to_string(&manifest)?;
    let value: toml::Value = toml::from_str(&raw).map_err(|err| {
        AppError::InvalidInput(format!(
            "invalid Cargo.toml at {}: {err}",
            manifest.display()
        ))
    })?;

    let root_package_name = value
        .get("package")
        .and_then(|entry| entry.as_table())
        .and_then(|table| table_get_string(table, "name"));
    let workspace_package = value
        .get("workspace")
        .and_then(|entry| entry.as_table())
        .and_then(|table| table.get("package"))
        .and_then(|entry| entry.as_table());

    let target_package = config
        .cargo_package
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());

    if let Some(name) = root_package_name.as_deref() {
        if target_package
            .as_deref()
            .map(|value| value == name)
            .unwrap_or(true)
        {
            let change = plan_toml_version_in_section(&manifest, "package", version)?;
            return Ok(change.into_iter().collect());
        }
    }

    if workspace_package.is_some() && target_package.is_none() {
        let change = plan_toml_version_in_section(&manifest, "workspace.package", version)?;
        return Ok(change.into_iter().collect());
    }

    let target_package = target_package.ok_or_else(|| {
        AppError::InvalidInput(
            "version_update.mode=cargo requires version_update.cargo_package for workspaces without [package] or [workspace.package]"
                .to_string(),
        )
    })?;

    let manifest = find_workspace_manifest(repo_root, &target_package)?;
    let change = plan_toml_version_in_section(&manifest, "package", version)?;
    Ok(change.into_iter().collect())
}

fn update_regex_version(
    config: &VersionUpdateConfig,
    repo_root: &Path,
    version: &str,
) -> Result<Vec<VersionUpdateChange>, AppError> {
    let file = config
        .regex_file
        .as_ref()
        .ok_or_else(|| AppError::InvalidInput("version_update.regex_file is required".to_string()))?
        .to_path_buf();
    let path = if file.is_absolute() {
        file
    } else {
        repo_root.join(file)
    };

    let pattern = config
        .regex_pattern
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            AppError::InvalidInput("version_update.regex_pattern is required".to_string())
        })?;
    let replacement_template = config
        .regex_replacement
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            AppError::InvalidInput("version_update.regex_replacement is required".to_string())
        })?;

    let replacement = replacement_template.replace("{version}", version);
    let regex = Regex::new(pattern).map_err(|err| {
        AppError::InvalidInput(format!(
            "invalid version_update.regex_pattern '{pattern}': {err}"
        ))
    })?;

    if !path.exists() {
        return Err(AppError::InvalidInput(format!(
            "version_update.regex_file not found at {}",
            path.display()
        )));
    }

    let content = fs::read_to_string(&path)?;
    if !regex.is_match(&content) {
        return Err(AppError::InvalidInput(format!(
            "version_update.regex_pattern did not match {}",
            path.display()
        )));
    }

    let updated = regex
        .replace_all(&content, replacement.as_str())
        .to_string();
    if updated == content {
        return Ok(Vec::new());
    }

    Ok(vec![VersionUpdateChange {
        path,
        content: updated,
    }])
}

fn plan_toml_version_in_section(
    path: &Path,
    section: &str,
    version: &str,
) -> Result<Option<VersionUpdateChange>, AppError> {
    let content = fs::read_to_string(path)?;
    let mut output = Vec::new();
    let mut in_section = false;
    let mut updated = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if is_table_header(trimmed) {
            in_section = header_name(trimmed) == section;
            output.push(line.to_string());
            continue;
        }

        if in_section {
            if let Some(updated_line) = replace_version_line(line, version) {
                output.push(updated_line);
                updated = true;
                continue;
            }
        }

        output.push(line.to_string());
    }

    if !updated {
        return Err(AppError::InvalidInput(format!(
            "version_update.mode=cargo could not find version in [{}] at {}",
            section,
            path.display()
        )));
    }

    let mut joined = output.join("\n");
    if content.ends_with('\n') {
        joined.push('\n');
    }

    if joined == content {
        return Ok(None);
    }

    Ok(Some(VersionUpdateChange {
        path: path.to_path_buf(),
        content: joined,
    }))
}

fn replace_version_line(line: &str, version: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let indent = &line[..line.len() - trimmed.len()];
    let mut parts = trimmed.splitn(2, '#');
    let left = parts.next().unwrap_or("");
    let comment = parts.next();

    let mut key_value = left.splitn(2, '=');
    let key = key_value.next()?.trim();
    let _value = key_value.next()?;

    if key != "version" {
        return None;
    }

    let mut updated = format!("{}version = \"{}\"", indent, version);
    if let Some(comment) = comment {
        if !comment.trim().is_empty() {
            updated.push_str(" #");
            updated.push_str(comment.trim_start());
        }
    }
    Some(updated)
}

fn is_table_header(trimmed: &str) -> bool {
    trimmed.starts_with('[') && trimmed.ends_with(']')
}

fn header_name(trimmed: &str) -> &str {
    trimmed.trim_start_matches('[').trim_end_matches(']').trim()
}

fn table_get_string(table: &toml::value::Table, key: &str) -> Option<String> {
    table
        .get(key)
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
}

fn find_workspace_manifest(root: &Path, package_name: &str) -> Result<PathBuf, AppError> {
    let mut matches = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if should_skip_dir(&path) {
                    continue;
                }
                stack.push(path);
                continue;
            }
            if path.file_name().and_then(|name| name.to_str()) == Some("Cargo.toml") {
                if let Some(name) = package_name_from_manifest(&path)? {
                    if name == package_name {
                        matches.push(path);
                    }
                }
            }
        }
    }

    if matches.is_empty() {
        return Err(AppError::InvalidInput(format!(
            "Cargo package '{package_name}' not found in workspace"
        )));
    }
    if matches.len() > 1 {
        let listed = matches
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(AppError::InvalidInput(format!(
            "multiple Cargo packages named '{package_name}' found: {listed}"
        )));
    }

    Ok(matches.remove(0))
}

fn package_name_from_manifest(path: &Path) -> Result<Option<String>, AppError> {
    let raw = fs::read_to_string(path)?;
    let value: toml::Value = toml::from_str(&raw).map_err(|err| {
        AppError::InvalidInput(format!("invalid Cargo.toml at {}: {err}", path.display()))
    })?;
    Ok(value
        .get("package")
        .and_then(|entry| entry.as_table())
        .and_then(|table| table_get_string(table, "name")))
}

fn should_skip_dir(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    matches!(name, ".git" | "target" | "node_modules" | ".distill")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn planning_does_not_write_files() {
        let dir = tempdir().unwrap();
        let manifest = dir.path().join("Cargo.toml");
        fs::write(
            &manifest,
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        let config = VersionUpdateConfig {
            mode: Some("cargo".to_string()),
            cargo_package: None,
            regex_file: None,
            regex_pattern: None,
            regex_replacement: None,
            extra: Default::default(),
        };

        let planned = plan_version_update(&config, dir.path(), "1.2.3").unwrap();
        assert_eq!(planned.len(), 1);
        assert!(planned[0].content.contains("version = \"1.2.3\""));

        let unchanged = fs::read_to_string(&manifest).unwrap();
        assert!(unchanged.contains("version = \"0.1.0\""));
    }

    #[test]
    fn updates_cargo_package_version() {
        let dir = tempdir().unwrap();
        let manifest = dir.path().join("Cargo.toml");
        fs::write(
            &manifest,
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        let config = VersionUpdateConfig {
            mode: Some("cargo".to_string()),
            cargo_package: None,
            regex_file: None,
            regex_pattern: None,
            regex_replacement: None,
            extra: Default::default(),
        };

        let changed = apply_version_update(&config, dir.path(), "1.2.3", false).unwrap();
        assert_eq!(changed, vec![manifest.clone()]);
        let updated = fs::read_to_string(&manifest).unwrap();
        assert!(updated.contains("version = \"1.2.3\""));
    }

    #[test]
    fn prefers_root_package_when_workspace_package_present() {
        let dir = tempdir().unwrap();
        let manifest = dir.path().join("Cargo.toml");
        fs::write(
            &manifest,
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n\n[workspace.package]\nversion = \"9.9.9\"\n",
        )
        .unwrap();

        let config = VersionUpdateConfig {
            mode: Some("cargo".to_string()),
            cargo_package: None,
            regex_file: None,
            regex_pattern: None,
            regex_replacement: None,
            extra: Default::default(),
        };

        let changed = apply_version_update(&config, dir.path(), "1.2.3", false).unwrap();
        assert_eq!(changed, vec![manifest.clone()]);

        let updated = fs::read_to_string(&manifest).unwrap();
        assert!(updated.contains("[package]\nname = \"demo\"\nversion = \"1.2.3\""));
        assert!(updated.contains("[workspace.package]\nversion = \"9.9.9\""));
    }

    #[test]
    fn updates_workspace_package_when_root_package_missing() {
        let dir = tempdir().unwrap();
        let manifest = dir.path().join("Cargo.toml");
        fs::write(&manifest, "[workspace.package]\nversion = \"0.5.0\"\n").unwrap();

        let config = VersionUpdateConfig {
            mode: Some("cargo".to_string()),
            cargo_package: None,
            regex_file: None,
            regex_pattern: None,
            regex_replacement: None,
            extra: Default::default(),
        };

        let changed = apply_version_update(&config, dir.path(), "2.0.0", false).unwrap();
        assert_eq!(changed, vec![manifest.clone()]);

        let updated = fs::read_to_string(&manifest).unwrap();
        assert!(updated.contains("version = \"2.0.0\""));
    }

    #[test]
    fn updates_workspace_member_version() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/app\"]\n",
        )
        .unwrap();

        let member_dir = dir.path().join("crates/app");
        fs::create_dir_all(&member_dir).unwrap();
        let member_manifest = member_dir.join("Cargo.toml");
        fs::write(
            &member_manifest,
            "[package]\nname = \"app\"\nversion = \"0.2.0\"\n",
        )
        .unwrap();

        let config = VersionUpdateConfig {
            mode: Some("cargo".to_string()),
            cargo_package: Some("app".to_string()),
            regex_file: None,
            regex_pattern: None,
            regex_replacement: None,
            extra: Default::default(),
        };

        let changed = apply_version_update(&config, dir.path(), "2.0.0", false).unwrap();
        assert_eq!(changed, vec![member_manifest.clone()]);
        let updated = fs::read_to_string(&member_manifest).unwrap();
        assert!(updated.contains("version = \"2.0.0\""));
    }

    #[test]
    fn requires_cargo_package_for_workspace_without_root_versions() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/app\"]\n",
        )
        .unwrap();

        let config = VersionUpdateConfig {
            mode: Some("cargo".to_string()),
            cargo_package: None,
            regex_file: None,
            regex_pattern: None,
            regex_replacement: None,
            extra: Default::default(),
        };

        let err = apply_version_update(&config, dir.path(), "1.0.0", false).unwrap_err();
        assert_eq!(
            err.to_string(),
            "version_update.mode=cargo requires version_update.cargo_package for workspaces without [package] or [workspace.package]"
        );
    }

    #[test]
    fn updates_regex_file() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("version.txt");
        fs::write(&file, "VERSION=0.1.0\n").unwrap();

        let config = VersionUpdateConfig {
            mode: Some("regex".to_string()),
            cargo_package: None,
            regex_file: Some(PathBuf::from("version.txt")),
            regex_pattern: Some("VERSION=\\d+\\.\\d+\\.\\d+".to_string()),
            regex_replacement: Some("VERSION={version}".to_string()),
            extra: Default::default(),
        };

        let changed = apply_version_update(&config, dir.path(), "3.1.4", false).unwrap();
        assert_eq!(changed, vec![file.clone()]);
        let updated = fs::read_to_string(&file).unwrap();
        assert!(updated.contains("VERSION=3.1.4"));
    }
}
