use eldritch_macros::{eldritch_library, eldritch_library_impl, eldritch_method};
use crate::ast::Value;
use alloc::string::String;

#[eldritch_library("crypto")]
pub trait CryptoLibrary {
    #[eldritch_method]
    fn aes_decrypt_file(&self, src: String, dst: String, key: String) -> Result<(), String>;

    #[eldritch_method]
    fn aes_encrypt_file(&self, src: String, dst: String, key: String) -> Result<(), String>;

    #[eldritch_method]
    fn encode_b64(&self, content: String, encode_type: Option<String>) -> Result<String, String>;

    #[eldritch_method]
    fn decode_b64(&self, content: String, decode_type: Option<String>) -> Result<String, String>;

    #[eldritch_method]
    fn from_json(&self, content: String) -> Result<Value, String>;

    #[eldritch_method]
    fn is_json(&self, content: String) -> Result<bool, String>;

    #[eldritch_method]
    fn to_json(&self, content: Value) -> Result<String, String>;

    #[eldritch_method]
    fn hash_file(&self, file: String, algo: String) -> Result<String, String>;
}

#[cfg(feature = "fake_bindings")]
#[derive(Default, Debug)]
#[eldritch_library_impl(CryptoLibrary)]
pub struct CryptoLibraryFake;

#[cfg(feature = "fake_bindings")]
impl CryptoLibrary for CryptoLibraryFake {
    fn aes_decrypt_file(&self, _src: String, _dst: String, _key: String) -> Result<(), String> {
        Ok(())
    }

    fn aes_encrypt_file(&self, _src: String, _dst: String, _key: String) -> Result<(), String> {
        Ok(())
    }

    fn encode_b64(&self, content: String, _encode_type: Option<String>) -> Result<String, String> {
        // Just return dummy encoded string
        Ok(alloc::format!("b64({})", content))
    }

    fn decode_b64(&self, content: String, _decode_type: Option<String>) -> Result<String, String> {
        Ok(alloc::format!("decoded({})", content))
    }

    fn from_json(&self, _content: String) -> Result<Value, String> {
        // Return empty dict
        use alloc::collections::BTreeMap;
        use core::cell::RefCell;
        use alloc::rc::Rc;
        Ok(Value::Dictionary(Rc::new(RefCell::new(BTreeMap::new()))))
    }

    fn is_json(&self, content: String) -> Result<bool, String> {
        Ok(content.starts_with("{"))
    }

    fn to_json(&self, _content: Value) -> Result<String, String> {
        Ok(String::from("{}"))
    }

    fn hash_file(&self, _file: String, _algo: String) -> Result<String, String> {
        Ok(String::from("deadbeef"))
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_fake() {
        let crypto = CryptoLibraryFake::default();
        crypto.aes_decrypt_file("s".into(), "d".into(), "k".into()).unwrap();
        assert_eq!(crypto.encode_b64("foo".into(), None).unwrap(), "b64(foo)");
        assert!(crypto.is_json("{".into()).unwrap());
        assert_eq!(crypto.hash_file("f".into(), "md5".into()).unwrap(), "deadbeef");
    }
}
