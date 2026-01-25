use crate::cli::TemplateArgs;
use crate::context::AppContext;
use crate::errors::AppError;
use crate::formula::{default_template, validate_template_string};
use std::fs;
use std::path::Path;

pub fn run(_ctx: &AppContext, args: &TemplateArgs) -> Result<(), AppError> {
    if let Some(path) = args.validate.as_deref() {
        let template = fs::read_to_string(path)?;
        validate_template_string(&template)
            .map_err(|err| template_error_with_context(err, path))?;
        println!("template: ok ({})", path.display());
        return Ok(());
    }

    println!("{}", default_template());
    Ok(())
}

fn template_error_with_context(err: AppError, path: &Path) -> AppError {
    match err {
        AppError::InvalidInput(message) => {
            AppError::InvalidInput(format!("template {}: {message}", path.display()))
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::repo_detect::RepoInfo;
    use tempfile::tempdir;

    fn base_context(dir: &Path) -> AppContext {
        AppContext {
            cwd: dir.to_path_buf(),
            config_path: dir.join(".distill/config.toml"),
            config: Config::default(),
            repo: RepoInfo::default(),
            verbose: 0,
        }
    }

    #[test]
    fn validates_default_template() {
        let dir = tempdir().expect("tempdir");
        let template_path = dir.path().join("template.rb");
        fs::write(&template_path, default_template()).expect("write template");

        let ctx = base_context(dir.path());
        let args = TemplateArgs {
            validate: Some(template_path),
        };

        run(&ctx, &args).expect("default template should validate");
    }

    #[test]
    fn errors_when_required_placeholders_are_missing() {
        let dir = tempdir().expect("tempdir");
        let template_path = dir.path().join("template.rb");
        let missing_assets = default_template().replace("{assets}", "");
        fs::write(&template_path, missing_assets).expect("write template");

        let ctx = base_context(dir.path());
        let args = TemplateArgs {
            validate: Some(template_path),
        };

        let err = run(&ctx, &args).expect_err("missing placeholders should error");
        assert!(
            err.to_string().contains("missing required placeholder {assets}"),
            "unexpected error message: {err}"
        );
    }
}
