use crate::asset_selection::{select_asset_name, AssetSelectionOptions};
use crate::cli::ReleaseArgs;
use crate::config::{ArtifactTarget, Config};
use crate::context::AppContext;
use crate::errors::AppError;
use crate::formula::{AssetMatrix, FormulaAsset, FormulaSpec, Os, Arch, TargetAsset};
use crate::host::github::GitHubClient;
use crate::host::{HostClient, Release};
use crate::preview::{preview_and_apply, PlannedWrite, RepoPlan};
use crate::version::resolve_version_tag;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::{Builder as TempBuilder, TempDir};

#[derive(Debug)]
struct ReleaseContext {
    tap_root: PathBuf,
    formula_path: PathBuf,
    formula_name: String,
    project_name: String,
    description: String,
    homepage: String,
    license: String,
    bins: Vec<String>,
    artifact_strategy: String,
    asset_name: Option<String>,
    asset_template: Option<String>,
    targets: BTreeMap<String, ArtifactTarget>,
    host_owner: String,
    host_repo: String,
    host_api_base: Option<String>,
    install_block: Option<String>,
}

#[derive(Debug)]
struct TapRoot {
    path: Option<PathBuf>,
    temp_dir: Option<TempDir>,
    cloned_from: Option<String>,
}

pub fn run(ctx: &AppContext, args: &ReleaseArgs) -> Result<(), AppError> {
    if !ctx.config_path.exists() {
        return Err(AppError::MissingConfig(format!(
            "config not found at {}",
            ctx.config_path.display()
        )));
    }

    let tap_root = resolve_tap_root_for_release(ctx, args)?;
    if let (Some(path), Some(remote)) = (tap_root.path.as_ref(), tap_root.cloned_from.as_ref()) {
        println!(
            "release: cloned tap repo from {} to {}",
            remote,
            path.display()
        );
    }
    let _tap_root_guard = tap_root.temp_dir.as_ref();

    let resolved = resolve_release_context(ctx, args, tap_root.path.as_ref())?;

    if resolved.artifact_strategy != "release-asset" {
        return Err(AppError::InvalidInput(format!(
            "artifact.strategy '{}' is not supported yet (use 'release-asset')",
            resolved.artifact_strategy
        )));
    }

    let client = GitHubClient::from_env(resolved.host_api_base.as_deref())?;
    let version_tag = resolve_version_tag(args.version.as_deref(), args.tag.as_deref())?;

    let release = if let Some(tag) = version_tag.tag.as_deref() {
        client.release_by_tag(
            &resolved.host_owner,
            &resolved.host_repo,
            tag,
            args.include_prerelease,
        )?
    } else {
        client.latest_release(
            &resolved.host_owner,
            &resolved.host_repo,
            args.include_prerelease,
        )?
    };

    let release_version = normalized_version_from_tag(&release.tag_name)?;
    if let Some(expected) = version_tag.version.as_deref() {
        if expected != release_version {
            return Err(AppError::InvalidInput(format!(
                "GitHub release tag '{}' does not match requested version '{}'",
                release.tag_name, expected
            )));
        }
    }

    let version = version_tag.version.unwrap_or(release_version);

    ensure_formula_target(&resolved.formula_path, &version, args.force)?;

    let asset_matrix = build_assets(&client, &release, &resolved, &version, args.dry_run)?;

    let spec = FormulaSpec {
        name: resolved.formula_name.clone(),
        desc: resolved.description.clone(),
        homepage: resolved.homepage.clone(),
        license: resolved.license.clone(),
        version: version.clone(),
        bins: resolved.bins.clone(),
        assets: asset_matrix,
        install_block: resolved.install_block.clone(),
    };

    let rendered = spec.render()?;

    let plan = RepoPlan {
        label: "tap".to_string(),
        repo_root: resolved.tap_root.clone(),
        writes: vec![PlannedWrite {
            path: resolved.formula_path.clone(),
            content: rendered,
        }],
    };

    let preview = preview_and_apply(&[plan], args.dry_run)?;
    print_preview(&preview);

    drop(tap_root);
    Ok(())
}

fn resolve_release_context(
    ctx: &AppContext,
    args: &ReleaseArgs,
    tap_path_override: Option<&PathBuf>,
) -> Result<ReleaseContext, AppError> {
    let config = &ctx.config;
    let mut missing = Vec::new();

    let tap_path = tap_path_override.cloned();
    let formula_name = resolve_string(None, config.tap.formula.as_ref()).unwrap_or_default();
    if formula_name.is_empty() {
        missing.push("tap.formula".to_string());
    }

    let formula_path = resolve_formula_path(config, tap_path.as_ref());
    let formula_path = match formula_path {
        Some(path) => path,
        None => {
            missing.push("tap.path or tap.formula_path".to_string());
            PathBuf::new()
        }
    };

    let tap_root = tap_path
        .clone()
        .or_else(|| tap_root_from_formula_path(&formula_path))
        .unwrap_or_else(|| ctx.cwd.clone());

    let description = resolve_string(None, config.project.description.as_ref()).unwrap_or_default();
    if description.trim().is_empty() {
        missing.push("project.description".to_string());
    }

    let homepage = resolve_string(None, config.project.homepage.as_ref()).unwrap_or_default();
    if homepage.trim().is_empty() {
        missing.push("project.homepage".to_string());
    }

    let license = resolve_string(None, config.project.license.as_ref()).unwrap_or_default();
    if license.trim().is_empty() {
        missing.push("project.license".to_string());
    }

    let bins = normalize_bins(config.project.bin.clone());
    if bins.is_empty() {
        missing.push("project.bin".to_string());
    }

    let project_name = resolve_string(None, config.project.name.as_ref())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| formula_name.clone());

    let artifact_strategy = resolve_string(args.artifact_strategy.as_ref(), config.artifact.strategy.as_ref())
        .unwrap_or_default();
    if artifact_strategy.trim().is_empty() {
        missing.push("artifact.strategy".to_string());
    }

    let asset_template = resolve_string(args.asset_template.as_ref(), config.artifact.asset_template.as_ref());
    let asset_name = resolve_string(args.asset_name.as_ref(), config.artifact.asset_name.as_ref());

    let host_owner = resolve_string(
        args.host_owner.as_ref(),
        config.artifact.owner.as_ref(),
    )
    .or_else(|| resolve_string(None, config.cli.owner.as_ref()))
    .unwrap_or_default();
    if host_owner.is_empty() {
        missing.push("host-owner".to_string());
    }

    let host_repo = resolve_string(
        args.host_repo.as_ref(),
        config.artifact.repo.as_ref(),
    )
    .or_else(|| resolve_string(None, config.cli.repo.as_ref()))
    .unwrap_or_default();
    if host_repo.is_empty() {
        missing.push("host-repo".to_string());
    }

    if let Some(provider) = config.host.provider.as_deref() {
        let normalized = provider.trim();
        if !normalized.is_empty() && normalized != "github" {
            return Err(AppError::InvalidInput(format!(
                "host.provider '{}' is not supported yet",
                provider
            )));
        }
    }

    if !missing.is_empty() {
        missing.sort();
        missing.dedup();
        return Err(AppError::MissingConfig(format!(
            "missing required fields for --non-interactive: {}",
            missing.join(", ")
        )));
    }

    Ok(ReleaseContext {
        tap_root,
        formula_path,
        formula_name,
        project_name,
        description,
        homepage,
        license,
        bins,
        artifact_strategy,
        asset_name,
        asset_template,
        targets: config.artifact.targets.clone(),
        host_owner,
        host_repo,
        host_api_base: config.host.api_base.clone(),
        install_block: config.template.install_block.clone(),
    })
}

fn resolve_tap_root_for_release(ctx: &AppContext, args: &ReleaseArgs) -> Result<TapRoot, AppError> {
    let tap_path = args.tap_path.clone().or_else(|| ctx.config.tap.path.clone());
    if tap_path.is_some() {
        return Ok(TapRoot {
            path: tap_path,
            temp_dir: None,
            cloned_from: None,
        });
    }

    if let Some(formula_path) = ctx.config.tap.formula_path.as_ref() {
        if formula_path.is_absolute() {
            return Ok(TapRoot {
                path: None,
                temp_dir: None,
                cloned_from: None,
            });
        }
    }

    let remote = ctx
        .config
        .tap
        .remote
        .as_deref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty());

    if let Some(remote) = remote {
        let temp_dir = TempBuilder::new()
            .prefix("brewdistillery-tap-")
            .tempdir()?;
        let dest = temp_dir.path().join("tap");
        run_git_clone(remote, &dest)?;
        return Ok(TapRoot {
            path: Some(dest),
            temp_dir: Some(temp_dir),
            cloned_from: Some(remote.to_string()),
        });
    }

    Ok(TapRoot {
        path: None,
        temp_dir: None,
        cloned_from: None,
    })
}

fn resolve_formula_path(config: &Config, tap_path: Option<&PathBuf>) -> Option<PathBuf> {
    if let Some(path) = config.tap.formula_path.as_ref() {
        if path.is_absolute() {
            return Some(path.clone());
        }
        if let Some(tap_path) = tap_path {
            return Some(tap_path.join(path));
        }
        return Some(path.clone());
    }

    let formula = config.tap.formula.as_ref()?;
    let tap_path = tap_path?;
    Some(tap_path.join("Formula").join(format!("{formula}.rb")))
}

fn resolve_string(flag: Option<&String>, config: Option<&String>) -> Option<String> {
    flag.or(config)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
}

fn normalize_bins(mut bins: Vec<String>) -> Vec<String> {
    for entry in bins.iter_mut() {
        *entry = entry.trim().to_string();
    }
    bins.retain(|entry| !entry.is_empty());
    bins.sort();
    bins.dedup();
    bins
}

fn normalized_version_from_tag(tag: &str) -> Result<String, AppError> {
    let resolved = resolve_version_tag(None, Some(tag))?;
    resolved.version.ok_or_else(|| {
        AppError::InvalidInput(format!(
            "GitHub release tag '{}' is not valid semver",
            tag
        ))
    })
}

fn ensure_formula_target(
    formula_path: &Path,
    version: &str,
    allow_force: bool,
) -> Result<(), AppError> {
    if !formula_path.exists() {
        return Err(AppError::InvalidInput(format!(
            "formula file not found at {}",
            formula_path.display()
        )));
    }

    let content = std::fs::read_to_string(formula_path)?;
    if let Some(existing) = extract_formula_version(&content) {
        if existing == version && !allow_force {
            return Err(AppError::InvalidInput(format!(
                "formula already targets version {version}; re-run with --force to re-apply",
            )));
        }
    }

    Ok(())
}

fn extract_formula_version(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("version") {
            let rest = rest.trim_start();
            if let Some(value) = extract_quoted(rest) {
                return Some(value);
            }
        }
    }
    None
}

fn extract_quoted(input: &str) -> Option<String> {
    let mut chars = input.chars();
    if chars.next()? != '"' {
        return None;
    }
    let mut output = String::new();
    for ch in chars {
        if ch == '"' {
            return Some(output);
        }
        output.push(ch);
    }
    None
}

fn build_assets(
    client: &GitHubClient,
    release: &Release,
    resolved: &ReleaseContext,
    version: &str,
    dry_run: bool,
) -> Result<AssetMatrix, AppError> {
    if resolved.targets.is_empty() {
        let selection = AssetSelectionOptions {
            asset_name: resolved.asset_name.clone(),
            asset_template: resolved.asset_template.clone(),
            project_name: Some(resolved.project_name.clone()),
            version: Some(version.to_string()),
            os: None,
            arch: None,
        };
        let asset = select_asset_for_release(client, release, selection, dry_run)?;
        return Ok(AssetMatrix::Universal(asset));
    }

    let mut targets = Vec::new();
    for (key, target) in &resolved.targets {
        let (os, arch) = parse_target_key(key)?;
        let selection = AssetSelectionOptions {
            asset_name: target
                .asset_name
                .clone()
                .or_else(|| resolved.asset_name.clone()),
            asset_template: target
                .asset_template
                .clone()
                .or_else(|| resolved.asset_template.clone()),
            project_name: Some(resolved.project_name.clone()),
            version: Some(version.to_string()),
            os: Some(os),
            arch: Some(arch),
        };
        let asset = select_asset_for_release(client, release, selection, dry_run)?;
        targets.push(TargetAsset { os, arch, asset });
    }

    Ok(AssetMatrix::PerTarget(targets))
}

fn select_asset_for_release(
    client: &GitHubClient,
    release: &Release,
    selection: AssetSelectionOptions,
    dry_run: bool,
) -> Result<FormulaAsset, AppError> {
    let available = release
        .assets
        .iter()
        .map(|asset| asset.name.clone())
        .collect::<Vec<_>>();

    let name = select_asset_name(&available, &selection)?;
    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name == name)
        .ok_or_else(|| {
            AppError::InvalidInput(format!(
                "release asset '{}' missing download URL",
                name
            ))
        })?;

    let sha256 = if dry_run {
        println!("dry-run: would download {}", asset.download_url);
        "DRY-RUN".to_string()
    } else {
        client.download_sha256(&asset.download_url, None)?
    };

    Ok(FormulaAsset {
        url: asset.download_url.clone(),
        sha256,
    })
}

fn parse_target_key(key: &str) -> Result<(Os, Arch), AppError> {
    let lower = key.trim().to_ascii_lowercase();
    if lower.is_empty() {
        return Err(AppError::InvalidInput(format!(
            "invalid target key '{key}': expected <os>-<arch>"
        )));
    }

    let has_darwin = lower.contains("darwin") || lower.contains("macos") || lower.contains("osx");
    let has_linux = lower.contains("linux");

    let os = match (has_darwin, has_linux) {
        (true, false) => Some(Os::Darwin),
        (false, true) => Some(Os::Linux),
        (false, false) => None,
        _ => {
            return Err(AppError::InvalidInput(format!(
                "invalid target key '{key}': expected <os>-<arch>"
            )))
        }
    };

    let has_arm64 = lower.contains("arm64") || lower.contains("aarch64");
    let has_amd64 = lower.contains("amd64") || lower.contains("x86_64") || lower.contains("x64");

    let arch = match (has_arm64, has_amd64) {
        (true, false) => Some(Arch::Arm64),
        (false, true) => Some(Arch::Amd64),
        (false, false) => None,
        _ => {
            return Err(AppError::InvalidInput(format!(
                "invalid target key '{key}': expected <os>-<arch>"
            )))
        }
    };

    let os = os.ok_or_else(|| {
        AppError::InvalidInput(format!(
            "invalid target key '{key}': expected <os>-<arch>"
        ))
    })?;
    let arch = arch.ok_or_else(|| {
        AppError::InvalidInput(format!(
            "invalid target key '{key}': expected <os>-<arch>"
        ))
    })?;

    Ok((os, arch))
}

fn tap_root_from_formula_path(formula_path: &Path) -> Option<PathBuf> {
    let parent = formula_path.parent()?;
    if parent
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "Formula")
        .unwrap_or(false)
    {
        return parent.parent().map(|path| path.to_path_buf());
    }
    Some(parent.to_path_buf())
}

fn run_git_clone(remote: &str, dest: &Path) -> Result<(), AppError> {
    let output = Command::new("git")
        .arg("clone")
        .arg(remote)
        .arg(dest)
        .output()
        .map_err(|err| AppError::GitState(format!("failed to run git clone: {err}")))?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let mut message = format!("failed to clone tap repo from {remote}");
        if !stdout.trim().is_empty() || !stderr.trim().is_empty() {
            message.push_str(":\n");
            if !stdout.trim().is_empty() {
                message.push_str(stdout.trim());
                message.push('\n');
            }
            if !stderr.trim().is_empty() {
                message.push_str(stderr.trim());
            }
        }
        return Err(AppError::GitState(message));
    }

    Ok(())
}

fn print_preview(preview: &crate::preview::PreviewOutput) {
    if !preview.summary.trim().is_empty() {
        println!("{}", preview.summary.trim_end());
    }

    if !preview.diff.trim().is_empty() {
        println!("{}", preview.diff.trim_end());
    }

    if preview.changed_files.is_empty() {
        println!("release: no changes to apply");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_version_from_formula() {
        let content = r#"class Brewtool < Formula
  desc "Brew tool"
  homepage "https://example.com"
  url "https://example.com/brewtool.tar.gz"
  sha256 "deadbeef"
  license "MIT"
  version "1.2.3"
end
"#;
        assert_eq!(extract_formula_version(content), Some("1.2.3".to_string()));
    }

    #[test]
    fn parses_target_keys() {
        let (os, arch) = parse_target_key("darwin-arm64").unwrap();
        assert_eq!(os, Os::Darwin);
        assert_eq!(arch, Arch::Arm64);

        let (os, arch) = parse_target_key("linux_x86_64").unwrap();
        assert_eq!(os, Os::Linux);
        assert_eq!(arch, Arch::Amd64);
    }

    #[test]
    fn derives_tap_root_from_formula_path() {
        let formula = PathBuf::from("/tmp/homebrew-brewtool/Formula/brewtool.rb");
        let root = tap_root_from_formula_path(&formula).unwrap();
        assert_eq!(root, PathBuf::from("/tmp/homebrew-brewtool"));
    }
}
