pub mod asset_selection;
pub mod cli;
pub mod commands;
pub mod config;
pub mod context;
pub mod errors;
pub mod formula;
pub mod git;
pub mod host;
pub mod license;
pub mod preview;
pub mod repo_detect;
pub mod version;
pub mod version_update;

use clap::Parser;

pub fn run_cli() -> Result<(), errors::AppError> {
    let cli = cli::Cli::parse();
    let ctx = context::AppContext::load(&cli)?;

    match &cli.command {
        cli::Commands::Init(args) => commands::init::run(&ctx, args),
        cli::Commands::Release(args) => commands::release::run(&ctx, args),
        cli::Commands::Doctor(args) => commands::doctor::run(&ctx, args),
        cli::Commands::Template(args) => commands::template::run(&ctx, args),
    }
}

pub fn main_with_exit() {
    if let Err(err) = run_cli() {
        eprintln!("error: {err}");
        std::process::exit(err.exit_code());
    }
}
