use crate::inter::{Context, ContextMode};
use anyhow::Result;

pub fn main() -> Result<()> {
    let ctx = Context::new(ContextMode::Check, false, &[], false)?;

    starlark_lsp::server::stdio_server(ctx)?;

    Ok(())
}
