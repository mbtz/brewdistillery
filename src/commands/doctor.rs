use crate::cli::DoctorArgs;
use crate::context::AppContext;
use crate::errors::AppError;

pub fn run(_ctx: &AppContext, _args: &DoctorArgs) -> Result<(), AppError> {
    Ok(())
}
