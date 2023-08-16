use allocative_derive::Allocative;
use derive_more::Display;
use starlark::environment::{MethodsBuilder, MethodsStatic, Methods};
use starlark::values::{UnpackValue, Value, ValueLike};
use starlark::{starlark_simple_value, values::StarlarkValue};
use starlark_derive::NoSerialize;
use starlark_derive::starlark_module;
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
#[starlark_value(type = "command_output_type")]
impl<'v> StarlarkValue<'v> for CommandOutput {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new();
        RES.methods(methods)
    }
}

impl<'v> UnpackValue<'v> for CommandOutput {
    fn expected() -> String {
        CommandOutput::get_type_value_static().as_str().to_owned()
    }

    fn unpack_value(value: Value<'v>) -> Option<Self> {
        let tmp = value.downcast_ref::<CommandOutput>().unwrap();
        Some(CommandOutput { 
            stdout: tmp.stdout.clone(), 
            stderr: tmp.stderr.clone(), 
            status: tmp.status 
        })
    }
}


#[starlark_module]
fn methods(builder: &mut MethodsBuilder) {
    fn stdout(this: CommandOutput) -> anyhow::Result<String> {
        Ok(this.stdout)
    }
    fn stderr(this: CommandOutput) -> anyhow::Result<String> {
        Ok(this.stderr)
    }
    fn status(this: CommandOutput) -> anyhow::Result<i32> {
        Ok(this.status)
    }
}