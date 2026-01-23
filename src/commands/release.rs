use crate::asset_selection::{select_asset_name, AssetSelectionOptions};
use crate::cli::ReleaseArgs;
use crate::config::{ArtifactTarget, Config};
use crate::context::AppContext;
use crate::errors::AppError;
use crate::formula::{AssetMatrix, FormulaAsset, FormulaSpec, Os, Arch, TargetAsset};
use crate::host::github::GitHubClient;
use crate::host::{HostClient, Release};
use crate::preview::{preview_and_apply, PlannedWrite, RepoPlan};
use crate::version::{resolve_version_tag, ResolvedVersionTag};
use crate::version_update::apply_version_update;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::{Builder as TempBuilder, TempDir};

const DEFAULT_ASSET_MAX_BYTES: u64 = 200 * 1024 * 1024;
const DEFAULT_TARBALL_URL_TEMPLATE: &str =
    "https://github.com/{owner}/{repo}/archive/refs/tags/{tag}.tar.gz";

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
    tag_format: Option<String>,
    tarball_url_template: Option<String>,
    commit_message_template: Option<String>,
    install_block: Option<String>,
    template_path: Option<PathBuf>,
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

    let client = GitHubClient::from_env(resolved.host_api_base.as_deref())?;
    let version_tag = resolve_version_tag(args.version.as_deref(), args.tag.as_deref())?;

    let (version, tag_to_create, asset_matrix) = match resolved.artifact_strategy.as_str() {
        "release-asset" => {
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

            let version = version_tag
                .version
                .clone()
                .unwrap_or(release_version);
            let tag_to_create = if args.skip_tag {
                None
            } else {
                Some(resolve_tag_name(
                    &version,
                    &version_tag,
                    resolved.tag_format.as_deref(),
                    &release.tag_name,
                )?)
            };

            let asset_matrix = build_assets(&client, &release, &resolved, &version, args.dry_run)?;
            (version, tag_to_create, asset_matrix)
        }
        "source-tarball" => {
            validate_source_tarball_inputs(&resolved)?;
            let (version, source_tag) = resolve_source_tarball_version_and_tag(
                &client,
                &resolved,
                &version_tag,
                args.include_prerelease,
            )?;
            let tarball = build_tarball_asset(
                &client,
                &resolved,
                &version,
                &source_tag,
                args.dry_run,
            )?;
            let tag_to_create = if args.skip_tag {
                None
            } else {
                Some(resolve_tag_name(
                    &version,
                    &version_tag,
                    resolved.tag_format.as_deref(),
                    &source_tag,
                )?)
            };
            (version, tag_to_create, AssetMatrix::Universal(tarball))
        }
        _ => {
            return Err(AppError::InvalidInput(format!(
                "artifact.strategy '{}' is not supported yet (use 'release-asset' or 'source-tarball')",
                resolved.artifact_strategy
            )))
        }
    };

    ensure_formula_target(&resolved.formula_path, &version, args.force)?;

    let version_update_mode = ctx
        .config
        .version_update
        .mode
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("none");
    let needs_cli_repo = tag_to_create.is_some() || version_update_mode != "none";

    if !args.allow_dirty {
        ensure_clean_repo(&resolved.tap_root, "tap repo")?;
        if needs_cli_repo {
            let cli_root = ctx.repo.git_root.as_ref().ok_or_else(|| {
                AppError::GitState("cli repo is not a git repository".to_string())
            })?;
            ensure_clean_repo(cli_root, "cli repo")?;
        }
    }

    if version_update_mode != "none" {
        let cli_root = ctx.repo.git_root.as_ref().ok_or_else(|| {
            AppError::GitState("cli repo is not a git repository".to_string())
        })?;
        let updated_files = apply_version_update(
            &ctx.config.version_update,
            cli_root,
            &version,
            args.dry_run,
        )?;
        if !updated_files.is_empty() {
            if args.dry_run {
                println!(
                    "dry-run: would update version in {}",
                    format_paths(&updated_files)
                );
            } else {
                let message = build_version_update_message(&version);
                commit_version_update(cli_root, &updated_files, &message)?;
                if !args.no_push {
                    push_repo(cli_root)?;
                }
            }
        }
    }

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

    let rendered = if let Some(path) = resolved.template_path.as_ref() {
        let template = read_template(path)?;
        spec.render_with_template(&template)?
    } else {
        spec.render()?
    };

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

    if args.dry_run {
        drop(tap_root);
        return Ok(());
    }

    let commit_message = build_commit_message(
        resolved.commit_message_template.as_deref(),
        &resolved.formula_name,
        &version,
    );

    if !preview.changed_files.is_empty() {
        commit_formula_update(
            &resolved.tap_root,
            &resolved.formula_path,
            &commit_message,
        )?;
        if !args.no_push {
            push_repo(&resolved.tap_root)?;
        }
    } else {
        println!("release: no formula changes to commit");
    }

    if let Some(tag_name) = tag_to_create.as_deref() {
        let cli_root = ctx.repo.git_root.as_ref().ok_or_else(|| {
            AppError::GitState("cli repo is not a git repository".to_string())
        })?;
        create_tag(cli_root, tag_name)?;
        if !args.no_push {
            push_tag(cli_root, tag_name)?;
        }
    }

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
        tag_format: config.release.tag_format.clone(),
        tarball_url_template: config.release.tarball_url_template.clone(),
        commit_message_template: config.release.commit_message_template.clone(),
        install_block: config.template.install_block.clone(),
        template_path: config.template.path.clone(),
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

fn resolve_tag_name(
    version: &str,
    version_tag: &ResolvedVersionTag,
    tag_format: Option<&str>,
    release_tag: &str,
) -> Result<String, AppError> {
    if let Some(tag) = version_tag.tag.as_ref() {
        return Ok(tag.clone());
    }

    if let Some(format) = tag_format {
        let trimmed = format.trim();
        if trimmed.is_empty() {
            return Ok(version.to_string());
        }
        if trimmed.contains("{version}") {
            return Ok(trimmed.replace("{version}", version));
        }
        return Ok(trimmed.to_string());
    }

    if !release_tag.trim().is_empty() {
        return Ok(release_tag.trim().to_string());
    }

    Ok(version.to_string())
}

fn build_commit_message(template: Option<&str>, formula: &str, version: &str) -> String {
    let default = format!(
        "feature: Updated formula for version '{version}'"
    );
    let template = template.map(str::trim).filter(|value| !value.is_empty());
    let Some(template) = template else {
        return default;
    };
    let mut output = template.replace("{version}", version);
    if output.contains("{formula}") {
        output = output.replace("{formula}", formula);
    }
    let trimmed = output.trim();
    if trimmed.is_empty() {
        default
    } else {
        trimmed.to_string()
    }
}

fn build_version_update_message(version: &str) -> String {
    format!("chore: Bump version to '{version}'")
}

fn format_paths(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn ensure_clean_repo(repo: &Path, label: &str) -> Result<(), AppError> {
    let output = run_git(repo, &["status", "--porcelain"])?;
    let status = String::from_utf8_lossy(&output.stdout);
    if status.trim().is_empty() {
        return Ok(());
    }
    Err(AppError::GitState(format!(
        "{label} has uncommitted changes; re-run with --allow-dirty to continue"
    )))
}

fn commit_formula_update(
    repo: &Path,
    formula_path: &Path,
    message: &str,
) -> Result<(), AppError> {
    let relative = formula_path
        .strip_prefix(repo)
        .map(|path| path.to_path_buf())
        .map_err(|_| {
            AppError::GitState(format!(
                "formula path {} is not inside tap repo {}",
                formula_path.display(),
                repo.display()
            ))
        })?;

    run_git(repo, &["add", relative.to_str().ok_or_else(|| {
        AppError::GitState("formula path contains invalid UTF-8".to_string())
    })?])?;

    let diff = run_git(repo, &["diff", "--cached", "--name-only"])?;
    if String::from_utf8_lossy(&diff.stdout).trim().is_empty() {
        return Ok(());
    }

    run_git(repo, &["commit", "-m", message])?;
    Ok(())
}

fn commit_version_update(
    repo: &Path,
    files: &[PathBuf],
    message: &str,
) -> Result<(), AppError> {
    for path in files {
        let relative = path
            .strip_prefix(repo)
            .map(|path| path.to_path_buf())
            .map_err(|_| {
                AppError::GitState(format!(
                    "version update file {} is not inside cli repo {}",
                    path.display(),
                    repo.display()
                ))
            })?;

        let relative = relative.to_str().ok_or_else(|| {
            AppError::GitState("version update path contains invalid UTF-8".to_string())
        })?;

        run_git(repo, &["add", relative])?;
    }

    let diff = run_git(repo, &["diff", "--cached", "--name-only"])?;
    if String::from_utf8_lossy(&diff.stdout).trim().is_empty() {
        return Ok(());
    }

    run_git(repo, &["commit", "-m", message])?;
    Ok(())
}

fn create_tag(repo: &Path, tag: &str) -> Result<(), AppError> {
    let exists = run_git(repo, &["tag", "--list", tag])?;
    if !String::from_utf8_lossy(&exists.stdout).trim().is_empty() {
        return Err(AppError::GitState(format!(
            "tag '{tag}' already exists; re-run with --skip-tag or choose a new version"
        )));
    }
    run_git(repo, &["tag", tag])?;
    Ok(())
}

fn push_repo(repo: &Path) -> Result<(), AppError> {
    let remote = select_git_remote(repo)?;
    run_git(repo, &["push", &remote, "HEAD"])?;
    Ok(())
}

fn push_tag(repo: &Path, tag: &str) -> Result<(), AppError> {
    let remote = select_git_remote(repo)?;
    run_git(repo, &["push", &remote, tag])?;
    Ok(())
}

fn select_git_remote(repo: &Path) -> Result<String, AppError> {
    let output = run_git(repo, &["remote"])?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let remotes = stdout
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    if remotes.is_empty() {
        return Err(AppError::GitState("git remote not configured".to_string()));
    }

    if remotes.iter().any(|remote| *remote == "origin") {
        return Ok("origin".to_string());
    }

    if remotes.len() == 1 {
        return Ok(remotes[0].to_string());
    }

    Err(AppError::GitState(
        "multiple git remotes found; configure origin or use a single remote".to_string(),
    ))
}

fn run_git(repo: &Path, args: &[&str]) -> Result<std::process::Output, AppError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .map_err(|err| AppError::GitState(format!("failed to run git: {err}")))?;

    if output.status.success() {
        return Ok(output);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut message = format!("git command failed: git {}", args.join(" "));
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
    Err(AppError::GitState(message))
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
    let mut macos: Option<FormulaAsset> = None;
    let mut linux: Option<FormulaAsset> = None;
    let mut mode: Option<TargetMode> = None;
    for (key, target) in &resolved.targets {
        match parse_target_key(key)? {
            TargetKey::Os(os) => {
                if matches!(mode, Some(TargetMode::PerTarget)) {
                    return Err(AppError::InvalidInput(
                        "target keys must be all <os> or all <os>-<arch>".to_string(),
                    ));
                }
                mode = Some(TargetMode::PerOs);
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
                    arch: None,
                };
                let asset = select_asset_for_release(client, release, selection, dry_run)?;
                match os {
                    Os::Darwin => {
                        if macos.is_some() {
                            return Err(AppError::InvalidInput(
                                "duplicate target entry for macos".to_string(),
                            ));
                        }
                        macos = Some(asset);
                    }
                    Os::Linux => {
                        if linux.is_some() {
                            return Err(AppError::InvalidInput(
                                "duplicate target entry for linux".to_string(),
                            ));
                        }
                        linux = Some(asset);
                    }
                }
            }
            TargetKey::OsArch(os, arch) => {
                if matches!(mode, Some(TargetMode::PerOs)) {
                    return Err(AppError::InvalidInput(
                        "target keys must be all <os> or all <os>-<arch>".to_string(),
                    ));
                }
                mode = Some(TargetMode::PerTarget);
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
        }
    }

    match mode {
        Some(TargetMode::PerOs) => Ok(AssetMatrix::PerOs { macos, linux }),
        Some(TargetMode::PerTarget) => Ok(AssetMatrix::PerTarget(targets)),
        None => Ok(AssetMatrix::Universal(select_asset_for_release(
            client,
            release,
            AssetSelectionOptions {
                asset_name: resolved.asset_name.clone(),
                asset_template: resolved.asset_template.clone(),
                project_name: Some(resolved.project_name.clone()),
                version: Some(version.to_string()),
                os: None,
                arch: None,
            },
            dry_run,
        )?)),
    }
}

fn validate_source_tarball_inputs(resolved: &ReleaseContext) -> Result<(), AppError> {
    if resolved.asset_name.is_some() || resolved.asset_template.is_some() {
        return Err(AppError::InvalidInput(
            "source-tarball strategy does not support asset_name or asset_template".to_string(),
        ));
    }
    if !resolved.targets.is_empty() {
        return Err(AppError::InvalidInput(
            "source-tarball strategy does not support artifact.targets".to_string(),
        ));
    }
    Ok(())
}

fn resolve_source_tarball_version_and_tag(
    client: &GitHubClient,
    resolved: &ReleaseContext,
    version_tag: &ResolvedVersionTag,
    include_prerelease: bool,
) -> Result<(String, String), AppError> {
    let mut release_tag: Option<String> = None;
    let version = if let Some(version) = version_tag.version.clone() {
        version
    } else {
        let release = client.latest_release(
            &resolved.host_owner,
            &resolved.host_repo,
            include_prerelease,
        )?;
        release_tag = Some(release.tag_name.clone());
        normalized_version_from_tag(&release.tag_name)?
    };

    let source_tag = resolve_source_tarball_tag(
        version_tag,
        resolved.tag_format.as_deref(),
        release_tag.as_deref(),
        &version,
    )?;

    Ok((version, source_tag))
}

fn resolve_source_tarball_tag(
    version_tag: &ResolvedVersionTag,
    tag_format: Option<&str>,
    release_tag: Option<&str>,
    version: &str,
) -> Result<String, AppError> {
    if let Some(tag) = version_tag.tag.as_deref() {
        return Ok(tag.to_string());
    }

    if let Some(tag) = release_tag.map(str::trim).filter(|value| !value.is_empty()) {
        return Ok(tag.to_string());
    }

    if let Some(format) = tag_format.map(str::trim).filter(|value| !value.is_empty()) {
        if format.contains("{version}") {
            return Ok(format.replace("{version}", version));
        }
        return Ok(format.to_string());
    }

    if version.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "source-tarball requires a non-empty version".to_string(),
        ));
    }

    Ok(version.to_string())
}

fn build_tarball_asset(
    client: &GitHubClient,
    resolved: &ReleaseContext,
    version: &str,
    tag: &str,
    dry_run: bool,
) -> Result<FormulaAsset, AppError> {
    let url = render_tarball_url(
        resolved.tarball_url_template.as_deref(),
        &resolved.host_owner,
        &resolved.host_repo,
        tag,
        version,
    )?;

    let sha256 = if dry_run {
        println!("dry-run: would download {}", url);
        "DRY-RUN".to_string()
    } else {
        client.download_sha256(&url, Some(DEFAULT_ASSET_MAX_BYTES))?
    };

    Ok(FormulaAsset { url, sha256 })
}

fn render_tarball_url(
    template: Option<&str>,
    owner: &str,
    repo: &str,
    tag: &str,
    version: &str,
) -> Result<String, AppError> {
    let owner = owner.trim();
    let repo = repo.trim();
    if owner.is_empty() || repo.is_empty() {
        return Err(AppError::InvalidInput(
            "tarball_url_template requires host owner and repo".to_string(),
        ));
    }

    let template = template
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_TARBALL_URL_TEMPLATE);

    let mut output = template.to_string();

    if output.contains("{owner}") {
        output = output.replace("{owner}", owner);
    }
    if output.contains("{repo}") {
        output = output.replace("{repo}", repo);
    }
    if output.contains("{tag}") {
        if tag.trim().is_empty() {
            return Err(AppError::InvalidInput(
                "tarball_url_template requires {tag}".to_string(),
            ));
        }
        output = output.replace("{tag}", tag);
    }
    if output.contains("{version}") {
        if version.trim().is_empty() {
            return Err(AppError::InvalidInput(
                "tarball_url_template requires {version}".to_string(),
            ));
        }
        output = output.replace("{version}", version);
    }

    let trimmed = output.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidInput(
            "tarball_url_template rendered empty output".to_string(),
        ));
    }

    Ok(trimmed.to_string())
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
        client.download_sha256(&asset.download_url, Some(DEFAULT_ASSET_MAX_BYTES))?
    };

    Ok(FormulaAsset {
        url: asset.download_url.clone(),
        sha256,
    })
}

fn read_template(path: &Path) -> Result<String, AppError> {
    if !path.exists() {
        return Err(AppError::InvalidInput(format!(
            "template not found at {}",
            path.display()
        )));
    }
    std::fs::read_to_string(path).map_err(|err| {
        AppError::InvalidInput(format!(
            "failed to read template at {}: {err}",
            path.display()
        ))
    })
}

#[derive(Debug, Clone, Copy)]
enum TargetKey {
    Os(Os),
    OsArch(Os, Arch),
}

#[derive(Debug, Clone, Copy)]
enum TargetMode {
    PerOs,
    PerTarget,
}

fn parse_target_key(key: &str) -> Result<TargetKey, AppError> {
    let lower = key.trim().to_ascii_lowercase();
    if lower.is_empty() {
        return Err(AppError::InvalidInput(format!(
            "invalid target key '{key}': expected <os> or <os>-<arch>"
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
                "invalid target key '{key}': expected <os> or <os>-<arch>"
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
                "invalid target key '{key}': expected <os> or <os>-<arch>"
            )))
        }
    };

    let os = os.ok_or_else(|| {
        AppError::InvalidInput(format!(
            "invalid target key '{key}': expected <os> or <os>-<arch>"
        ))
    })?;

    match arch {
        Some(arch) => Ok(TargetKey::OsArch(os, arch)),
        None => Ok(TargetKey::Os(os)),
    }
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
        match parse_target_key("darwin-arm64").unwrap() {
            TargetKey::OsArch(os, arch) => {
                assert_eq!(os, Os::Darwin);
                assert_eq!(arch, Arch::Arm64);
            }
            _ => panic!("expected os+arch target"),
        }

        match parse_target_key("linux_x86_64").unwrap() {
            TargetKey::OsArch(os, arch) => {
                assert_eq!(os, Os::Linux);
                assert_eq!(arch, Arch::Amd64);
            }
            _ => panic!("expected os+arch target"),
        }

        match parse_target_key("darwin").unwrap() {
            TargetKey::Os(os) => assert_eq!(os, Os::Darwin),
            _ => panic!("expected os-only target"),
        }
    }

    #[test]
    fn derives_tap_root_from_formula_path() {
        let formula = PathBuf::from("/tmp/homebrew-brewtool/Formula/brewtool.rb");
        let root = tap_root_from_formula_path(&formula).unwrap();
        assert_eq!(root, PathBuf::from("/tmp/homebrew-brewtool"));
    }

    #[test]
    fn renders_default_tarball_url() {
        let url = render_tarball_url(
            None,
            "acme",
            "brewtool",
            "v1.2.3",
            "1.2.3",
        )
        .unwrap();
        assert_eq!(
            url,
            "https://github.com/acme/brewtool/archive/refs/tags/v1.2.3.tar.gz"
        );
    }

    #[test]
    fn renders_custom_tarball_template() {
        let url = render_tarball_url(
            Some("https://example.com/{owner}/{repo}/releases/{tag}/{version}.tgz"),
            "acme",
            "brewtool",
            "v1.2.3",
            "1.2.3",
        )
        .unwrap();
        assert_eq!(
            url,
            "https://example.com/acme/brewtool/releases/v1.2.3/1.2.3.tgz"
        );
    }

    #[test]
    fn source_tarball_tag_prefers_explicit_tag() {
        let version_tag = ResolvedVersionTag {
            version: Some("1.2.3".to_string()),
            tag: Some("v1.2.3".to_string()),
            normalized_tag: Some("1.2.3".to_string()),
        };

        let tag = resolve_source_tarball_tag(&version_tag, None, None, "1.2.3").unwrap();
        assert_eq!(tag, "v1.2.3");
    }
}
