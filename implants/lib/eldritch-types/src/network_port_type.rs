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
#[display(fmt = "(ip: {}, port: {}, protocol: {}, status: {})", ip, port, protocol, status)]
pub struct NetworkPortType {
    pub ip: String,
    pub port: i32,
    pub protocol: String,
    pub status: String,
}
starlark_simple_value!(NetworkPortType);

#[allow(non_upper_case_globals)]
#[starlark_value(type = "network_port_type")]
impl<'v> StarlarkValue<'v> for NetworkPortType {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new();
        RES.methods(methods)
    }
}

impl<'v> UnpackValue<'v> for NetworkPortType {
    fn expected() -> String {
        NetworkPortType::get_type_value_static().as_str().to_owned()
    }

    fn unpack_value(value: Value<'v>) -> Option<Self> {
        let tmp = value.downcast_ref::<NetworkPortType>().unwrap();
        Some(NetworkPortType { 
            ip: tmp.ip.clone(), 
            port: tmp.port, 
            protocol: tmp.protocol.clone(),
            status: tmp.status.clone(),
        })
    }
}


#[starlark_module]
fn methods(builder: &mut MethodsBuilder) {
    fn ip(this: NetworkPortType) -> anyhow::Result<String> {
        Ok(this.ip)
    }
    fn port(this: NetworkPortType) -> anyhow::Result<i32> {
        Ok(this.port)
    }
    fn protocol(this: NetworkPortType) -> anyhow::Result<String> {
        Ok(this.protocol)
    }
    fn status(this: NetworkPortType) -> anyhow::Result<String> {
        Ok(this.status)
    }
}