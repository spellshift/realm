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
#[display(fmt = "pid:{},ppid:{},status:{},name:{},path:{},username:{},command:{},cwd:{},environ:{}", pid, ppid, status, name, path, username, command, cwd, environ)]
pub struct Proc {
    pub pid: u32,
    pub ppid: u32,
    pub status: String,
    pub name: String,
    pub path: String,
    pub username: String,
    pub command: String,
    pub cwd: String,
    pub environ: String,
}

starlark_simple_value!(Proc);

#[allow(non_upper_case_globals)]
#[starlark_value(type = "proc")]
impl<'v> StarlarkValue<'v> for Proc {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new();
        RES.methods(methods)
    }
}

impl<'v> UnpackValue<'v> for Proc {
    fn expected() -> String {
        Proc::get_type_value_static().as_str().to_owned()
    }

    fn unpack_value(value: Value<'v>) -> Option<Self> {
        let tmp = value.downcast_ref::<Proc>().unwrap();
        Some(Proc { 
            pid: tmp.pid.clone(), 
            ppid: tmp.ppid.clone(), 
            status: tmp.status.clone(),
            name: tmp.name.clone(),
            path: tmp.path.clone(),
            username: tmp.username.clone(),
            command: tmp.command.clone(),
            cwd: tmp.cwd.clone(), 
            environ: tmp.command.clone(), 
        })
    }
}


#[starlark_module]
fn methods(builder: &mut MethodsBuilder) {
    fn pid(this: Proc) -> anyhow::Result<u32> {
        Ok(this.pid)
    }
    fn ppid(this: Proc) -> anyhow::Result<u32> {
        Ok(this.ppid)
    }
    fn status(this: Proc) -> anyhow::Result<String> {
        Ok(this.status)
    }
    fn name(this: Proc) -> anyhow::Result<String> {
        Ok(this.name)
    }
    fn path(this: Proc) -> anyhow::Result<String> {
        Ok(this.path)
    }
    fn username(this: Proc) -> anyhow::Result<String> {
        Ok(this.username)
    }
    fn command(this: Proc) -> anyhow::Result<String> {
        Ok(this.command)
    }
    fn cwd(this: Proc) -> anyhow::Result<String> {
        Ok(this.cwd)
    }
    fn environ(this: Proc) -> anyhow::Result<String> {
        Ok(this.environ)
    }
}