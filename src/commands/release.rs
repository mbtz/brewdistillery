use crate::cli::ReleaseArgs;
use crate::context::AppContext;
use crate::errors::AppError;

pub fn run(_ctx: &AppContext, _args: &ReleaseArgs) -> Result<(), AppError> {
    Ok(())
}
