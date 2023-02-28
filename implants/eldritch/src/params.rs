use derive_more::Display;
use starlark::environment::{MethodsBuilder, MethodsStatic, Methods};
use starlark::{starlark_simple_value, starlark_type, starlark_module};
use starlark::values::{StarlarkValue, ProvidesStaticType, Value, UnpackValue, ValueLike};
use serde::{Serialize,Serializer};

#[derive(Clone, Debug, PartialEq, Display, ProvidesStaticType)]
#[display(fmt = "Params")]
pub struct Params  {
    data: String
}
starlark_simple_value!(Params); 

// impl<'v, V: ValueLike<'v>> StarlarkValue<'v> for ParamsGen<V>
//     where Self: AnyLifetime<'v> {
//     starlark_type!("params");

//     // To implement methods which are work for both `One` and `FrozenOne`,
//     // use the `ValueLike` trait.
// }

// impl<'v> Freeze for Params<'v> {
//     type Frozen = FrozenParams;
//     fn freeze(self, freezer: &Freezer) -> anyhow::Result<Self::Frozen> {
//         Ok(ParamsGen{
//             data: self.data
//         })
//     } 
// }

impl<'v> StarlarkValue<'v> for Params {
    starlark_type!("params");

    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new();
        RES.methods(methods)
    } 
}

impl Serialize for Params {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_none()
    }
}

impl<'v> UnpackValue<'v> for Params {
    fn expected() -> String {
        Params::get_type_value_static().as_str().to_owned()
    }

    fn unpack_value(value: Value<'v>) -> Option<Self> {
        // value.downcast_ref::<Params>()
        Some(value.downcast_ref::<Params>().unwrap().clone())
    }
}

pub fn get(tome_params: String) -> Params {
    Params{data: tome_params}
}

#[starlark_module]
fn methods(builder: &mut MethodsBuilder) {
    fn get(this: Params, name: String) -> anyhow::Result<String> {
        let parsed: serde_json::Value = serde_json::from_str(this.data.as_str())?;
        let obj: serde_json::Map<String, serde_json::Value> = parsed.as_object().unwrap().clone();
        let ret_val = match obj.get(name.as_str()){
            Some(val) => val,
            None => return Err(anyhow::anyhow!("no such parameter: {}\n", name)),
        };
        let res = match ret_val.as_str() {
            Some(val) => val,
            None => return Err(anyhow::anyhow!("parameter is invalid type: {}\n", name)),
        };
        Ok(res.to_string())

    //    let map = match serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(this.data.as_str()){
    //     Ok(val) => val,
    //     Err(err) => return Err(anyhow::anyhow!("failed to parse parameter JSON: {}\n", err)),

    //     map.get(name)
    // };

        // let res: String = this.data[&name];
        // let res: String = match this.data.get(&name) {
        //     Some(val) => val.to_string().clone(),
        //     None => return Err(anyhow::anyhow!("No such parameter: {}\n", name)),
        // };
        // Ok(res)
    }
}