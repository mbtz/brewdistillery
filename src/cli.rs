use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "bd", version, about = "Homebrew formula initialization and release helper")]
pub struct Cli {
    #[arg(global = true, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[arg(global = true, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Init(InitArgs),
    #[command(alias = "ship")]
    Release(ReleaseArgs),
    Doctor(DoctorArgs),
}

#[derive(Args, Debug)]
pub struct InitArgs {
    #[arg(long)]
    pub non_interactive: bool,

    #[arg(long, value_name = "PATH")]
    pub tap_path: Option<PathBuf>,

    #[arg(long)]
    pub tap_remote: Option<String>,

    #[arg(long)]
    pub tap_owner: Option<String>,

    #[arg(long)]
    pub tap_repo: Option<String>,

    #[arg(long)]
    pub formula_name: Option<String>,

    #[arg(long)]
    pub repo_name: Option<String>,

    #[arg(long)]
    pub license: Option<String>,

    #[arg(long)]
    pub homepage: Option<String>,

    #[arg(long)]
    pub description: Option<String>,

    #[arg(long, action = clap::ArgAction::Append)]
    pub bin_name: Vec<String>,

    #[arg(long)]
    pub version: Option<String>,

    #[arg(long)]
    pub artifact_strategy: Option<String>,

    #[arg(long)]
    pub asset_template: Option<String>,

    #[arg(long)]
    pub host_owner: Option<String>,

    #[arg(long)]
    pub host_repo: Option<String>,

    #[arg(long)]
    pub yes: bool,

    #[arg(long)]
    pub force: bool,

    #[arg(long)]
    pub dry_run: bool,

    #[arg(long)]
    pub tap_new: bool,

    #[arg(long)]
    pub import_formula: bool,
}

#[derive(Args, Debug)]
pub struct ReleaseArgs {
    #[arg(long)]
    pub version: Option<String>,

    #[arg(long)]
    pub tag: Option<String>,

    #[arg(long)]
    pub skip_tag: bool,

    #[arg(long)]
    pub no_push: bool,

    #[arg(long, value_name = "PATH")]
    pub tap_path: Option<PathBuf>,

    #[arg(long)]
    pub tap_remote: Option<String>,

    #[arg(long)]
    pub artifact_strategy: Option<String>,

    #[arg(long)]
    pub asset_template: Option<String>,

    #[arg(long)]
    pub asset_name: Option<String>,

    #[arg(long)]
    pub include_prerelease: bool,

    #[arg(long)]
    pub force: bool,

    #[arg(long)]
    pub host_owner: Option<String>,

    #[arg(long)]
    pub host_repo: Option<String>,

    #[arg(long)]
    pub dry_run: bool,

    #[arg(long)]
    pub allow_dirty: bool,
}

#[derive(Args, Debug)]
pub struct DoctorArgs {
    #[arg(long, value_name = "PATH")]
    pub tap_path: Option<PathBuf>,

    #[arg(long)]
    pub strict: bool,

    #[arg(long)]
    pub audit: bool,
}
