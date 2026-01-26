use crate::asset_selection::{select_asset_name, AssetSelectionOptions};
use crate::cli::ReleaseArgs;
use crate::config::{ArtifactConfig, ArtifactTarget, Config};
use crate::context::AppContext;
use crate::errors::AppError;
use crate::formula::{Arch, AssetMatrix, FormulaAsset, FormulaSpec, Os, TargetAsset};
use crate::git::{
    commit_paths, create_tag, ensure_clean_repo, ensure_tag_absent, git_clone, push_head, push_tag,
    RemoteContext,
};
use crate::host::github::GitHubClient;
use crate::host::{
    DownloadPolicy, HostClient, Release, DEFAULT_CHECKSUM_MAX_BYTES, DEFAULT_CHECKSUM_MAX_RETRIES,
    DEFAULT_CHECKSUM_RETRY_BASE_DELAY_MS, DEFAULT_CHECKSUM_RETRY_MAX_DELAY_MS,
    DEFAULT_CHECKSUM_TIMEOUT_SECS,
};
use crate::preview::{preview_and_apply, PlannedWrite, RepoPlan};
use crate::repo_detect::ProjectMetadata;
use crate::version::{resolve_version_tag, ResolvedVersionTag};
use crate::version_update::{plan_version_update, VersionUpdateChange};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tempfile::{Builder as TempBuilder, TempDir};

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
    tap_remote_url: Option<String>,
    host_owner: String,
    host_repo: String,
    host_api_base: Option<String>,
    tag_format: Option<String>,
    tarball_url_template: Option<String>,
    commit_message_template: Option<String>,
    install_block: Option<String>,
    template_path: Option<PathBuf>,
    cli_remote_url: Option<String>,
    checksum_max_bytes: u64,
    download_policy: DownloadPolicy,
}

#[derive(Debug)]
struct TapRoot {
    path: Option<PathBuf>,
    temp_dir: Option<TempDir>,
    cloned_from: Option<String>,
    remote_url: Option<String>,
}

#[derive(Debug)]
enum ReleaseDiscovery {
    ReleaseAsset {
        release: Release,
        version: String,
        tag_to_create: Option<String>,
    },
    SourceTarball {
        version: String,
        source_tag: String,
        tag_to_create: Option<String>,
    },
}

impl ReleaseDiscovery {
    fn version(&self) -> &str {
        match self {
            ReleaseDiscovery::ReleaseAsset { version, .. }
            | ReleaseDiscovery::SourceTarball { version, .. } => version,
        }
    }

    fn tag_to_create(&self) -> Option<&str> {
        match self {
            ReleaseDiscovery::ReleaseAsset { tag_to_create, .. }
            | ReleaseDiscovery::SourceTarball { tag_to_create, .. } => {
                tag_to_create.as_deref()
            }
        }
    }
}

pub fn run(ctx: &AppContext, args: &ReleaseArgs) -> Result<(), AppError> {
    if !ctx.config_path.exists() {
        return Err(AppError::MissingConfig(format!(
            "config not found at {}",
            ctx.config_path.display()
        )));
    }

    let tap_root = if args.dry_run {
        let tap_root = resolve_tap_root_for_release(ctx, args, true)?;
        if tap_root.path.is_none()
            && tap_root.remote_url.is_some()
            && !has_absolute_formula_path(&ctx.config)
        {
            return Err(AppError::MissingConfig(
                "dry-run requires tap.path or an absolute tap.formula_path; tap.remote cannot be auto-cloned".to_string(),
            ));
        }
        validate_release_preflight(ctx, args, args.dry_run)?;
        tap_root
    } else {
        validate_release_preflight(ctx, args, args.dry_run)?;
        resolve_tap_root_for_release(ctx, args, false)?
    };
    if let (Some(path), Some(remote)) = (tap_root.path.as_ref(), tap_root.cloned_from.as_ref()) {
        println!(
            "release: cloned tap repo from {} to {}",
            remote,
            path.display()
        );
    }
    let _tap_root_guard = tap_root.temp_dir.as_ref();

    let resolved = resolve_release_context(
        ctx,
        args,
        tap_root.path.as_ref(),
        tap_root.remote_url.as_deref(),
        args.dry_run,
    )?;

    let version_tag = resolve_version_tag(args.version.as_deref(), args.tag.as_deref())?;
    if args.dry_run {
        return run_dry_run_release(ctx, args, &resolved, &version_tag);
    }
    let client =
        GitHubClient::from_env(resolved.host_api_base.as_deref(), resolved.download_policy)?;

    let discovery = match resolved.artifact_strategy.as_str() {
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

            let version = version_tag.version.clone().unwrap_or(release_version);
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

            ReleaseDiscovery::ReleaseAsset {
                release,
                version,
                tag_to_create,
            }
        }
        "source-tarball" => {
            validate_source_tarball_inputs(&resolved)?;
            let (version, source_tag) = resolve_source_tarball_version_and_tag(
                &client,
                &resolved,
                &version_tag,
                args.include_prerelease,
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
            ReleaseDiscovery::SourceTarball {
                version,
                source_tag,
                tag_to_create,
            }
        }
        _ => {
            return Err(AppError::InvalidInput(format!(
            "artifact.strategy '{}' is not supported yet (use 'release-asset' or 'source-tarball')",
            resolved.artifact_strategy
        )))
        }
    };

    let version = discovery.version().to_string();
    let tag_to_create = discovery.tag_to_create().map(|tag| tag.to_string());

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

    if !args.dry_run && needs_cli_repo && ctx.repo.git_root.is_none() {
        return Err(AppError::GitState(
            "cli repo is not a git repository".to_string(),
        ));
    }

    let cli_repo_root = ctx.repo.git_root.as_ref().unwrap_or(&ctx.cwd);

    if let Some(tag_name) = tag_to_create.as_deref() {
        ensure_tag_absent(cli_repo_root, tag_name)?;
    }

    if !args.allow_dirty {
        ensure_clean_repo(&resolved.tap_root, "tap repo")?;
        if needs_cli_repo {
            let cli_root = ctx.repo.git_root.as_ref().ok_or_else(|| {
                AppError::GitState("cli repo is not a git repository".to_string())
            })?;
            ensure_clean_repo(cli_root, "cli repo")?;
        }
    }

    let asset_matrix = match &discovery {
        ReleaseDiscovery::ReleaseAsset { release, .. } => {
            build_assets(&client, release, &resolved, &version, args.dry_run)?
        }
        ReleaseDiscovery::SourceTarball { source_tag, .. } => {
            let tarball = build_tarball_asset(
                &client,
                &resolved,
                &version,
                source_tag,
                resolved.checksum_max_bytes,
                args.dry_run,
            )?;
            AssetMatrix::Universal(tarball)
        }
    };

    let version_update_plan = if version_update_mode != "none" {
        plan_version_update(&ctx.config.version_update, cli_repo_root, &version)?
    } else {
        Vec::new()
    };
    let version_update_paths = planned_paths(&version_update_plan);

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

    let mut plans = Vec::new();
    if !version_update_plan.is_empty() {
        plans.push(RepoPlan {
            label: "cli".to_string(),
            repo_root: cli_repo_root.to_path_buf(),
            writes: planned_writes(&version_update_plan),
        });
    }
    plans.push(RepoPlan {
        label: "tap".to_string(),
        repo_root: resolved.tap_root.clone(),
        writes: vec![PlannedWrite {
            path: resolved.formula_path.clone(),
            content: rendered,
        }],
    });

    let preview = preview_and_apply(&plans, args.dry_run)?;
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

    let formula_changed = preview
        .changed_files
        .iter()
        .any(|path| path == &resolved.formula_path);
    let cli_changed = version_update_paths
        .iter()
        .any(|path| preview.changed_files.iter().any(|changed| changed == path));

    if !cli_changed && !formula_changed {
        println!("release: no changes to commit");
    } else {
        if cli_changed {
            let cli_root = ctx.repo.git_root.as_ref().ok_or_else(|| {
                AppError::GitState("cli repo is not a git repository".to_string())
            })?;
            let message = build_version_update_message(&version);
            commit_version_update(cli_root, &version_update_paths, &message)?;
        }

        if formula_changed {
            commit_formula_update(&resolved.tap_root, &resolved.formula_path, &commit_message)?;
        } else {
            println!("release: no formula changes to commit");
        }
    }

    if let Some(tag_name) = tag_to_create.as_deref() {
        let cli_root =
            ctx.repo.git_root.as_ref().ok_or_else(|| {
                AppError::GitState("cli repo is not a git repository".to_string())
            })?;
        create_tag(cli_root, tag_name)?;
    }

    if !args.no_push {
        if formula_changed {
            push_head(
                &resolved.tap_root,
                resolved.tap_remote_url.as_deref(),
                RemoteContext::Tap,
            )?;
        }
        if cli_changed {
            let cli_root = ctx.repo.git_root.as_ref().ok_or_else(|| {
                AppError::GitState("cli repo is not a git repository".to_string())
            })?;
            push_head(
                cli_root,
                resolved.cli_remote_url.as_deref(),
                RemoteContext::Cli,
            )?;
        }
        if let Some(tag_name) = tag_to_create.as_deref() {
            let cli_root = ctx.repo.git_root.as_ref().ok_or_else(|| {
                AppError::GitState("cli repo is not a git repository".to_string())
            })?;
            push_tag(
                cli_root,
                tag_name,
                resolved.cli_remote_url.as_deref(),
                RemoteContext::Cli,
            )?;
        }
    }

    drop(tap_root);
    Ok(())
}

fn has_absolute_formula_path(config: &Config) -> bool {
    config
        .tap
        .formula_path
        .as_ref()
        .map(|path| path.is_absolute())
        .unwrap_or(false)
}

fn validate_release_preflight(
    ctx: &AppContext,
    args: &ReleaseArgs,
    require_asset_selection: bool,
) -> Result<(), AppError> {
    let config = &ctx.config;
    let metadata = ctx.repo.metadata.as_ref();
    let inferred_owner_repo = infer_owner_repo_from_metadata(metadata);
    let mut missing = Vec::new();

    let tap_path = args.tap_path.clone().or_else(|| config.tap.path.clone());
    let tap_remote = args
        .tap_remote
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .or_else(|| {
            config
                .tap
                .remote
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| value.to_string())
        });

    let has_absolute_formula_path = config
        .tap
        .formula_path
        .as_ref()
        .map(|path| path.is_absolute())
        .unwrap_or(false);
    if tap_path.is_none() && tap_remote.is_none() && !has_absolute_formula_path {
        missing.push("tap.path, tap.remote, or tap.formula_path".to_string());
    }

    let formula_name = resolve_string(None, config.tap.formula.as_ref()).unwrap_or_default();
    if formula_name.is_empty() {
        missing.push("tap.formula".to_string());
    }

    let description = resolve_string(None, config.project.description.as_ref())
        .or_else(|| resolve_string(None, metadata.and_then(|meta| meta.description.as_ref())))
        .unwrap_or_default();
    if description.trim().is_empty() {
        missing.push("project.description".to_string());
    }

    let homepage = resolve_string(None, config.project.homepage.as_ref())
        .or_else(|| resolve_string(None, metadata.and_then(|meta| meta.homepage.as_ref())))
        .unwrap_or_default();
    if homepage.trim().is_empty() {
        missing.push("project.homepage".to_string());
    }

    let license = resolve_string(None, config.project.license.as_ref())
        .or_else(|| resolve_string(None, metadata.and_then(|meta| meta.license.as_ref())))
        .unwrap_or_default();
    if license.trim().is_empty() {
        missing.push("project.license".to_string());
    }

    let bins = {
        let bins = normalize_bins(config.project.bin.clone());
        if bins.is_empty() {
            metadata
                .map(|meta| normalize_bins(meta.bin.clone()))
                .unwrap_or_default()
        } else {
            bins
        }
    };
    if bins.is_empty() {
        missing.push("project.bin".to_string());
    }

    let artifact_strategy = resolve_string(
        args.artifact_strategy.as_ref(),
        config.artifact.strategy.as_ref(),
    )
    .unwrap_or_default();
    if artifact_strategy.trim().is_empty() {
        missing.push("artifact.strategy".to_string());
    }

    let asset_template = resolve_string(
        args.asset_template.as_ref(),
        config.artifact.asset_template.as_ref(),
    );
    let asset_name = resolve_string(
        args.asset_name.as_ref(),
        config.artifact.asset_name.as_ref(),
    );

    let targets = config.artifact.targets.clone();
    let target_mode = if !targets.is_empty() && !artifact_strategy.trim().is_empty() {
        Some(validate_target_keys_shape(&targets)?)
    } else {
        None
    };
    if let Some(mode) = target_mode {
        validate_target_templates(&targets, mode, asset_name.as_ref(), asset_template.as_ref())?;
    }

    match artifact_strategy.as_str() {
        "release-asset" => {
            if require_asset_selection {
                let global_selection = asset_name.is_some() || asset_template.is_some();
                if !global_selection {
                    if targets.is_empty() {
                        missing.push("artifact.asset_name or artifact.asset_template".to_string());
                    } else {
                        let mut missing_targets = targets
                            .iter()
                            .filter_map(|(key, target)| {
                                (!target_has_selection(target)).then(|| key.clone())
                            })
                            .collect::<Vec<_>>();
                        missing_targets.sort();
                        for key in missing_targets {
                            missing.push(format!(
                                "artifact.targets.{key}.asset_name or asset_template"
                            ));
                        }
                    }
                }
            }
        }
        "source-tarball" => {
            if asset_name.is_some() || asset_template.is_some() {
                return Err(AppError::InvalidInput(
                    "source-tarball strategy does not support asset_name or asset_template"
                        .to_string(),
                ));
            }
            if !targets.is_empty() {
                return Err(AppError::InvalidInput(
                    "source-tarball strategy does not support artifact.targets".to_string(),
                ));
            }
        }
        _ => {}
    }

    let host_owner = resolve_string(args.host_owner.as_ref(), config.artifact.owner.as_ref())
        .or_else(|| resolve_string(None, config.cli.owner.as_ref()))
        .or_else(|| inferred_owner_repo.as_ref().map(|(owner, _)| owner.clone()))
        .unwrap_or_default();
    if host_owner.is_empty() {
        missing.push("host-owner".to_string());
    }

    let host_repo = resolve_string(args.host_repo.as_ref(), config.artifact.repo.as_ref())
        .or_else(|| resolve_string(None, config.cli.repo.as_ref()))
        .or_else(|| inferred_owner_repo.as_ref().map(|(_, repo)| repo.clone()))
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

    Ok(())
}

fn run_dry_run_release(
    ctx: &AppContext,
    args: &ReleaseArgs,
    resolved: &ReleaseContext,
    version_tag: &ResolvedVersionTag,
) -> Result<(), AppError> {
    let version = version_tag.version.clone().ok_or_else(|| {
        AppError::MissingConfig("missing required fields for --dry-run: version or tag".to_string())
    })?;

    let release_tag = resolve_tag_name(&version, version_tag, resolved.tag_format.as_deref(), "")?;

    let (version, tag_to_create, asset_matrix) = match resolved.artifact_strategy.as_str() {
        "release-asset" => {
            let asset_matrix = build_assets_dry_run(resolved, &version, &release_tag)?;
            let tag_to_create = if args.skip_tag {
                None
            } else {
                Some(release_tag.clone())
            };
            (version, tag_to_create, asset_matrix)
        }
        "source-tarball" => {
            validate_source_tarball_inputs(resolved)?;
            let source_tag = resolve_source_tarball_tag(
                version_tag,
                resolved.tag_format.as_deref(),
                None,
                &version,
            )?;
            let tarball = build_tarball_asset_dry_run(resolved, &version, &source_tag)?;
            let tag_to_create = if args.skip_tag {
                None
            } else {
                Some(resolve_tag_name(
                    &version,
                    version_tag,
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
    let cli_repo_root = ctx.repo.git_root.as_ref().unwrap_or(&ctx.cwd);

    if !args.allow_dirty {
        ensure_clean_repo(&resolved.tap_root, "tap repo")?;
        if needs_cli_repo {
            let cli_root = ctx.repo.git_root.as_ref().ok_or_else(|| {
                AppError::GitState("cli repo is not a git repository".to_string())
            })?;
            ensure_clean_repo(cli_root, "cli repo")?;
        }
    }

    let version_update_plan = if version_update_mode != "none" {
        plan_version_update(&ctx.config.version_update, cli_repo_root, &version)?
    } else {
        Vec::new()
    };
    let version_update_paths = planned_paths(&version_update_plan);
    if !version_update_paths.is_empty() {
        println!(
            "dry-run: would update version in {}",
            format_paths(&version_update_paths)
        );
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

    let mut plans = Vec::new();
    if !version_update_plan.is_empty() {
        plans.push(RepoPlan {
            label: "cli".to_string(),
            repo_root: cli_repo_root.to_path_buf(),
            writes: planned_writes(&version_update_plan),
        });
    }
    plans.push(RepoPlan {
        label: "tap".to_string(),
        repo_root: resolved.tap_root.clone(),
        writes: vec![PlannedWrite {
            path: resolved.formula_path.clone(),
            content: rendered,
        }],
    });

    let preview = preview_and_apply(&plans, true)?;
    print_preview(&preview);
    Ok(())
}

fn resolve_release_context(
    ctx: &AppContext,
    args: &ReleaseArgs,
    tap_path_override: Option<&PathBuf>,
    tap_remote_override: Option<&str>,
    require_asset_selection: bool,
) -> Result<ReleaseContext, AppError> {
    let config = &ctx.config;
    let metadata = ctx.repo.metadata.as_ref();
    let inferred_owner_repo = infer_owner_repo_from_metadata(metadata);
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
            missing.push("tap.path, tap.remote, or tap.formula_path".to_string());
            PathBuf::new()
        }
    };

    let tap_root = tap_path
        .clone()
        .or_else(|| tap_root_from_formula_path(&formula_path))
        .unwrap_or_else(|| ctx.cwd.clone());

    let tap_remote_url = tap_remote_override
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .or_else(|| {
            config
                .tap
                .remote
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| value.to_string())
        });

    let description = resolve_string(None, config.project.description.as_ref())
        .or_else(|| resolve_string(None, metadata.and_then(|meta| meta.description.as_ref())))
        .unwrap_or_default();
    if description.trim().is_empty() {
        missing.push("project.description".to_string());
    }

    let homepage = resolve_string(None, config.project.homepage.as_ref())
        .or_else(|| resolve_string(None, metadata.and_then(|meta| meta.homepage.as_ref())))
        .unwrap_or_default();
    if homepage.trim().is_empty() {
        missing.push("project.homepage".to_string());
    }

    let license = resolve_string(None, config.project.license.as_ref())
        .or_else(|| resolve_string(None, metadata.and_then(|meta| meta.license.as_ref())))
        .unwrap_or_default();
    if license.trim().is_empty() {
        missing.push("project.license".to_string());
    }

    let bins = {
        let bins = normalize_bins(config.project.bin.clone());
        if bins.is_empty() {
            metadata
                .map(|meta| normalize_bins(meta.bin.clone()))
                .unwrap_or_default()
        } else {
            bins
        }
    };
    if bins.is_empty() {
        missing.push("project.bin".to_string());
    }

    let project_name = resolve_string(None, config.project.name.as_ref())
        .or_else(|| resolve_string(None, metadata.and_then(|meta| meta.name.as_ref())))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| formula_name.clone());

    let checksum_max_bytes = resolve_checksum_max_bytes(&config.artifact);
    let download_policy = resolve_download_policy(&config.artifact);

    let artifact_strategy = resolve_string(
        args.artifact_strategy.as_ref(),
        config.artifact.strategy.as_ref(),
    )
    .unwrap_or_default();
    if artifact_strategy.trim().is_empty() {
        missing.push("artifact.strategy".to_string());
    }

    let asset_template = resolve_string(
        args.asset_template.as_ref(),
        config.artifact.asset_template.as_ref(),
    );
    let asset_name = resolve_string(
        args.asset_name.as_ref(),
        config.artifact.asset_name.as_ref(),
    );

    let targets = config.artifact.targets.clone();
    let target_mode = if !targets.is_empty() && !artifact_strategy.trim().is_empty() {
        Some(validate_target_keys_shape(&targets)?)
    } else {
        None
    };
    if let Some(mode) = target_mode {
        validate_target_templates(&targets, mode, asset_name.as_ref(), asset_template.as_ref())?;
    }

    match artifact_strategy.as_str() {
        "release-asset" => {
            if require_asset_selection {
                let global_selection = asset_name.is_some() || asset_template.is_some();
                if !global_selection {
                    if targets.is_empty() {
                        missing.push("artifact.asset_name or artifact.asset_template".to_string());
                    } else {
                        let mut missing_targets = targets
                            .iter()
                            .filter_map(|(key, target)| {
                                (!target_has_selection(target)).then(|| key.clone())
                            })
                            .collect::<Vec<_>>();
                        missing_targets.sort();
                        for key in missing_targets {
                            missing.push(format!(
                                "artifact.targets.{key}.asset_name or asset_template"
                            ));
                        }
                    }
                }
            }
        }
        "source-tarball" => {
            if asset_name.is_some() || asset_template.is_some() {
                return Err(AppError::InvalidInput(
                    "source-tarball strategy does not support asset_name or asset_template"
                        .to_string(),
                ));
            }
            if !targets.is_empty() {
                return Err(AppError::InvalidInput(
                    "source-tarball strategy does not support artifact.targets".to_string(),
                ));
            }
        }
        _ => {}
    }

    let host_owner = resolve_string(args.host_owner.as_ref(), config.artifact.owner.as_ref())
        .or_else(|| resolve_string(None, config.cli.owner.as_ref()))
        .or_else(|| inferred_owner_repo.as_ref().map(|(owner, _)| owner.clone()))
        .unwrap_or_default();
    if host_owner.is_empty() {
        missing.push("host-owner".to_string());
    }

    let host_repo = resolve_string(args.host_repo.as_ref(), config.artifact.repo.as_ref())
        .or_else(|| resolve_string(None, config.cli.repo.as_ref()))
        .or_else(|| inferred_owner_repo.as_ref().map(|(_, repo)| repo.clone()))
        .unwrap_or_default();
    if host_repo.is_empty() {
        missing.push("host-repo".to_string());
    }

    let cli_remote_url = config
        .cli
        .remote
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());

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
        targets,
        tap_remote_url,
        host_owner,
        host_repo,
        host_api_base: config.host.api_base.clone(),
        tag_format: config.release.tag_format.clone(),
        tarball_url_template: config.release.tarball_url_template.clone(),
        commit_message_template: config.release.commit_message_template.clone(),
        install_block: config.template.install_block.clone(),
        template_path: config.template.path.clone(),
        cli_remote_url,
        checksum_max_bytes,
        download_policy,
    })
}

fn resolve_tap_root_for_release(
    ctx: &AppContext,
    args: &ReleaseArgs,
    dry_run: bool,
) -> Result<TapRoot, AppError> {
    let remote_url = args
        .tap_remote
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .or_else(|| {
            ctx.config
                .tap
                .remote
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| value.to_string())
        });

    let tap_path = args
        .tap_path
        .clone()
        .or_else(|| ctx.config.tap.path.clone());
    if tap_path.is_some() {
        return Ok(TapRoot {
            path: tap_path,
            temp_dir: None,
            cloned_from: None,
            remote_url,
        });
    }

    if let Some(formula_path) = ctx.config.tap.formula_path.as_ref() {
        if formula_path.is_absolute() {
            return Ok(TapRoot {
                path: None,
                temp_dir: None,
                cloned_from: None,
                remote_url,
            });
        }
    }

    if let Some(remote) = remote_url.as_deref() {
        if dry_run {
            return Ok(TapRoot {
                path: None,
                temp_dir: None,
                cloned_from: None,
                remote_url,
            });
        }
        let temp_dir = TempBuilder::new().prefix("brewdistillery-tap-").tempdir()?;
        let dest = temp_dir.path().join("tap");
        git_clone(remote, &dest)?;
        return Ok(TapRoot {
            path: Some(dest),
            temp_dir: Some(temp_dir),
            cloned_from: Some(remote.to_string()),
            remote_url,
        });
    }

    Ok(TapRoot {
        path: None,
        temp_dir: None,
        cloned_from: None,
        remote_url,
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

fn infer_owner_repo_from_metadata(
    metadata: Option<&ProjectMetadata>,
) -> Option<(String, String)> {
    metadata
        .and_then(|meta| meta.homepage.as_deref())
        .and_then(parse_owner_repo_from_github)
}

fn parse_owner_repo_from_github(input: &str) -> Option<(String, String)> {
    let trimmed = input.trim();
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
    let mut parts = cleaned.split('/').filter(|part| !part.is_empty());
    let owner = parts.next()?.to_string();
    let repo = parts.next()?.to_string();
    if owner.is_empty() || repo.is_empty() {
        None
    } else {
        Some((owner, repo))
    }
}

fn resolve_checksum_max_bytes(config: &ArtifactConfig) -> u64 {
    config
        .checksum_max_bytes
        .unwrap_or(DEFAULT_CHECKSUM_MAX_BYTES)
        .max(1)
}

fn resolve_download_policy(config: &ArtifactConfig) -> DownloadPolicy {
    let timeout_secs = config
        .checksum_timeout_secs
        .unwrap_or(DEFAULT_CHECKSUM_TIMEOUT_SECS)
        .max(1);

    let max_retries = config
        .checksum_max_retries
        .map(|value| value as usize)
        .unwrap_or(DEFAULT_CHECKSUM_MAX_RETRIES)
        .max(1);

    let base_delay = config
        .checksum_retry_base_delay_ms
        .unwrap_or(DEFAULT_CHECKSUM_RETRY_BASE_DELAY_MS)
        .max(1);
    let max_delay = config
        .checksum_retry_max_delay_ms
        .unwrap_or(DEFAULT_CHECKSUM_RETRY_MAX_DELAY_MS)
        .max(base_delay);

    DownloadPolicy {
        timeout_secs,
        max_retries,
        retry_base_delay_ms: base_delay,
        retry_max_delay_ms: max_delay,
    }
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
        AppError::InvalidInput(format!("GitHub release tag '{}' is not valid semver", tag))
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
    let default = format!("feature: Updated formula for version '{version}'");
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

fn planned_paths(changes: &[VersionUpdateChange]) -> Vec<PathBuf> {
    changes.iter().map(|change| change.path.clone()).collect()
}

fn planned_writes(changes: &[VersionUpdateChange]) -> Vec<PlannedWrite> {
    changes
        .iter()
        .map(|change| PlannedWrite {
            path: change.path.clone(),
            content: change.content.clone(),
        })
        .collect()
}

fn commit_formula_update(repo: &Path, formula_path: &Path, message: &str) -> Result<(), AppError> {
    commit_paths(repo, &[formula_path], message)
}

fn commit_version_update(repo: &Path, files: &[PathBuf], message: &str) -> Result<(), AppError> {
    let paths = files.iter().map(PathBuf::as_path).collect::<Vec<_>>();
    commit_paths(repo, &paths, message)
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

fn build_assets_dry_run(
    resolved: &ReleaseContext,
    version: &str,
    tag: &str,
) -> Result<AssetMatrix, AppError> {
    if resolved.targets.is_empty() {
        let asset =
            build_release_asset_dry_run(resolved, None, version, tag, "universal", None, None)?;
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
                let asset = build_release_asset_dry_run(
                    resolved,
                    Some(target),
                    version,
                    tag,
                    key,
                    Some(os),
                    None,
                )?;
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
                let asset = build_release_asset_dry_run(
                    resolved,
                    Some(target),
                    version,
                    tag,
                    key,
                    Some(os),
                    Some(arch),
                )?;
                targets.push(TargetAsset { os, arch, asset });
            }
        }
    }

    match mode {
        Some(TargetMode::PerOs) => Ok(AssetMatrix::PerOs { macos, linux }),
        Some(TargetMode::PerTarget) => Ok(AssetMatrix::PerTarget(targets)),
        None => Ok(AssetMatrix::Universal(build_release_asset_dry_run(
            resolved,
            None,
            version,
            tag,
            "universal",
            None,
            None,
        )?)),
    }
}

fn build_release_asset_dry_run(
    resolved: &ReleaseContext,
    target: Option<&ArtifactTarget>,
    version: &str,
    tag: &str,
    target_key: &str,
    os: Option<Os>,
    arch: Option<Arch>,
) -> Result<FormulaAsset, AppError> {
    let (asset_name, asset_template) = resolve_target_selection(resolved, target);
    let name = if let Some(name) = asset_name {
        name
    } else if let Some(template) = asset_template {
        render_asset_template_dry_run(&template, &resolved.project_name, version, os, arch)?
    } else {
        return Err(AppError::MissingConfig(format!(
            "missing required fields for --dry-run: asset-name or asset-template for target '{target_key}'"
        )));
    };

    let url = release_asset_url(&resolved.host_owner, &resolved.host_repo, tag, &name)?;
    println!("dry-run: would download {}", url);
    Ok(FormulaAsset {
        url,
        sha256: "DRY-RUN".to_string(),
    })
}

fn resolve_target_selection(
    resolved: &ReleaseContext,
    target: Option<&ArtifactTarget>,
) -> (Option<String>, Option<String>) {
    let asset_name = target
        .and_then(|value| selection_value(value.asset_name.as_ref()))
        .or_else(|| selection_value(resolved.asset_name.as_ref()));
    let asset_template = target
        .and_then(|value| selection_value(value.asset_template.as_ref()))
        .or_else(|| selection_value(resolved.asset_template.as_ref()));
    (asset_name, asset_template)
}

fn selection_value(value: Option<&String>) -> Option<String> {
    value
        .map(|entry| entry.trim())
        .filter(|entry| !entry.is_empty())
        .map(|entry| entry.to_string())
}

fn render_asset_template_dry_run(
    template: &str,
    project_name: &str,
    version: &str,
    os: Option<Os>,
    arch: Option<Arch>,
) -> Result<String, AppError> {
    let mut output = template.to_string();

    if output.contains("{name}") {
        if project_name.trim().is_empty() {
            return Err(AppError::InvalidInput(
                "asset_template requires {name}".to_string(),
            ));
        }
        output = output.replace("{name}", project_name.trim());
    }

    if output.contains("{version}") {
        if version.trim().is_empty() {
            return Err(AppError::InvalidInput(
                "asset_template requires {version}".to_string(),
            ));
        }
        output = output.replace("{version}", version.trim());
    }

    if output.contains("{os}") {
        let os =
            os.ok_or_else(|| AppError::InvalidInput("asset_template requires {os}".to_string()))?;
        output = output.replace("{os}", os_token(os));
    }

    if output.contains("{arch}") {
        let arch = arch
            .ok_or_else(|| AppError::InvalidInput("asset_template requires {arch}".to_string()))?;
        output = output.replace("{arch}", arch_token(arch));
    }

    let trimmed = output.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidInput(
            "asset_template rendered an empty asset name".to_string(),
        ));
    }
    Ok(trimmed.to_string())
}

fn os_token(os: Os) -> &'static str {
    match os {
        Os::Darwin => "darwin",
        Os::Linux => "linux",
    }
}

fn arch_token(arch: Arch) -> &'static str {
    match arch {
        Arch::Arm64 => "arm64",
        Arch::Amd64 => "amd64",
    }
}

fn release_asset_url(
    owner: &str,
    repo: &str,
    tag: &str,
    asset_name: &str,
) -> Result<String, AppError> {
    let owner = owner.trim();
    let repo = repo.trim();
    let tag = tag.trim();
    if owner.is_empty() || repo.is_empty() {
        return Err(AppError::InvalidInput(
            "release asset URL requires host owner and repo".to_string(),
        ));
    }
    if tag.is_empty() {
        return Err(AppError::InvalidInput(
            "release asset URL requires a non-empty tag".to_string(),
        ));
    }
    if asset_name.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "release asset URL requires a non-empty asset name".to_string(),
        ));
    }

    Ok(format!(
        "https://github.com/{owner}/{repo}/releases/download/{tag}/{}",
        asset_name.trim()
    ))
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
            target_key: Some("universal".to_string()),
        };
        let asset = select_asset_for_release(
            client,
            release,
            selection,
            resolved.checksum_max_bytes,
            dry_run,
        )?;
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
                    target_key: Some(key.clone()),
                };
                let asset = select_asset_for_release(
                    client,
                    release,
                    selection,
                    resolved.checksum_max_bytes,
                    dry_run,
                )?;
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
                    target_key: Some(key.clone()),
                };
                let asset = select_asset_for_release(
                    client,
                    release,
                    selection,
                    resolved.checksum_max_bytes,
                    dry_run,
                )?;
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
                target_key: Some("universal".to_string()),
            },
            resolved.checksum_max_bytes,
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
    max_bytes: u64,
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
        client.download_sha256(&url, Some(max_bytes))?
    };

    Ok(FormulaAsset { url, sha256 })
}

fn build_tarball_asset_dry_run(
    resolved: &ReleaseContext,
    version: &str,
    tag: &str,
) -> Result<FormulaAsset, AppError> {
    let url = render_tarball_url(
        resolved.tarball_url_template.as_deref(),
        &resolved.host_owner,
        &resolved.host_repo,
        tag,
        version,
    )?;
    println!("dry-run: would download {}", url);
    Ok(FormulaAsset {
        url,
        sha256: "DRY-RUN".to_string(),
    })
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
    max_bytes: u64,
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
            AppError::InvalidInput(format!("release asset '{}' missing download URL", name))
        })?;

    let sha256 = if dry_run {
        println!("dry-run: would download {}", asset.download_url);
        "DRY-RUN".to_string()
    } else {
        client.download_sha256(&asset.download_url, Some(max_bytes))?
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

fn target_has_selection(target: &ArtifactTarget) -> bool {
    target
        .asset_name
        .as_deref()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
        || target
            .asset_template
            .as_deref()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
}

fn validate_target_keys_shape(
    targets: &BTreeMap<String, ArtifactTarget>,
) -> Result<TargetMode, AppError> {
    let mut mode: Option<TargetMode> = None;
    for key in targets.keys() {
        match parse_target_key(key)? {
            TargetKey::Os(_) => {
                if matches!(mode, Some(TargetMode::PerTarget)) {
                    return Err(AppError::InvalidInput(
                        "target keys must be all <os> or all <os>-<arch>".to_string(),
                    ));
                }
                mode = Some(TargetMode::PerOs);
            }
            TargetKey::OsArch(_, _) => {
                if matches!(mode, Some(TargetMode::PerOs)) {
                    return Err(AppError::InvalidInput(
                        "target keys must be all <os> or all <os>-<arch>".to_string(),
                    ));
                }
                mode = Some(TargetMode::PerTarget);
            }
        }
    }
    debug_assert!(!targets.is_empty());
    Ok(mode.unwrap_or(TargetMode::PerOs))
}

fn validate_target_templates(
    targets: &BTreeMap<String, ArtifactTarget>,
    mode: TargetMode,
    global_asset_name: Option<&String>,
    global_asset_template: Option<&String>,
) -> Result<(), AppError> {
    if !matches!(mode, TargetMode::PerOs) {
        return Ok(());
    }

    let global_asset_name = selection_value(global_asset_name);
    let global_asset_template = selection_value(global_asset_template);

    for (key, target) in targets {
        let asset_name =
            selection_value(target.asset_name.as_ref()).or_else(|| global_asset_name.clone());
        if asset_name.is_some() {
            continue;
        }

        let template = selection_value(target.asset_template.as_ref())
            .or_else(|| global_asset_template.clone());
        if let Some(template) = template {
            if template.contains("{arch}") {
                return Err(AppError::InvalidInput(format!(
                    "per-OS target '{key}' cannot use {{arch}} in asset_template; use <os>-<arch> targets instead"
                )));
            }
        }
    }

    Ok(())
}

fn parse_target_key(key: &str) -> Result<TargetKey, AppError> {
    let lower = key.trim().to_ascii_lowercase();
    if lower.is_empty() {
        return Err(invalid_target_key(key));
    }

    let tokens = if lower.contains('-') {
        let mut parts = lower.splitn(3, '-');
        let os = parts.next().unwrap_or("");
        let arch = parts.next();
        let extra = parts.next();
        if extra.is_some() {
            return Err(invalid_target_key(key));
        }
        match arch {
            Some(arch_token) if !arch_token.trim().is_empty() => vec![os, arch_token],
            _ => vec![os],
        }
    } else if lower.contains('_') {
        let mut parts = lower.splitn(2, '_');
        let os = parts.next().unwrap_or("");
        let arch = parts.next();
        match arch {
            Some(arch_token) if !arch_token.trim().is_empty() => vec![os, arch_token],
            _ => vec![os],
        }
    } else {
        vec![lower.as_str()]
    };

    match tokens.as_slice() {
        [os_token] => {
            let os = os_from_token(os_token).ok_or_else(|| invalid_target_key(key))?;
            Ok(TargetKey::Os(os))
        }
        [os_token, arch_token] => {
            let os = os_from_token(os_token).ok_or_else(|| invalid_target_key(key))?;
            let arch = arch_from_token(arch_token).ok_or_else(|| invalid_target_key(key))?;
            Ok(TargetKey::OsArch(os, arch))
        }
        _ => Err(invalid_target_key(key)),
    }
}

fn invalid_target_key(key: &str) -> AppError {
    AppError::InvalidInput(format!(
        "invalid target key '{key}': expected <os> or <os>-<arch>"
    ))
}

fn os_from_token(token: &str) -> Option<Os> {
    match token {
        "darwin" | "macos" | "osx" => Some(Os::Darwin),
        "linux" => Some(Os::Linux),
        _ => None,
    }
}

fn arch_from_token(token: &str) -> Option<Arch> {
    match token {
        "arm64" | "aarch64" => Some(Arch::Arm64),
        "amd64" | "x86_64" | "x64" => Some(Arch::Amd64),
        _ => None,
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
    use crate::cli::ReleaseArgs;
    use crate::config::{ArtifactTarget, Config};
    use crate::context::AppContext;
    use crate::git::run_git;
    use crate::repo_detect::{ProjectMetadata, RepoInfo};
    use std::collections::BTreeMap;
    use std::fs;
    use std::io::{ErrorKind, Read, Write};
    use std::net::TcpListener;
    use std::path::Path;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc,
        Arc,
    };
    use std::thread;
    use std::time::Duration;
    use tempfile::tempdir;

    fn base_release_args() -> ReleaseArgs {
        ReleaseArgs {
            version: None,
            tag: None,
            skip_tag: false,
            no_push: true,
            tap_path: None,
            tap_remote: None,
            artifact_strategy: None,
            asset_template: None,
            asset_name: None,
            include_prerelease: false,
            force: false,
            host_owner: None,
            host_repo: None,
            dry_run: true,
            allow_dirty: true,
        }
    }

    fn base_config(tap_path: &Path) -> Config {
        let mut config = Config::default();
        config.tap.formula = Some("brewtool".to_string());
        config.tap.path = Some(tap_path.to_path_buf());
        config.project.name = Some("brewtool".to_string());
        config.project.description = Some("Brew tool".to_string());
        config.project.homepage = Some("https://example.com/brewtool".to_string());
        config.project.license = Some("MIT".to_string());
        config.project.bin = vec!["brewtool".to_string()];
        config.cli.owner = Some("acme".to_string());
        config.cli.repo = Some("brewtool".to_string());
        config.artifact.strategy = Some("release-asset".to_string());
        config
    }

    fn base_context(config: Config, cwd: &Path) -> AppContext {
        let config_path = cwd.join(".distill/config.toml");
        config.save(&config_path).unwrap();
        AppContext {
            cwd: cwd.to_path_buf(),
            config_path,
            config,
            repo: RepoInfo::default(),
            verbose: 0,
        }
    }

    fn base_context_with_metadata(
        config: Config,
        cwd: &Path,
        metadata: ProjectMetadata,
    ) -> AppContext {
        let config_path = cwd.join(".distill/config.toml");
        config.save(&config_path).unwrap();
        AppContext {
            cwd: cwd.to_path_buf(),
            config_path,
            config,
            repo: RepoInfo {
                metadata: Some(metadata),
                ..RepoInfo::default()
            },
            verbose: 0,
        }
    }

    fn spawn_tarball_server(body: Vec<u8>) -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let addr = listener.local_addr().expect("local addr");
        listener
            .set_nonblocking(true)
            .expect("set nonblocking listener");

        let handle = thread::spawn(move || {
            for _ in 0..500 {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let mut buffer = [0u8; 1024];
                        let _ = stream.read(&mut buffer);

                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            body.len()
                        );
                        let _ = stream.write_all(response.as_bytes());
                        let _ = stream.write_all(&body);
                        let _ = stream.flush();
                        return;
                    }
                    Err(err) if err.kind() == ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => return,
                }
            }
        });

        (format!("http://{}", addr), handle)
    }

    fn init_git_repo(path: &Path) {
        fs::create_dir_all(path).expect("create repo dir");
        run_git(path, &["init"]).expect("git init");
        run_git(path, &["config", "commit.gpgsign", "false"]).expect("git commit.gpgsign");
        run_git(path, &["config", "tag.gpgsign", "false"]).expect("git tag.gpgsign");
        run_git(path, &["config", "user.email", "test@example.com"]).expect("git email");
        run_git(path, &["config", "user.name", "Test User"]).expect("git name");
    }

    fn commit_all(path: &Path, message: &str) {
        run_git(path, &["add", "."]).expect("git add");
        run_git(path, &["commit", "-m", message]).expect("git commit");
    }

    fn spawn_github_release_server(
        owner: &str,
        repo: &str,
        tag: &str,
        asset_name: &str,
        asset_body: Vec<u8>,
    ) -> (String, mpsc::Sender<()>, thread::JoinHandle<()>, Arc<AtomicUsize>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind GitHub test server");
        let addr = listener.local_addr().expect("local addr");
        listener
            .set_nonblocking(true)
            .expect("set nonblocking listener");

        let base_url = format!("http://{}", addr);
        let asset_path = format!("/assets/{asset_name}");
        let release_path = format!("/repos/{owner}/{repo}/releases/latest");
        let asset_url = format!("{base_url}{asset_path}");
        let asset_len = asset_body.len();
        let release_body = format!(
            "{{\"tag_name\":\"{tag}\",\"draft\":false,\"prerelease\":false,\"assets\":[{{\"name\":\"{asset_name}\",\"browser_download_url\":\"{asset_url}\",\"size\":{asset_len}}}]}}"
        )
        .into_bytes();

        let asset_hits = Arc::new(AtomicUsize::new(0));
        let asset_hits_thread = Arc::clone(&asset_hits);
        let (stop_tx, stop_rx) = mpsc::channel::<()>();
        let handle = thread::spawn(move || {
            for _ in 0..400 {
                if stop_rx.try_recv().is_ok() {
                    return;
                }
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let mut buffer = [0u8; 4096];
                        let read = stream.read(&mut buffer).unwrap_or(0);
                        let request = String::from_utf8_lossy(&buffer[..read]);
                        let path = request
                            .lines()
                            .next()
                            .and_then(|line| line.split_whitespace().nth(1))
                            .unwrap_or("/");

                        let (status, body, content_type) = if path == release_path {
                            ("200 OK", release_body.as_slice(), "application/json")
                        } else if path == asset_path {
                            asset_hits_thread.fetch_add(1, Ordering::SeqCst);
                            ("200 OK", asset_body.as_slice(), "application/octet-stream")
                        } else {
                            ("404 Not Found", b"not found".as_slice(), "text/plain")
                        };

                        let response = format!(
                            "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\nContent-Type: {content_type}\r\n\r\n",
                            body.len()
                        );
                        let _ = stream.write_all(response.as_bytes());
                        let _ = stream.write_all(body);
                        let _ = stream.flush();
                    }
                    Err(err) if err.kind() == ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(_) => return,
                }
            }
        });

        (base_url, stop_tx, handle, asset_hits)
    }

    #[test]
    fn checksum_policy_defaults_apply_when_not_configured() {
        let dir = tempdir().unwrap();
        let tap_path = dir.path().join("homebrew-brewtool");
        let mut config = base_config(&tap_path);
        config.artifact.asset_template = Some("brewtool-{version}.tar.gz".to_string());
        let ctx = base_context(config, dir.path());
        let args = base_release_args();

        let resolved =
            resolve_release_context(&ctx, &args, Some(&tap_path), None, args.dry_run).unwrap();
        assert_eq!(resolved.checksum_max_bytes, DEFAULT_CHECKSUM_MAX_BYTES);
        assert_eq!(resolved.download_policy, DownloadPolicy::default());
    }

    #[test]
    fn checksum_policy_respects_config_overrides() {
        let dir = tempdir().unwrap();
        let tap_path = dir.path().join("homebrew-brewtool");
        let mut config = base_config(&tap_path);
        config.artifact.asset_template = Some("brewtool-{version}.tar.gz".to_string());
        config.artifact.checksum_max_bytes = Some(10 * 1024 * 1024);
        config.artifact.checksum_timeout_secs = Some(15);
        config.artifact.checksum_max_retries = Some(5);
        config.artifact.checksum_retry_base_delay_ms = Some(400);
        config.artifact.checksum_retry_max_delay_ms = Some(1600);

        let ctx = base_context(config, dir.path());
        let args = base_release_args();

        let resolved =
            resolve_release_context(&ctx, &args, Some(&tap_path), None, args.dry_run).unwrap();
        assert_eq!(resolved.checksum_max_bytes, 10 * 1024 * 1024);
        assert_eq!(
            resolved.download_policy,
            DownloadPolicy {
                timeout_secs: 15,
                max_retries: 5,
                retry_base_delay_ms: 400,
                retry_max_delay_ms: 1600,
            }
        );
    }

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
    fn dry_run_does_not_clone_remote() {
        let dir = tempdir().unwrap();
        let mut config = Config::default();
        config.tap.remote = Some("ssh://invalid-remote".to_string());
        let ctx = base_context(config, dir.path());
        let args = base_release_args();

        let tap_root = resolve_tap_root_for_release(&ctx, &args, true).unwrap();
        assert!(tap_root.path.is_none());
        assert_eq!(tap_root.remote_url.as_deref(), Some("ssh://invalid-remote"));
    }

    #[test]
    fn missing_fields_fail_before_remote_clone() {
        let dir = tempdir().unwrap();
        let mut config = Config::default();
        config.tap.formula = Some("brewtool".to_string());
        config.tap.remote = Some("ssh://invalid-remote".to_string());

        let ctx = base_context(config, dir.path());
        let mut args = base_release_args();
        args.dry_run = false;

        let err = run(&ctx, &args).unwrap_err();
        assert!(matches!(err, AppError::MissingConfig(_)));
        assert!(err.to_string().contains("project.description"));
    }

    #[test]
    fn preflight_uses_repo_metadata_fallbacks() {
        let dir = tempdir().unwrap();
        let tap_path = dir.path().join("homebrew-brewtool");
        let mut config = Config::default();
        config.tap.formula = Some("brewtool".to_string());
        config.tap.path = Some(tap_path);
        config.artifact.strategy = Some("release-asset".to_string());

        let mut metadata = ProjectMetadata::default();
        metadata.name = Some("brewtool".to_string());
        metadata.description = Some("Brew tool".to_string());
        metadata.homepage = Some("https://github.com/acme/brewtool".to_string());
        metadata.license = Some("MIT".to_string());
        metadata.bin = vec!["brewtool".to_string()];

        let ctx = base_context_with_metadata(config, dir.path(), metadata);
        let mut args = base_release_args();
        args.dry_run = false;

        validate_release_preflight(&ctx, &args, false).unwrap();
    }

    #[test]
    fn resolve_release_context_uses_repo_metadata_fallbacks() {
        let dir = tempdir().unwrap();
        let tap_path = dir.path().join("homebrew-brewtool");
        let mut config = Config::default();
        config.tap.formula = Some("brewtool".to_string());
        config.tap.path = Some(tap_path.clone());
        config.artifact.strategy = Some("release-asset".to_string());

        let mut metadata = ProjectMetadata::default();
        metadata.name = Some("brewtool".to_string());
        metadata.description = Some("Brew tool".to_string());
        metadata.homepage = Some("https://github.com/acme/brewtool".to_string());
        metadata.license = Some("MIT".to_string());
        metadata.bin = vec!["brewtool".to_string()];

        let ctx = base_context_with_metadata(config, dir.path(), metadata);
        let mut args = base_release_args();
        args.dry_run = false;

        let resolved =
            resolve_release_context(&ctx, &args, Some(&tap_path), None, args.dry_run).unwrap();
        assert_eq!(resolved.description, "Brew tool");
        assert_eq!(resolved.homepage, "https://github.com/acme/brewtool");
        assert_eq!(resolved.license, "MIT");
        assert_eq!(resolved.bins, vec!["brewtool".to_string()]);
        assert_eq!(resolved.project_name, "brewtool");
        assert_eq!(resolved.host_owner, "acme");
        assert_eq!(resolved.host_repo, "brewtool");
    }

    #[test]
    fn dry_run_requires_local_tap_path_when_remote_only() {
        let dir = tempdir().unwrap();
        let tap_path = dir.path().join("homebrew-brewtool");
        let mut config = base_config(&tap_path);
        config.tap.path = None;
        config.tap.remote = Some("ssh://invalid-remote".to_string());
        let ctx = base_context(config, dir.path());
        let args = base_release_args();

        let err = run(&ctx, &args).unwrap_err();
        assert!(matches!(err, AppError::MissingConfig(_)));
        assert_eq!(
            err.to_string(),
            "dry-run requires tap.path or an absolute tap.formula_path; tap.remote cannot be auto-cloned"
        );
    }

    #[test]
    fn dry_run_requires_version_or_tag() {
        let dir = tempdir().unwrap();
        let tap_path = dir.path().join("homebrew-brewtool");
        let mut config = base_config(&tap_path);
        config.artifact.asset_template = Some("brewtool-{version}.tar.gz".to_string());
        let ctx = base_context(config, dir.path());
        let mut args = base_release_args();
        args.tap_path = Some(tap_path.clone());

        let resolved =
            resolve_release_context(&ctx, &args, Some(&tap_path), None, args.dry_run).unwrap();
        let version_tag = resolve_version_tag(None, None).unwrap();

        let err = run_dry_run_release(&ctx, &args, &resolved, &version_tag).unwrap_err();
        assert!(matches!(err, AppError::MissingConfig(_)));
        assert_eq!(
            err.to_string(),
            "missing required fields for --dry-run: version or tag"
        );
    }

    #[test]
    fn release_requires_asset_selection_in_non_interactive_mode() {
        let dir = tempdir().unwrap();
        let tap_path = dir.path().join("homebrew-brewtool");
        let config = base_config(&tap_path);
        let ctx = base_context(config, dir.path());
        let args = base_release_args();

        let err =
            resolve_release_context(&ctx, &args, Some(&tap_path), None, args.dry_run).unwrap_err();
        assert!(matches!(err, AppError::MissingConfig(_)));
        let message = err.to_string();
        assert!(message.contains("artifact.asset_name or artifact.asset_template"));
    }

    #[test]
    fn non_dry_run_allows_asset_auto_selection() {
        let dir = tempdir().unwrap();
        let tap_path = dir.path().join("homebrew-brewtool");
        let config = base_config(&tap_path);
        let ctx = base_context(config, dir.path());
        let mut args = base_release_args();
        args.dry_run = false;

        let resolved =
            resolve_release_context(&ctx, &args, Some(&tap_path), None, args.dry_run).unwrap();
        assert!(resolved.asset_name.is_none());
        assert!(resolved.asset_template.is_none());
    }

    #[test]
    fn tag_exists_fails_before_formula_writes() {
        let cli_dir = tempdir().unwrap();
        let tap_dir = tempdir().unwrap();
        init_git_repo(cli_dir.path());
        init_git_repo(tap_dir.path());

        let formula_path = tap_dir.path().join("Formula").join("brewtool.rb");
        fs::create_dir_all(formula_path.parent().expect("formula dir")).unwrap();
        let initial_formula = r#"class Brewtool < Formula
  desc "Brew tool"
  homepage "https://example.com/brewtool"
  url "https://example.com/brewtool-1.2.2.tar.gz"
  sha256 "0000000000000000000000000000000000000000000000000000000000000000"
  license "MIT"
  version "1.2.2"

  def install
    bin.install "brewtool"
  end
end
"#;
        fs::write(&formula_path, initial_formula).unwrap();
        commit_all(tap_dir.path(), "init tap");

        fs::write(cli_dir.path().join("README.md"), "cli\n").unwrap();
        commit_all(cli_dir.path(), "init cli");

        let asset_name = "brewtool-1.2.3-darwin-arm64.tar.gz";
        let (api_base, stop_tx, handle, asset_hits) = spawn_github_release_server(
            "acme",
            "brewtool",
            "v1.2.3",
            asset_name,
            b"brewtool-asset".to_vec(),
        );

        let mut config = base_config(tap_dir.path());
        config.host.api_base = Some(api_base);

        let mut ctx = base_context(config, cli_dir.path());
        ctx.repo.git_root = Some(cli_dir.path().to_path_buf());
        run_git(cli_dir.path(), &["tag", "v1.2.3"]).unwrap();

        let before = fs::read_to_string(&formula_path).unwrap();

        let mut args = base_release_args();
        args.dry_run = false;
        args.tap_path = Some(tap_dir.path().to_path_buf());

        let err = run(&ctx, &args).unwrap_err();
        let _ = stop_tx.send(());
        let _ = handle.join();

        assert!(matches!(err, AppError::GitState(_)));
        assert_eq!(
            err.to_string(),
            "tag 'v1.2.3' already exists; re-run with --skip-tag or choose a new version"
        );
        assert_eq!(
            asset_hits.load(Ordering::SeqCst),
            0,
            "asset download should not run when tag preflight fails"
        );
        let after = fs::read_to_string(&formula_path).unwrap();
        assert_eq!(before, after);
    }

    #[test]
    fn release_missing_tap_path_message_mentions_remote() {
        let dir = tempdir().unwrap();
        let tap_path = dir.path().join("homebrew-brewtool");
        let mut config = base_config(&tap_path);
        config.tap.path = None;
        config.tap.formula_path = None;
        config.artifact.asset_template = Some("brewtool-{version}.tar.gz".to_string());

        let ctx = base_context(config, dir.path());
        let args = base_release_args();

        let err = resolve_release_context(&ctx, &args, None, None, args.dry_run).unwrap_err();
        assert!(matches!(err, AppError::MissingConfig(_)));
        assert!(err.to_string().contains("tap.remote"));
    }

    #[test]
    fn release_rejects_mixed_target_key_shapes_before_network() {
        let dir = tempdir().unwrap();
        let tap_path = dir.path().join("homebrew-brewtool");
        let mut config = base_config(&tap_path);
        config.artifact.asset_template = Some("brewtool-{version}-{os}-{arch}.tar.gz".to_string());

        let mut targets = BTreeMap::new();
        targets.insert(
            "darwin".to_string(),
            ArtifactTarget {
                asset_template: Some("brewtool-{version}-darwin.tar.gz".to_string()),
                asset_name: None,
                extra: Default::default(),
            },
        );
        targets.insert(
            "darwin-arm64".to_string(),
            ArtifactTarget {
                asset_template: Some("brewtool-{version}-darwin-arm64.tar.gz".to_string()),
                asset_name: None,
                extra: Default::default(),
            },
        );
        config.artifact.targets = targets;

        let ctx = base_context(config, dir.path());
        let args = base_release_args();

        let err =
            resolve_release_context(&ctx, &args, Some(&tap_path), None, args.dry_run).unwrap_err();
        assert!(matches!(err, AppError::InvalidInput(_)));
        assert_eq!(
            err.to_string(),
            "target keys must be all <os> or all <os>-<arch>"
        );
    }

    #[test]
    fn rejects_invalid_target_keys_before_network() {
        let dir = tempdir().unwrap();
        let tap_path = dir.path().join("homebrew-brewtool");
        let mut config = base_config(&tap_path);
        config.artifact.asset_template = Some("brewtool-{version}-{os}.tar.gz".to_string());

        let mut targets = BTreeMap::new();
        targets.insert(
            "darwin-arm64-extra".to_string(),
            ArtifactTarget {
                asset_template: None,
                asset_name: None,
                extra: Default::default(),
            },
        );
        config.artifact.targets = targets;

        let ctx = base_context(config, dir.path());
        let args = base_release_args();

        let err =
            resolve_release_context(&ctx, &args, Some(&tap_path), None, args.dry_run).unwrap_err();
        assert!(matches!(err, AppError::InvalidInput(_)));
        assert_eq!(
            err.to_string(),
            "invalid target key 'darwin-arm64-extra': expected <os> or <os>-<arch>"
        );
    }

    #[test]
    fn per_os_targets_reject_arch_template_placeholder() {
        let dir = tempdir().unwrap();
        let tap_path = dir.path().join("homebrew-brewtool");
        let mut config = base_config(&tap_path);
        config.artifact.asset_template = Some("brewtool-{version}-{os}-{arch}.tar.gz".to_string());

        let mut targets = BTreeMap::new();
        targets.insert(
            "darwin".to_string(),
            ArtifactTarget {
                asset_template: None,
                asset_name: None,
                extra: Default::default(),
            },
        );
        targets.insert(
            "linux".to_string(),
            ArtifactTarget {
                asset_template: None,
                asset_name: None,
                extra: Default::default(),
            },
        );
        config.artifact.targets = targets;

        let ctx = base_context(config, dir.path());
        let args = base_release_args();

        let err =
            resolve_release_context(&ctx, &args, Some(&tap_path), None, args.dry_run).unwrap_err();
        assert!(matches!(err, AppError::InvalidInput(_)));
        assert_eq!(
            err.to_string(),
            "per-OS target 'darwin' cannot use {arch} in asset_template; use <os>-<arch> targets instead"
        );
    }

    #[test]
    fn version_update_plan_does_not_write_when_template_missing() {
        let dir = tempdir().unwrap();
        let tap_path = dir.path().join("homebrew-brewtool");
        let formula_path = tap_path.join("Formula/brewtool.rb");
        std::fs::create_dir_all(formula_path.parent().unwrap()).unwrap();
        std::fs::write(
            &formula_path,
            "class Brewtool < Formula\n  version \"0.1.0\"\nend\n",
        )
        .unwrap();

        let cargo_manifest = dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_manifest,
            "[package]\nname = \"brewtool\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        let (base_url, server) = spawn_tarball_server(b"tarball-data".to_vec());

        let mut config = base_config(&tap_path);
        config.artifact.strategy = Some("source-tarball".to_string());
        config.artifact.asset_template = None;
        config.artifact.asset_name = None;
        config.release.tarball_url_template =
            Some(format!("{base_url}/brewtool-{{version}}.tar.gz"));
        config.version_update.mode = Some("cargo".to_string());
        config.template.path = Some(dir.path().join("missing-template.rb"));

        let mut ctx = base_context(config, dir.path());
        ctx.repo.git_root = Some(dir.path().to_path_buf());

        let mut args = base_release_args();
        args.dry_run = false;
        args.allow_dirty = true;
        args.skip_tag = true;
        args.tap_path = Some(tap_path.clone());
        args.version = Some("1.2.3".to_string());
        args.tag = Some("v1.2.3".to_string());

        let err = run(&ctx, &args).unwrap_err();
        assert!(matches!(err, AppError::InvalidInput(_)));
        assert!(err.to_string().contains("template not found"));

        server.join().unwrap();

        let cargo_after = std::fs::read_to_string(&cargo_manifest).unwrap();
        assert!(cargo_after.contains("version = \"0.1.0\""));
        let formula_after = std::fs::read_to_string(&formula_path).unwrap();
        assert!(formula_after.contains("version \"0.1.0\""));
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
        let url = render_tarball_url(None, "acme", "brewtool", "v1.2.3", "1.2.3").unwrap();
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
