use eldritch_core::{Interpreter, Value};
use eldritch_libcrypto::std::StdCryptoLibrary;
use std::io::Write;
use tempfile::NamedTempFile;

// Note: json and base64 functions seem to have been moved out of crypto library in v2 or not implemented yet?
// Checked lib.rs, only aes, md5, sha1, sha256, hash_file are present.
// The v1 tests had decode_b64, encode_b64, from_json, to_json.
// I should check if they are in another library or if they need to be added.
// For now, I will only test what is available in v2.

#[test]
fn test_crypto_hash_file() {
    let mut temp = NamedTempFile::new().unwrap();
    write!(temp, "hello world").unwrap();
    let path = temp.path().to_str().unwrap().replace("\\", "/");

    let lib = StdCryptoLibrary::default();
    let mut interp = Interpreter::new();
    interp.register_lib(lib);

    let code = format!("crypto.hash_file('{}', 'md5')", path);
    let res = interp.interpret(&code).unwrap();
    // md5 of "hello world" is 5eb63bbbe01eeed093cb22bb8f5acdc3
    assert_eq!(res, Value::String("5eb63bbbe01eeed093cb22bb8f5acdc3".to_string()));
}

#[test]
fn test_crypto_md5() {
    let lib = StdCryptoLibrary::default();
    let mut interp = Interpreter::new();
    interp.register_lib(lib);

    // md5 accepts bytes.
    // In eldritch, bytes can be created from list of ints or string maybe?
    // Let's assume bytes literals or conversion.
    // crypto.md5(b'hello')

    let res = interp.interpret("crypto.md5(b'hello')").unwrap();
    assert_eq!(res, Value::String("5d41402abc4b2a76b9719d911017c592".to_string()));
}
