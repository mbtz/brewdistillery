use crate::cli::DoctorArgs;
use crate::config::Config;
use crate::context::AppContext;
use crate::errors::AppError;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
struct Issue {
    message: String,
    is_error: bool,
}

pub fn run(ctx: &AppContext, args: &DoctorArgs) -> Result<(), AppError> {
    if !ctx.config_path.exists() {
        return Err(AppError::MissingConfig(format!(
            "config not found at {}",
            ctx.config_path.display()
        )));
    }

    let mut issues = Vec::new();
    let config = &ctx.config;

    check_required_fields(config, &mut issues);
    check_tap(config, args.tap_path.as_ref(), &mut issues);
    check_artifact(config, &mut issues);

    let strict = args.strict;
    let (errors, warnings) = split_issues(&issues);
    let has_errors = !errors.is_empty() || (strict && !warnings.is_empty());

    if has_errors {
        let mut combined = errors;
        if strict {
            combined.extend(warnings);
        }
        return Err(AppError::InvalidInput(format_issues(
            "doctor found issues",
            &combined,
        )));
    }

    if !warnings.is_empty() {
        println!("{}", format_issues("doctor warnings", &warnings));
    } else {
        println!("doctor: ok");
    }

    if args.audit {
        run_brew_audit(config, args.tap_path.as_ref())?;
    }

    Ok(())
}

fn check_required_fields(config: &Config, issues: &mut Vec<Issue>) {
    if config.project.name.as_deref().unwrap_or_default().trim().is_empty() {
        push_issue(issues, true, "project.name is missing");
    }
    if config
        .project
        .description
        .as_deref()
        .unwrap_or_default()
        .trim()
        .is_empty()
    {
        push_issue(issues, true, "project.description is missing");
    }
    if config
        .project
        .homepage
        .as_deref()
        .unwrap_or_default()
        .trim()
        .is_empty()
    {
        push_issue(issues, true, "project.homepage is missing");
    }
    if config
        .project
        .license
        .as_deref()
        .unwrap_or_default()
        .trim()
        .is_empty()
    {
        push_issue(issues, true, "project.license is missing");
    }
    if config.project.bin.is_empty() {
        push_issue(issues, true, "project.bin is empty");
    }

    if config.cli.owner.as_deref().unwrap_or_default().trim().is_empty() {
        push_issue(issues, true, "cli.owner is missing");
    }
    if config.cli.repo.as_deref().unwrap_or_default().trim().is_empty() {
        push_issue(issues, true, "cli.repo is missing");
    }
}

fn check_tap(config: &Config, tap_override: Option<&PathBuf>, issues: &mut Vec<Issue>) {
    let tap_owner = config.tap.owner.as_deref().unwrap_or_default().trim();
    let tap_repo = config.tap.repo.as_deref().unwrap_or_default().trim();
    let tap_remote = config.tap.remote.as_deref().unwrap_or_default().trim();
    let tap_path = tap_override
        .map(|path| path.as_path())
        .or_else(|| config.tap.path.as_deref());

    if tap_path.is_none() && tap_remote.is_empty() && (tap_owner.is_empty() || tap_repo.is_empty())
    {
        push_issue(
            issues,
            true,
            "tap identity is missing (set tap.path, tap.remote, or tap.owner+tap.repo)",
        );
    }

    if let Some(path) = tap_path {
        if !path.exists() {
            push_issue(
                issues,
                true,
                format!("tap path does not exist: {}", path.display()),
            );
        }
    }

    let formula_name = config.tap.formula.as_deref().unwrap_or_default().trim();
    if formula_name.is_empty() {
        push_issue(issues, true, "tap.formula is missing");
        return;
    }

    if let Some(formula_path) = resolve_formula_path(config, tap_path) {
        if !formula_path.exists() {
            push_issue(
                issues,
                true,
                format!("formula file not found at {}", formula_path.display()),
            );
        }
    } else {
        push_issue(
            issues,
            false,
            "formula path cannot be resolved without tap.path or tap.formula_path",
        );
    }
}

fn check_artifact(config: &Config, issues: &mut Vec<Issue>) {
    if config.artifact.strategy.is_none() {
        push_issue(issues, false, "artifact.strategy is missing");
        return;
    }

    if config.artifact.asset_template.is_none() && config.artifact.asset_name.is_none() {
        push_issue(
            issues,
            false,
            "artifact.asset_template or artifact.asset_name is missing",
        );
    }
}

fn resolve_formula_path(config: &Config, tap_path: Option<&Path>) -> Option<PathBuf> {
    if let Some(path) = config.tap.formula_path.as_ref() {
        return Some(path.clone());
    }
    let formula = config.tap.formula.as_ref()?;
    let tap_path = tap_path?;
    Some(tap_path.join("Formula").join(format!("{formula}.rb")))
}

fn push_issue(issues: &mut Vec<Issue>, is_error: bool, message: impl Into<String>) {
    issues.push(Issue {
        message: message.into(),
        is_error,
    });
}

fn split_issues(issues: &[Issue]) -> (Vec<Issue>, Vec<Issue>) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    for issue in issues {
        if issue.is_error {
            errors.push(issue.clone());
        } else {
            warnings.push(issue.clone());
        }
    }
    (errors, warnings)
}

fn format_issues(title: &str, issues: &[Issue]) -> String {
    let mut output = String::new();
    output.push_str(title);
    output.push('\n');
    for issue in issues {
        output.push_str("  - ");
        output.push_str(&issue.message);
        output.push('\n');
    }
    output
}

fn run_brew_audit(config: &Config, tap_override: Option<&PathBuf>) -> Result<(), AppError> {
    let tap_path = tap_override
        .map(|path| path.as_path())
        .or_else(|| config.tap.path.as_deref());
    let formula_path = resolve_formula_path(config, tap_path).ok_or_else(|| {
        AppError::Audit("brew audit requires a resolved formula path".to_string())
    })?;

    let output = Command::new("brew")
        .arg("audit")
        .arg("--new-formula")
        .arg(&formula_path)
        .output()
        .map_err(|err| AppError::Audit(format!("failed to run brew audit: {err}")))?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let mut message = String::from("brew audit failed");
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
        return Err(AppError::Audit(message));
    }

    println!("brew audit: ok");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo_detect::RepoInfo;
    use tempfile::tempdir;

    fn base_config() -> Config {
        let mut config = Config::default();
        config.schema_version = Some(1);
        config.project.name = Some("brewtool".to_string());
        config.project.description = Some("Brew tool".to_string());
        config.project.homepage = Some("https://example.com".to_string());
        config.project.license = Some("MIT".to_string());
        config.project.bin = vec!["brewtool".to_string()];
        config.cli.owner = Some("acme".to_string());
        config.cli.repo = Some("brewtool".to_string());
        config.tap.formula = Some("brewtool".to_string());
        config
    }

    fn base_args() -> DoctorArgs {
        DoctorArgs {
            tap_path: None,
            strict: false,
            audit: false,
        }
    }

    #[test]
    fn errors_when_config_missing() {
        let dir = tempdir().unwrap();
        let ctx = AppContext {
            cwd: dir.path().to_path_buf(),
            config_path: dir.path().join(".distill/config.toml"),
            config: Config::default(),
            repo: RepoInfo::default(),
            verbose: 0,
        };

        let err = run(&ctx, &base_args()).unwrap_err();
        assert!(matches!(err, AppError::MissingConfig(_)));
    }

    #[test]
    fn errors_when_formula_missing() {
        let dir = tempdir().unwrap();
        let tap_path = dir.path().join("tap");
        std::fs::create_dir_all(&tap_path).unwrap();

        let mut config = base_config();
        config.tap.path = Some(tap_path.clone());
        let config_path = dir.path().join(".distill/config.toml");
        config.save(&config_path).unwrap();

        let ctx = AppContext {
            cwd: dir.path().to_path_buf(),
            config_path,
            config,
            repo: RepoInfo::default(),
            verbose: 0,
        };

        let err = run(&ctx, &base_args()).unwrap_err();
        assert!(matches!(err, AppError::InvalidInput(_)));
    }
}
