use crate::cli::ReleaseArgs;
use crate::context::AppContext;
use crate::errors::AppError;
use crate::version::resolve_version_tag;

pub fn run(_ctx: &AppContext, _args: &ReleaseArgs) -> Result<(), AppError> {
    let _resolved = resolve_version_tag(_args.version.as_deref(), _args.tag.as_deref())?;
    Ok(())
}
