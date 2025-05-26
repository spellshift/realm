mod aes_decrypt_file_impl;
mod aes_encrypt_file_impl;
mod decode_b64_impl;
mod encode_b64_impl;
mod from_json_impl;
mod hash_file_impl;
mod is_json_impl;
mod to_json_impl;

use starlark::environment::MethodsBuilder;
use starlark::starlark_module;
use starlark::values::none::NoneType;
use starlark::values::starlark_value;
use starlark::values::{Heap, Value};

/*
 * Define our library for this module.
 */
crate::eldritch_lib!(CryptoLibrary, "crypto_library");

/*
 * Below, we define starlark wrappers for all of our library methods.
 * The functions must be defined here to be present on our library.
 */
#[starlark_module]
#[rustfmt::skip]
#[allow(clippy::needless_lifetimes, clippy::type_complexity, clippy::too_many_arguments)]
fn methods(builder: &mut MethodsBuilder) {
    #[allow(unused_variables)]
    fn aes_encrypt_file<'v>(this: &CryptoLibrary, src: String, dst: String, key: String) -> anyhow::Result<NoneType> {
        aes_encrypt_file_impl::encrypt_file(src, dst, key)?;
        Ok(NoneType{})
    }

    #[allow(unused_variables)]
    fn aes_decrypt_file<'v>(this: &CryptoLibrary, src: String, dst: String, key: String) -> anyhow::Result<NoneType> {
        aes_decrypt_file_impl::decrypt_file(src, dst, key)?;
        Ok(NoneType{})
    }

    #[allow(unused_variables)]
    fn hash_file<'v>(this: &CryptoLibrary, file: String, algo: String) -> anyhow::Result<String> {
        hash_file_impl::hash_file(file, algo)
    }

    #[allow(unused_variables)]
    fn encode_b64<'v>(this: &CryptoLibrary, content: String, encode_type: Option<String>) -> anyhow::Result<String> {
        encode_b64_impl::encode_b64(content, encode_type)
    }

    #[allow(unused_variables)]
    fn decode_b64<'v>(this: &CryptoLibrary, content: String, encode_type: Option<String>) -> anyhow::Result<String> {
        decode_b64_impl::decode_b64(content, encode_type)
    }

    #[allow(unused_variables)]
    fn is_json<'v>(this: &CryptoLibrary, starlark_heap: &'v Heap, content: String) -> anyhow::Result<bool> {
        is_json_impl::is_json(content)
    }


    #[allow(unused_variables)]
    fn from_json<'v>(this: &CryptoLibrary, starlark_heap: &'v Heap, content: String) -> anyhow::Result<Value<'v>> {
        from_json_impl::from_json(starlark_heap, content)
    }

    #[allow(unused_variables)]
    fn to_json<'v>(this: &CryptoLibrary, content: Value) -> anyhow::Result<String> {
        to_json_impl::to_json(content)
    }
}
