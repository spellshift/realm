use allocative_derive::Allocative;
use derive_more::Display;
use starlark::{starlark_simple_value, values::StarlarkValue};
use starlark_derive::NoSerialize;
use starlark_derive::starlark_value;
use starlark_derive::ProvidesStaticType;

#[derive(Clone, Debug, PartialEq, Eq, Display, ProvidesStaticType, NoSerialize, Allocative)]
#[display(fmt = "stdout: {}, stderr: {}, status: {}", stdout, stderr, status)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub status: i32,
}
starlark_simple_value!(CommandOutput);

#[allow(non_upper_case_globals)]
#[starlark_value(type = "command_output")]
impl<'v> StarlarkValue<'v> for CommandOutput {}
