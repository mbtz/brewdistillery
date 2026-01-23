use crate::cli::InitArgs;
use crate::config::Config;
use crate::context::AppContext;
use crate::errors::AppError;
use crate::formula::normalize_formula_name;
use crate::repo_detect::ProjectMetadata;
use std::fs;
use std::path::{Path, PathBuf};

pub fn run(ctx: &AppContext, args: &InitArgs) -> Result<(), AppError> {
    if args.non_interactive {
        return run_non_interactive(ctx, args);
    }

    Err(AppError::Other(
        "interactive init is not implemented yet; use --non-interactive".to_string(),
    ))
}

#[derive(Debug, Clone)]
struct ResolvedInit {
    formula_name: String,
    project_name: String,
    description: String,
    homepage: String,
    license: String,
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

fn run_non_interactive(ctx: &AppContext, args: &InitArgs) -> Result<(), AppError> {
    let resolved = resolve_required(ctx, args)?;

    let mut next_config = ctx.config.clone();
    apply_resolved(&mut next_config, &resolved);

    let rendered = render_config(&next_config, &ctx.config_path)?;
    let existing = read_optional(&ctx.config_path)?;

    if let Some(existing) = existing.as_deref() {
        if existing != rendered && !resolved.allow_overwrite {
            return Err(AppError::InvalidInput(format!(
                "config already exists at {}; re-run with --force or --yes to overwrite",
                ctx.config_path.display()
            )));
        }
    }

    if args.dry_run {
        return Ok(());
    }

    if existing.as_deref() != Some(rendered.as_str()) {
        next_config.save(&ctx.config_path)?;
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

    let version = resolve_string(
        args.version.as_ref(),
        None,
        metadata.and_then(|meta| meta.version.as_ref()),
    );
    if version.is_none() {
        missing.push("version".to_string());
    }

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
        args.tap_path = Some(dir.path().join("tap"));
        args.host_owner = Some("acme".to_string());
        args.host_repo = Some("brewtool".to_string());

        run_non_interactive(&ctx, &args).unwrap();
        let config = Config::load(&ctx.config_path).unwrap();

        assert_eq!(config.project.name.as_deref(), Some("brewtool"));
        assert_eq!(config.tap.formula.as_deref(), Some("brewtool"));
        assert_eq!(config.cli.owner.as_deref(), Some("acme"));
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
}
