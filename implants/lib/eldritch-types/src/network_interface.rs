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
#[display(fmt = "name: {}, mac: {}, ips: {:?}", name, mac, ips)]
pub struct NetworkInterface {
    pub name: String,
    pub mac: String,
    pub ips: Vec<String>,
}
starlark_simple_value!(NetworkInterface);

#[allow(non_upper_case_globals)]
#[starlark_value(type = "network_interface")]
impl<'v> StarlarkValue<'v> for NetworkInterface {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new();
        RES.methods(methods)
    }
}

impl<'v> UnpackValue<'v> for NetworkInterface {
    fn expected() -> String {
        NetworkInterface::get_type_value_static().as_str().to_owned()
    }

    fn unpack_value(value: Value<'v>) -> Option<Self> {
        let tmp = value.downcast_ref::<NetworkInterface>().unwrap();
        Some(NetworkInterface { 
            name: tmp.name.clone(), 
            mac: tmp.mac.clone(), 
            ips: tmp.ips.clone(),
        })
    }
}


#[starlark_module]
fn methods(builder: &mut MethodsBuilder) {
    fn name(this: NetworkInterface) -> anyhow::Result<String> {
        Ok(this.name)
    }
    fn mac(this: NetworkInterface) -> anyhow::Result<String> {
        Ok(this.mac)
    }
    fn ips(this: NetworkInterface) -> anyhow::Result<Vec<String>> {
        Ok(this.ips)
    }
}