use crate::cli::InitArgs;
use crate::config::Config;
use crate::context::AppContext;
use crate::errors::AppError;
use crate::formula::{normalize_formula_name, AssetMatrix, FormulaAsset, FormulaSpec};
use crate::preview::{PlannedWrite, RepoPlan};
use crate::repo_detect::ProjectMetadata;
use crate::version::resolve_version_tag;
use dialoguer::{Confirm, Input};
use dialoguer::theme::ColorfulTheme;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run(ctx: &AppContext, args: &InitArgs) -> Result<(), AppError> {
    if args.non_interactive {
        return run_non_interactive(ctx, args);
    }

    if args.import_formula {
        return Err(AppError::Other(
            "--import-formula is not implemented for interactive init yet".to_string(),
        ));
    }

    run_interactive(ctx, args)
}

#[derive(Debug, Clone)]
struct ResolvedInit {
    formula_name: String,
    project_name: String,
    description: String,
    homepage: String,
    license: String,
    version: String,
    bins: Vec<String>,
    cli_owner: String,
    cli_repo: String,
    tap_owner: Option<String>,
    tap_repo: Option<String>,
    tap_remote: Option<String>,
    tap_path: Option<PathBuf>,
    artifact_strategy: Option<String>,
    asset_template: Option<String>,
    allow_overwrite: bool,
}

fn run_interactive(ctx: &AppContext, args: &InitArgs) -> Result<(), AppError> {
    let metadata = ctx.repo.metadata.as_ref();
    let theme = ColorfulTheme::default();

    let formula_default = resolve_string(
        args.formula_name.as_ref(),
        ctx.config.tap.formula.as_ref(),
        args.repo_name
            .as_ref()
            .or_else(|| metadata.and_then(|meta| meta.name.as_ref())),
    );

    let formula_name = prompt_formula_name(&theme, "Formula name", formula_default)?;

    let description = prompt_required(
        &theme,
        "Description",
        resolve_string(
            args.description.as_ref(),
            ctx.config.project.description.as_ref(),
            metadata.and_then(|meta| meta.description.as_ref()),
        ),
    )?;

    let homepage = prompt_required(
        &theme,
        "Homepage",
        resolve_string(
            args.homepage.as_ref(),
            ctx.config.project.homepage.as_ref(),
            metadata.and_then(|meta| meta.homepage.as_ref()),
        ),
    )?;

    let license = prompt_required(
        &theme,
        "License (SPDX)",
        resolve_string(
            args.license.as_ref(),
            ctx.config.project.license.as_ref(),
            metadata.and_then(|meta| meta.license.as_ref()),
        ),
    )?;

    let version = prompt_version(
        &theme,
        "Version",
        resolve_string(
            args.version.as_ref(),
            None,
            metadata.and_then(|meta| meta.version.as_ref()),
        ),
    )?;

    let bins_default = resolve_bins(args, &ctx.config, metadata);
    let bins = prompt_bins(&theme, "Binary name(s)", bins_default, &formula_name)?;

    let project_name = resolve_string(
        args.repo_name.as_ref(),
        ctx.config.project.name.as_ref(),
        metadata.and_then(|meta| meta.name.as_ref()),
    )
    .unwrap_or_else(|| formula_name.clone());

    let (owner_default, repo_default) = infer_owner_repo_defaults(
        args.host_owner.as_ref(),
        args.host_repo.as_ref(),
        ctx.config.cli.owner.as_ref(),
        ctx.config.cli.repo.as_ref(),
        Some(&homepage),
    );

    let cli_owner = prompt_required(&theme, "GitHub owner", owner_default)?;
    let cli_repo = prompt_required(&theme, "GitHub repo", repo_default)?;

    let tap_path = if let Some(path) = args
        .tap_path
        .clone()
        .or_else(|| ctx.config.tap.path.clone())
    {
        Some(path)
    } else {
        let default_repo = ctx
            .config
            .tap
            .repo
            .clone()
            .unwrap_or_else(|| format!("homebrew-{}", project_name));
        let default_path = default_tap_path(ctx, &default_repo);
        Some(prompt_path(
            &theme,
            "Tap path",
            Some(default_path.to_string_lossy().to_string()),
        )?)
    };

    let tap_remote = resolve_string(
        args.tap_remote.as_ref(),
        ctx.config.tap.remote.as_ref(),
        None,
    );

    let mut tap_owner = resolve_string(args.tap_owner.as_ref(), ctx.config.tap.owner.as_ref(), None);
    let mut tap_repo = resolve_string(args.tap_repo.as_ref(), ctx.config.tap.repo.as_ref(), None);

    if tap_owner.is_none() || tap_repo.is_none() {
        if let Some(remote) = tap_remote.as_deref() {
            if let Some((owner, repo)) = parse_owner_repo_from_remote(remote) {
                if tap_owner.is_none() {
                    tap_owner = Some(owner);
                }
                if tap_repo.is_none() {
                    tap_repo = Some(repo);
                }
            }
        }
    }

    if tap_owner.is_none() || tap_repo.is_none() {
        if let Some(path) = tap_path.as_ref() {
            if let Some(repo_name) = path.file_name().and_then(|name| name.to_str()) {
                if tap_repo.is_none() {
                    tap_repo = Some(repo_name.to_string());
                }
            }
        }
        if tap_owner.is_none() {
            tap_owner = Some(cli_owner.clone());
        }
    }

    if args.tap_new {
        tap_owner = Some(prompt_required(
            &theme,
            "Tap owner (for brew tap-new)",
            tap_owner.clone(),
        )?);
        tap_repo = Some(prompt_required(
            &theme,
            "Tap repo (for brew tap-new)",
            tap_repo.clone(),
        )?);
    }

    let artifact_strategy = resolve_string(
        args.artifact_strategy.as_ref(),
        ctx.config.artifact.strategy.as_ref(),
        None,
    );
    let asset_template = resolve_string(
        args.asset_template.as_ref(),
        ctx.config.artifact.asset_template.as_ref(),
        None,
    );

    let mut resolved = ResolvedInit {
        formula_name,
        project_name,
        description,
        homepage,
        license,
        version,
        bins,
        cli_owner,
        cli_repo,
        tap_owner,
        tap_repo,
        tap_remote,
        tap_path,
        artifact_strategy,
        asset_template,
        allow_overwrite: args.force || args.yes,
    };

    resolve_tap_repo(ctx, args, &mut resolved)?;

    let mut next_config = ctx.config.clone();
    apply_resolved(&mut next_config, &resolved);

    let rendered_config = render_config(&next_config, &ctx.config_path)?;

    let formula_path = resolve_formula_path(&resolved, &next_config);
    let formula_content = match formula_path.as_ref() {
        Some(_) => Some(render_formula(&resolved, &next_config)?),
        None => None,
    };

    let mut plans = Vec::new();
    plans.push(RepoPlan {
        label: "cli".to_string(),
        repo_root: ctx.cwd.clone(),
        writes: vec![PlannedWrite {
            path: ctx.config_path.clone(),
            content: rendered_config.clone(),
        }],
    });

    if let (Some(path), Some(content)) = (formula_path.clone(), formula_content.clone()) {
        let tap_root = path
            .parent()
            .and_then(|parent| parent.parent())
            .map(|path| path.to_path_buf())
            .unwrap_or_else(|| ctx.cwd.clone());
        plans.push(RepoPlan {
            label: "tap".to_string(),
            repo_root: tap_root,
            writes: vec![PlannedWrite { path, content }],
        });
    }

    let preview = crate::preview::preview_and_apply(&plans, true)?;
    if !preview.summary.trim().is_empty() {
        println!("{}", preview.summary.trim_end());
    }
    if !preview.diff.trim().is_empty() {
        println!("{}", preview.diff.trim_end());
    }
    if preview.changed_files.is_empty() {
        println!("init: no changes to apply");
    }

    if args.dry_run {
        println!("dry-run: no changes applied");
        return Ok(());
    }

    let apply = if resolved.allow_overwrite {
        true
    } else {
        Confirm::with_theme(&theme)
            .with_prompt("Apply these changes?")
            .default(true)
            .interact()
            .map_err(|err| AppError::Other(format!("failed to read confirmation: {err}")))?
    };

    if !apply {
        println!("init: cancelled");
        return Ok(());
    }

    let _ = crate::preview::preview_and_apply(&plans, false)?;
    Ok(())
}

fn run_non_interactive(ctx: &AppContext, args: &InitArgs) -> Result<(), AppError> {
    let mut resolved = resolve_required(ctx, args)?;
    resolve_tap_repo(ctx, args, &mut resolved)?;

    let mut next_config = ctx.config.clone();
    apply_resolved(&mut next_config, &resolved);

    let rendered = render_config(&next_config, &ctx.config_path)?;
    let existing = read_optional(&ctx.config_path)?;

    let formula_path = resolve_formula_path(&resolved, &next_config);
    let formula_content = match formula_path.as_ref() {
        Some(_) => Some(render_formula(&resolved, &next_config)?),
        None => None,
    };
    let existing_formula = match formula_path.as_ref() {
        Some(path) => read_optional(path)?,
        None => None,
    };

    if let Some(existing) = existing.as_deref() {
        if existing != rendered && !resolved.allow_overwrite {
            return Err(AppError::InvalidInput(format!(
                "config already exists at {}; re-run with --force or --yes to overwrite",
                ctx.config_path.display()
            )));
        }
    }

    if let (Some(existing), Some(formula)) =
        (existing_formula.as_deref(), formula_content.as_deref())
    {
        if existing != formula && !resolved.allow_overwrite {
            return Err(AppError::InvalidInput(format!(
                "formula already exists at {}; re-run with --force or --yes to overwrite",
                formula_path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_default()
            )));
        }
    }

    if args.dry_run {
        return Ok(());
    }

    if existing.as_deref() != Some(rendered.as_str()) {
        next_config.save(&ctx.config_path)?;
    }

    if let (Some(path), Some(content)) = (formula_path, formula_content) {
        if existing_formula.as_deref() != Some(content.as_str()) {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, content)?;
        }
    }

    Ok(())
}

fn resolve_required(ctx: &AppContext, args: &InitArgs) -> Result<ResolvedInit, AppError> {
    let metadata = ctx.repo.metadata.as_ref();
    let mut missing = Vec::new();

    let description = resolve_string(
        args.description.as_ref(),
        ctx.config.project.description.as_ref(),
        metadata.and_then(|meta| meta.description.as_ref()),
    )
    .unwrap_or_else(|| {
        missing.push("description".to_string());
        String::new()
    });

    let homepage = resolve_string(
        args.homepage.as_ref(),
        ctx.config.project.homepage.as_ref(),
        metadata.and_then(|meta| meta.homepage.as_ref()),
    )
    .unwrap_or_else(|| {
        missing.push("homepage".to_string());
        String::new()
    });

    let license = resolve_string(
        args.license.as_ref(),
        ctx.config.project.license.as_ref(),
        metadata.and_then(|meta| meta.license.as_ref()),
    )
    .unwrap_or_else(|| {
        missing.push("license".to_string());
        String::new()
    });

    let version = match resolve_string(
        args.version.as_ref(),
        None,
        metadata.and_then(|meta| meta.version.as_ref()),
    ) {
        Some(value) => {
            let resolved = resolve_version_tag(Some(value.as_str()), None)?;
            resolved.version.unwrap_or(value)
        }
        None => {
            missing.push("version".to_string());
            String::new()
        }
    };

    let cli_owner = resolve_string(args.host_owner.as_ref(), ctx.config.cli.owner.as_ref(), None)
        .unwrap_or_else(|| {
            missing.push("host-owner".to_string());
            String::new()
        });

    let cli_repo = resolve_string(args.host_repo.as_ref(), ctx.config.cli.repo.as_ref(), None)
        .unwrap_or_else(|| {
            missing.push("host-repo".to_string());
            String::new()
        });

    let tap_owner =
        resolve_string(args.tap_owner.as_ref(), ctx.config.tap.owner.as_ref(), None);
    let tap_repo = resolve_string(args.tap_repo.as_ref(), ctx.config.tap.repo.as_ref(), None);
    let tap_remote =
        resolve_string(args.tap_remote.as_ref(), ctx.config.tap.remote.as_ref(), None);
    let tap_path = args.tap_path.clone().or_else(|| ctx.config.tap.path.clone());

    if tap_path.is_none() && tap_remote.is_none() && !(tap_owner.is_some() && tap_repo.is_some()) {
        missing.push("tap-path or tap-remote or tap-owner+tap-repo".to_string());
    }
    if args.tap_new {
        if tap_owner.is_none() {
            missing.push("tap-owner".to_string());
        }
        if tap_repo.is_none() {
            missing.push("tap-repo".to_string());
        }
    }

    let formula_source = resolve_string(
        args.formula_name.as_ref(),
        ctx.config.tap.formula.as_ref(),
        args.repo_name
            .as_ref()
            .or_else(|| metadata.and_then(|meta| meta.name.as_ref())),
    )
    .unwrap_or_else(|| {
        missing.push("formula-name".to_string());
        String::new()
    });

    let formula_name = if formula_source.is_empty() {
        String::new()
    } else {
        normalize_formula_name(&formula_source)?
    };

    if formula_name.is_empty() {
        missing.push("formula-name".to_string());
    }

    let bins = resolve_bins(args, &ctx.config, metadata);
    if bins.is_empty() {
        missing.push("bin-name".to_string());
    }

    if !missing.is_empty() {
        missing.sort();
        missing.dedup();
        return Err(AppError::MissingConfig(format!(
            "missing required fields for --non-interactive: {}",
            missing.join(", ")
        )));
    }

    let project_name = resolve_string(
        args.repo_name.as_ref(),
        ctx.config.project.name.as_ref(),
        metadata.and_then(|meta| meta.name.as_ref()),
    )
    .unwrap_or_else(|| formula_name.clone());

    let artifact_strategy =
        resolve_string(args.artifact_strategy.as_ref(), ctx.config.artifact.strategy.as_ref(), None);
    let asset_template =
        resolve_string(args.asset_template.as_ref(), ctx.config.artifact.asset_template.as_ref(), None);

    Ok(ResolvedInit {
        formula_name,
        project_name,
        description,
        homepage,
        license,
        version,
        bins,
        cli_owner,
        cli_repo,
        tap_owner,
        tap_repo,
        tap_remote,
        tap_path,
        artifact_strategy,
        asset_template,
        allow_overwrite: args.force || args.yes,
    })
}

fn resolve_tap_repo(
    ctx: &AppContext,
    args: &InitArgs,
    resolved: &mut ResolvedInit,
) -> Result<(), AppError> {
    if args.tap_new {
        let owner = resolved
            .tap_owner
            .as_deref()
            .unwrap_or_default()
            .trim();
        let repo = resolved
            .tap_repo
            .as_deref()
            .unwrap_or_default()
            .trim();
        if owner.is_empty() || repo.is_empty() {
            return Err(AppError::MissingConfig(
                "missing required fields for --tap-new: tap-owner, tap-repo".to_string(),
            ));
        }

        let brew_root = brew_repo_root()?;
        let tap_path = brew_root
            .join("Library")
            .join("Taps")
            .join(owner)
            .join(repo);

        if let Some(existing) = resolved.tap_path.as_ref() {
            if existing != &tap_path {
                return Err(AppError::InvalidInput(format!(
                    "--tap-new uses Homebrew tap directory {}; remove --tap-path or set it to match",
                    tap_path.display()
                )));
            }
        }

        resolved.tap_path = Some(tap_path.clone());

        if args.dry_run {
            return Ok(());
        }

        if tap_path.exists() {
            if !tap_path.is_dir() {
                return Err(AppError::InvalidInput(format!(
                    "tap path exists but is not a directory: {}",
                    tap_path.display()
                )));
            }
            return Ok(());
        }

        run_brew_tap_new(owner, repo)?;
        return Ok(());
    }

    let remote = resolved
        .tap_remote
        .as_deref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty());

    if remote.is_none() {
        return Ok(());
    }

    let remote = remote.unwrap();

    if resolved.tap_owner.is_none() || resolved.tap_repo.is_none() {
        if let Some((owner, repo)) = parse_owner_repo_from_remote(remote) {
            if resolved.tap_owner.is_none() {
                resolved.tap_owner = Some(owner);
            }
            if resolved.tap_repo.is_none() {
                resolved.tap_repo = Some(repo);
            }
        }
    }

    let tap_path = if let Some(path) = resolved.tap_path.as_ref() {
        path.clone()
    } else {
        let repo = resolved
            .tap_repo
            .as_deref()
            .and_then(|value| {
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            })
            .or_else(|| parse_owner_repo_from_remote(remote).map(|(_, repo)| repo))
            .ok_or_else(|| {
                AppError::InvalidInput(
                    "tap remote provided but tap repo name could not be derived; set --tap-path or --tap-repo".to_string(),
                )
            })?;
        default_tap_path(ctx, &repo)
    };

    resolved.tap_path = Some(tap_path.clone());

    if args.dry_run {
        return Ok(());
    }

    if tap_path.exists() {
        if !tap_path.is_dir() {
            return Err(AppError::InvalidInput(format!(
                "tap path exists but is not a directory: {}",
                tap_path.display()
            )));
        }

        if tap_path.join(".git").exists() {
            return Ok(());
        }

        if !is_dir_empty(&tap_path)? {
            return Err(AppError::InvalidInput(format!(
                "tap path already exists but is not a git repo: {}",
                tap_path.display()
            )));
        }
    }

    if let Some(parent) = tap_path.parent() {
        fs::create_dir_all(parent)?;
    }
    run_git_clone(remote, &tap_path)?;
    Ok(())
}

fn default_tap_path(ctx: &AppContext, repo: &str) -> PathBuf {
    let base = ctx.cwd.parent().unwrap_or(ctx.cwd.as_path());
    base.join(repo)
}

fn brew_repo_root() -> Result<PathBuf, AppError> {
    let output = Command::new("brew")
        .arg("--repo")
        .output()
        .map_err(|err| {
            AppError::InvalidInput(format!(
                "brew tap-new requires Homebrew; failed to run brew --repo: {err}"
            ))
        })?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let mut message = "brew --repo failed".to_string();
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
        return Err(AppError::InvalidInput(message));
    }

    let repo_root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if repo_root.is_empty() {
        return Err(AppError::InvalidInput(
            "brew --repo returned empty output".to_string(),
        ));
    }
    Ok(PathBuf::from(repo_root))
}

fn run_brew_tap_new(owner: &str, repo: &str) -> Result<(), AppError> {
    let output = Command::new("brew")
        .arg("tap-new")
        .arg(format!("{owner}/{repo}"))
        .output()
        .map_err(|err| {
            AppError::InvalidInput(format!(
                "failed to run brew tap-new {owner}/{repo}: {err}"
            ))
        })?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let mut message = format!("brew tap-new {owner}/{repo} failed");
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
        return Err(AppError::InvalidInput(message));
    }

    Ok(())
}

fn parse_owner_repo_from_remote(remote: &str) -> Option<(String, String)> {
    parse_owner_repo_from_github(remote)
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

fn is_dir_empty(path: &Path) -> Result<bool, AppError> {
    let mut entries = fs::read_dir(path)?;
    Ok(entries.next().is_none())
}

fn resolve_string(
    flag: Option<&String>,
    config: Option<&String>,
    meta: Option<&String>,
) -> Option<String> {
    flag.or(config)
        .or(meta)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
}

fn resolve_bins(args: &InitArgs, config: &Config, meta: Option<&ProjectMetadata>) -> Vec<String> {
    if !args.bin_name.is_empty() {
        return normalize_bins(args.bin_name.iter().cloned().collect());
    }

    if !config.project.bin.is_empty() {
        return normalize_bins(config.project.bin.clone());
    }

    if let Some(meta) = meta {
        if !meta.bin.is_empty() {
            return normalize_bins(meta.bin.clone());
        }
    }

    Vec::new()
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

fn prompt_required(
    theme: &ColorfulTheme,
    label: &str,
    default: Option<String>,
) -> Result<String, AppError> {
    loop {
        let mut input = Input::with_theme(theme).with_prompt(label);
        if let Some(default) = default.as_deref() {
            input = input.default(default.to_string());
        }
        let value = input
            .allow_empty(true)
            .interact_text()
            .map_err(|err| AppError::Other(format!("failed to read {label}: {err}")))?;
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }
}

fn prompt_formula_name(
    theme: &ColorfulTheme,
    label: &str,
    default: Option<String>,
) -> Result<String, AppError> {
    loop {
        let value = prompt_required(theme, label, default.clone())?;
        match normalize_formula_name(&value) {
            Ok(normalized) => return Ok(normalized),
            Err(err) => {
                println!("invalid formula name: {err}");
            }
        }
    }
}

fn prompt_version(
    theme: &ColorfulTheme,
    label: &str,
    default: Option<String>,
) -> Result<String, AppError> {
    loop {
        let value = prompt_required(theme, label, default.clone())?;
        let resolved = resolve_version_tag(Some(value.as_str()), None);
        match resolved {
            Ok(resolved) => {
                return Ok(resolved.version.unwrap_or(value));
            }
            Err(err) => {
                println!("invalid version: {err}");
            }
        }
    }
}

fn prompt_bins(
    theme: &ColorfulTheme,
    label: &str,
    default_bins: Vec<String>,
    fallback: &str,
) -> Result<Vec<String>, AppError> {
    let default_value = if default_bins.is_empty() {
        Some(fallback.to_string())
    } else {
        Some(default_bins.join(", "))
    };

    loop {
        let value = prompt_required(theme, label, default_value.clone())?;
        let bins = parse_bins_input(&value);
        if !bins.is_empty() {
            return Ok(bins);
        }
    }
}

fn parse_bins_input(input: &str) -> Vec<String> {
    let mut bins = Vec::new();
    for entry in input.split(',') {
        let trimmed = entry.trim();
        if trimmed.is_empty() {
            continue;
        }
        for part in trimmed.split_whitespace() {
            if !part.trim().is_empty() {
                bins.push(part.trim().to_string());
            }
        }
    }
    normalize_bins(bins)
}

fn prompt_path(
    theme: &ColorfulTheme,
    label: &str,
    default: Option<String>,
) -> Result<PathBuf, AppError> {
    loop {
        let value = prompt_required(theme, label, default.clone())?;
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Ok(PathBuf::from(trimmed));
        }
    }
}

fn infer_owner_repo_defaults(
    owner_flag: Option<&String>,
    repo_flag: Option<&String>,
    owner_config: Option<&String>,
    repo_config: Option<&String>,
    homepage: Option<&str>,
) -> (Option<String>, Option<String>) {
    let owner = resolve_string(owner_flag, owner_config, None);
    let repo = resolve_string(repo_flag, repo_config, None);

    if owner.is_some() && repo.is_some() {
        return (owner, repo);
    }

    if let Some(homepage) = homepage {
        if let Some((derived_owner, derived_repo)) = parse_owner_repo_from_homepage(homepage) {
            return (
                owner.or(Some(derived_owner)),
                repo.or(Some(derived_repo)),
            );
        }
    }

    (owner, repo)
}

fn parse_owner_repo_from_homepage(homepage: &str) -> Option<(String, String)> {
    parse_owner_repo_from_github(homepage)
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

fn apply_resolved(config: &mut Config, resolved: &ResolvedInit) {
    if config.schema_version.is_none() {
        config.schema_version = Some(1);
    }

    config.project.name = Some(resolved.project_name.clone());
    config.project.description = Some(resolved.description.clone());
    config.project.homepage = Some(resolved.homepage.clone());
    config.project.license = Some(resolved.license.clone());
    config.project.bin = resolved.bins.clone();

    config.cli.owner = Some(resolved.cli_owner.clone());
    config.cli.repo = Some(resolved.cli_repo.clone());

    config.tap.owner = resolved.tap_owner.clone();
    config.tap.repo = resolved.tap_repo.clone();
    config.tap.remote = resolved.tap_remote.clone();
    config.tap.path = resolved.tap_path.clone();
    config.tap.formula = Some(resolved.formula_name.clone());

    if let Some(tap_path) = resolved.tap_path.as_ref() {
        config.tap.formula_path =
            Some(tap_path.join("Formula").join(format!("{}.rb", resolved.formula_name)));
    }

    if let Some(strategy) = resolved.artifact_strategy.as_ref() {
        config.artifact.strategy = Some(strategy.clone());
    }

    if let Some(template) = resolved.asset_template.as_ref() {
        config.artifact.asset_template = Some(template.clone());
    }
}

fn render_config(config: &Config, path: &Path) -> Result<String, AppError> {
    let content = toml::to_string_pretty(config).map_err(|err| {
        AppError::InvalidInput(format!("failed to serialize config {}: {err}", path.display()))
    })?;
    Ok(format!("{content}\n"))
}

fn read_optional(path: &Path) -> Result<Option<String>, AppError> {
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(fs::read_to_string(path)?))
}

fn resolve_formula_path(resolved: &ResolvedInit, config: &Config) -> Option<PathBuf> {
    config
        .tap
        .formula_path
        .clone()
        .or_else(|| resolved.tap_path.as_ref().map(|path| {
            path.join("Formula")
                .join(format!("{}.rb", resolved.formula_name))
        }))
}

fn render_formula(resolved: &ResolvedInit, config: &Config) -> Result<String, AppError> {
    let asset = build_placeholder_asset(resolved, config);
    let spec = FormulaSpec {
        name: resolved.formula_name.clone(),
        desc: resolved.description.clone(),
        homepage: resolved.homepage.clone(),
        license: resolved.license.clone(),
        version: resolved.version.clone(),
        bins: resolved.bins.clone(),
        assets: AssetMatrix::Universal(asset),
        install_block: config.template.install_block.clone(),
    };
    spec.render()
}

fn build_placeholder_asset(resolved: &ResolvedInit, config: &Config) -> FormulaAsset {
    let asset_name = config
        .artifact
        .asset_name
        .clone()
        .and_then(|value| {
            if value.trim().is_empty() {
                None
            } else {
                Some(value)
            }
        })
        .or_else(|| {
            config
                .artifact
                .asset_template
                .as_deref()
                .and_then(|template| {
                    render_asset_template(template, &resolved.project_name, &resolved.version)
                })
        })
        .unwrap_or_else(|| format!("{}-{}.tar.gz", resolved.project_name, resolved.version));

    let url = format!(
        "https://github.com/{}/{}/releases/download/{}/{}",
        resolved.cli_owner, resolved.cli_repo, resolved.version, asset_name
    );

    FormulaAsset {
        url,
        sha256: "TODO".to_string(),
    }
}

fn render_asset_template(template: &str, name: &str, version: &str) -> Option<String> {
    if template.contains("{os}") || template.contains("{arch}") {
        return None;
    }

    let mut output = template.to_string();
    if output.contains("{name}") {
        output = output.replace("{name}", name);
    }
    if output.contains("{version}") {
        output = output.replace("{version}", version);
    }

    let trimmed = output.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo_detect::{MetadataSource, RepoInfo};
    use tempfile::tempdir;

    fn base_args() -> InitArgs {
        InitArgs {
            non_interactive: true,
            tap_path: None,
            tap_remote: None,
            tap_owner: None,
            tap_repo: None,
            formula_name: None,
            repo_name: None,
            license: None,
            homepage: None,
            description: None,
            bin_name: Vec::new(),
            version: None,
            artifact_strategy: None,
            asset_template: None,
            host_owner: None,
            host_repo: None,
            yes: false,
            force: false,
            dry_run: false,
            tap_new: false,
            import_formula: false,
        }
    }

    fn base_context(dir: &Path) -> AppContext {
        AppContext {
            cwd: dir.to_path_buf(),
            config_path: dir.join(".distill/config.toml"),
            config: Config::default(),
            repo: RepoInfo {
                git_root: Some(dir.to_path_buf()),
                metadata: Some(ProjectMetadata {
                    name: Some("brewtool".to_string()),
                    description: Some("Brew tool".to_string()),
                    homepage: Some("https://example.com".to_string()),
                    license: Some("MIT".to_string()),
                    version: Some("1.2.3".to_string()),
                    bin: vec!["brewtool".to_string()],
                    source: MetadataSource::Cargo,
                }),
            },
            verbose: 0,
        }
    }

    #[test]
    fn errors_when_required_fields_missing() {
        let dir = tempdir().unwrap();
        let ctx = base_context(dir.path());
        let args = base_args();

        let err = run_non_interactive(&ctx, &args).unwrap_err();
        assert!(matches!(err, AppError::MissingConfig(_)));
    }

    #[test]
    fn writes_config_for_non_interactive() {
        let dir = tempdir().unwrap();
        let ctx = base_context(dir.path());
        let mut args = base_args();
        let tap_path = dir.path().join("tap");
        args.tap_path = Some(tap_path.clone());
        args.host_owner = Some("acme".to_string());
        args.host_repo = Some("brewtool".to_string());

        run_non_interactive(&ctx, &args).unwrap();
        let config = Config::load(&ctx.config_path).unwrap();

        assert_eq!(config.project.name.as_deref(), Some("brewtool"));
        assert_eq!(config.tap.formula.as_deref(), Some("brewtool"));
        assert_eq!(config.cli.owner.as_deref(), Some("acme"));

        let formula_path = tap_path.join("Formula").join("brewtool.rb");
        let formula = fs::read_to_string(formula_path).unwrap();
        assert!(formula.contains("class Brewtool < Formula"));
        assert!(formula.contains("version \"1.2.3\""));
        assert!(formula.contains("sha256 \"TODO\""));
    }

    #[test]
    fn requires_force_for_config_overwrite() {
        let dir = tempdir().unwrap();
        let mut ctx = base_context(dir.path());
        let mut args = base_args();
        args.tap_path = Some(dir.path().join("tap"));
        args.host_owner = Some("acme".to_string());
        args.host_repo = Some("brewtool".to_string());

        let mut existing = ctx.config.clone();
        existing.schema_version = Some(1);
        existing.project.name = Some("old".to_string());
        existing.save(&ctx.config_path).unwrap();
        ctx.config = existing;

        let err = run_non_interactive(&ctx, &args).unwrap_err();
        assert!(matches!(err, AppError::InvalidInput(_)));

        args.force = true;
        run_non_interactive(&ctx, &args).unwrap();
    }

    #[test]
    fn derives_tap_path_from_remote_when_missing() {
        let dir = tempdir().unwrap();
        let cli_dir = dir.path().join("cli");
        fs::create_dir_all(&cli_dir).unwrap();

        let mut ctx = base_context(dir.path());
        ctx.cwd = cli_dir.clone();
        ctx.config_path = cli_dir.join(".distill/config.toml");

        let mut resolved = ResolvedInit {
            formula_name: "brewtool".to_string(),
            project_name: "brewtool".to_string(),
            description: "Brew tool".to_string(),
            homepage: "https://example.com".to_string(),
            license: "MIT".to_string(),
            version: "1.2.3".to_string(),
            bins: vec!["brewtool".to_string()],
            cli_owner: "acme".to_string(),
            cli_repo: "brewtool".to_string(),
            tap_owner: None,
            tap_repo: None,
            tap_remote: Some("https://github.com/acme/homebrew-brewtool.git".to_string()),
            tap_path: None,
            artifact_strategy: None,
            asset_template: None,
            allow_overwrite: false,
        };

        let mut args = base_args();
        args.dry_run = true;

        resolve_tap_repo(&ctx, &args, &mut resolved).unwrap();

        let expected = dir.path().join("homebrew-brewtool");
        assert_eq!(resolved.tap_path.as_ref(), Some(&expected));
        assert_eq!(resolved.tap_owner.as_deref(), Some("acme"));
        assert_eq!(resolved.tap_repo.as_deref(), Some("homebrew-brewtool"));
    }

    #[test]
    fn parses_owner_repo_from_remote() {
        let parsed = parse_owner_repo_from_remote("git@github.com:acme/homebrew-foo.git");
        assert_eq!(
            parsed,
            Some(("acme".to_string(), "homebrew-foo".to_string()))
        );

        let parsed = parse_owner_repo_from_remote("https://github.com/acme/homebrew-bar");
        assert_eq!(
            parsed,
            Some(("acme".to_string(), "homebrew-bar".to_string()))
        );
    }
}
