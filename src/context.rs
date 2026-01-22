use crate::cli::Cli;
use crate::config::Config;
use crate::errors::AppError;
use crate::repo_detect::{detect_repo, RepoInfo};
use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AppContext {
    pub cwd: PathBuf,
    pub config_path: PathBuf,
    pub config: Config,
    pub repo: RepoInfo,
    pub verbose: u8,
}

impl AppContext {
    pub fn load(cli: &Cli) -> Result<Self, AppError> {
        let cwd = env::current_dir()?;
        let config_path = cli
            .config
            .clone()
            .unwrap_or_else(|| cwd.join(".distill/config.toml"));
        let config = Config::load(&config_path)?;
        let repo = detect_repo(&cwd)?;

        Ok(Self {
            cwd,
            config_path,
            config,
            repo,
            verbose: cli.verbose,
        })
    }
}
