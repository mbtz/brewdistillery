use crate::cli::InitArgs;
use crate::context::AppContext;
use crate::errors::AppError;

pub fn run(_ctx: &AppContext, _args: &InitArgs) -> Result<(), AppError> {
    Ok(())
}
