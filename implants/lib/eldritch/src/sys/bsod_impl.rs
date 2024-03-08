use anyhow::{anyhow, Result};
use bsod::bsod;
use starlark::values::none::NoneType;

pub fn bsod_impl() -> Result<NoneType> {
    bsod();
    Err(anyhow!(
        "Reached end of function. System should have crashed."
    ))
}
