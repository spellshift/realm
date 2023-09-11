mod aes_encrypt_file_impl;
mod aes_decrypt_file_impl;
mod hash_file_impl;

use allocative::Allocative;
use derive_more::Display;

use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::values::none::NoneType;
use starlark::values::starlark_value;
use starlark::values::{StarlarkValue, Value, Heap, dict::Dict, UnpackValue, ValueLike, ProvidesStaticType};
use starlark::{starlark_simple_value, starlark_module};

use serde::{Serialize,Serializer};

#[derive(Copy, Clone, Debug, PartialEq, Display, ProvidesStaticType, Allocative)]
#[display(fmt = "CryptoLibrary")]
pub struct CryptoLibrary();
starlark_simple_value!(CryptoLibrary);

#[allow(non_upper_case_globals)]
#[starlark_value(type = "sys_library")]
impl<'v> StarlarkValue<'v> for CryptoLibrary {

    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new();
        RES.methods(methods)
    }
}

impl Serialize for CryptoLibrary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_none()
    }
}

impl<'v> UnpackValue<'v> for CryptoLibrary {
    fn expected() -> String {
        CryptoLibrary::get_type_value_static().as_str().to_owned()
    }

    fn unpack_value(value: Value<'v>) -> Option<Self> {
        Some(*value.downcast_ref::<CryptoLibrary>().unwrap())
    }
}

// This is where all of the "crypto.X" impl methods are bound
#[starlark_module]
fn methods(builder: &mut MethodsBuilder) {
    fn aes_encrypt_file<'v>(this: CryptoLibrary, src: String, dst: String, key: String) -> anyhow::Result<NoneType> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        aes_encrypt_file_impl::encrypt_file(src, dst, key)?;
        Ok(NoneType{})
    }
    fn aes_decrypt_file<'v>(this: CryptoLibrary, src: String, dst: String, key: String) -> anyhow::Result<NoneType> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        aes_decrypt_file_impl::decrypt_file(src, dst, key)?;
        Ok(NoneType{})
    }
    fn hash_file<'v>(this: CryptoLibrary, file: String, algo: String) -> anyhow::Result<String> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        hash_file_impl::hash_file(file, algo)
    }
}
