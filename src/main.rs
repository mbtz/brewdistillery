use brewdistillery::cli::{Cli, Commands};
use brewdistillery::context::AppContext;
use brewdistillery::errors::AppError;
use clap::Parser;

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(err.exit_code());
    }
}

fn run() -> Result<(), AppError> {
    let cli = Cli::parse();
    let ctx = AppContext::load(&cli)?;

    match &cli.command {
        Commands::Init(args) => brewdistillery::commands::init::run(&ctx, args),
        Commands::Release(args) => brewdistillery::commands::release::run(&ctx, args),
        Commands::Doctor(args) => brewdistillery::commands::doctor::run(&ctx, args),
    }
}
