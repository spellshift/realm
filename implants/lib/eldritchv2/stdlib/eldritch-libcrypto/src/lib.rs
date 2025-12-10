extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("crypto")]
/// The `crypto` library provides cryptographic primitives, hashing, encoding, and JSON handling utilities.
///
/// It supports:
/// - AES encryption and decryption.
/// - Hashing (MD5, SHA1, SHA256) for data and files.
/// - Base64 encoding and decoding.
/// - JSON serialization and deserialization.
pub trait CryptoLibrary {
    #[eldritch_method]
    /// Decrypts data using AES (CBC mode).
    ///
    /// **Parameters**
    /// - `key` (`Bytes`): The decryption key (must be 16, 24, or 32 bytes).
    /// - `iv` (`Bytes`): The initialization vector (must be 16 bytes).
    /// - `data` (`Bytes`): The encrypted data to decrypt.
    ///
    /// **Returns**
    /// - `Bytes`: The decrypted data.
    ///
    /// **Errors**
    /// - Returns an error string if decryption fails (e.g., invalid padding, incorrect key length).
    fn aes_decrypt(&self, key: Vec<u8>, iv: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, String>;

    #[eldritch_method]
    /// Encrypts data using AES (CBC mode).
    ///
    /// **Parameters**
    /// - `key` (`Bytes`): The encryption key (must be 16, 24, or 32 bytes).
    /// - `iv` (`Bytes`): The initialization vector (must be 16 bytes).
    /// - `data` (`Bytes`): The data to encrypt.
    ///
    /// **Returns**
    /// - `Bytes`: The encrypted data.
    ///
    /// **Errors**
    /// - Returns an error string if encryption fails (e.g., incorrect key length).
    fn aes_encrypt(&self, key: Vec<u8>, iv: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, String>;

    #[eldritch_method]
    /// Decrypts a file using AES.
    ///
    /// **Parameters**
    /// - `src` (`str`): The source file path.
    /// - `dst` (`str`): The destination file path.
    /// - `key` (`str`): The decryption key.
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if decryption fails or file operations fail.
    fn aes_decrypt_file(&self, src: String, dst: String, key: String) -> Result<(), String>;

    #[eldritch_method]
    /// Encrypts a file using AES.
    ///
    /// **Parameters**
    /// - `src` (`str`): The source file path.
    /// - `dst` (`str`): The destination file path.
    /// - `key` (`str`): The encryption key.
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if encryption fails or file operations fail.
    fn aes_encrypt_file(&self, src: String, dst: String, key: String) -> Result<(), String>;

    #[eldritch_method]
    /// Calculates the MD5 hash of the provided data.
    ///
    /// **Parameters**
    /// - `data` (`Bytes`): The input data.
    ///
    /// **Returns**
    /// - `str`: The hexadecimal representation of the hash.
    ///
    /// **Errors**
    /// - Returns an error string if hashing fails.
    fn md5(&self, data: Vec<u8>) -> Result<String, String>;

    #[eldritch_method]
    /// Calculates the SHA1 hash of the provided data.
    ///
    /// **Parameters**
    /// - `data` (`Bytes`): The input data.
    ///
    /// **Returns**
    /// - `str`: The hexadecimal representation of the hash.
    ///
    /// **Errors**
    /// - Returns an error string if hashing fails.
    fn sha1(&self, data: Vec<u8>) -> Result<String, String>;

    #[eldritch_method]
    /// Calculates the SHA256 hash of the provided data.
    ///
    /// **Parameters**
    /// - `data` (`Bytes`): The input data.
    ///
    /// **Returns**
    /// - `str`: The hexadecimal representation of the hash.
    ///
    /// **Errors**
    /// - Returns an error string if hashing fails.
    fn sha256(&self, data: Vec<u8>) -> Result<String, String>;

    #[eldritch_method]
    /// Calculates the hash of a file on disk.
    ///
    /// **Parameters**
    /// - `file` (`str`): The path to the file.
    /// - `algo` (`str`): The hashing algorithm to use ("MD5", "SHA1", "SHA256", "SHA512").
    ///
    /// **Returns**
    /// - `str`: The hexadecimal representation of the hash.
    ///
    /// **Errors**
    /// - Returns an error string if the file cannot be read or the algorithm is not supported.
    fn hash_file(&self, file: String, algo: String) -> Result<String, String>;

    #[eldritch_method]
    /// Encodes a string to Base64.
    ///
    /// **Parameters**
    /// - `content` (`str`): The string content to encode.
    /// - `encode_type` (`Option<str>`): The encoding variant. Valid options:
    ///   - "STANDARD" (default)
    ///   - "STANDARD_NO_PAD"
    ///   - "URL_SAFE"
    ///   - "URL_SAFE_NO_PAD"
    ///
    /// **Returns**
    /// - `str`: The Base64 encoded string.
    ///
    /// **Errors**
    /// - Returns an error string if the encoding type is invalid.
    fn encode_b64(&self, content: String, encode_type: Option<String>) -> Result<String, String>;

    #[eldritch_method]
    /// Decodes a Base64 encoded string.
    ///
    /// **Parameters**
    /// - `content` (`str`): The Base64 string to decode.
    /// - `encode_type` (`Option<str>`): The decoding variant (matches encoding options).
    ///   - "STANDARD" (default)
    ///   - "STANDARD_NO_PAD"
    ///   - "URL_SAFE"
    ///   - "URL_SAFE_NO_PAD"
    ///
    /// **Returns**
    /// - `str`: The decoded string.
    ///
    /// **Errors**
    /// - Returns an error string if decoding fails or the variant is invalid.
    fn decode_b64(&self, content: String, encode_type: Option<String>) -> Result<String, String>;

    #[eldritch_method]
    /// Checks if a string is valid JSON.
    ///
    /// **Parameters**
    /// - `content` (`str`): The string to check.
    ///
    /// **Returns**
    /// - `bool`: `True` if valid JSON, `False` otherwise.
    fn is_json(&self, content: String) -> Result<bool, String>;

    #[eldritch_method]
    /// Parses a JSON string into an Eldritch value (Dict, List, etc.).
    ///
    /// **Parameters**
    /// - `content` (`str`): The JSON string.
    ///
    /// **Returns**
    /// - `Value`: The parsed value.
    ///
    /// **Errors**
    /// - Returns an error string if the JSON is invalid.
    fn from_json(&self, content: String) -> Result<Value, String>;

    #[eldritch_method]
    /// Serializes an Eldritch value into a JSON string.
    ///
    /// **Parameters**
    /// - `content` (`Value`): The value to serialize.
    ///
    /// **Returns**
    /// - `str`: The JSON string representation.
    ///
    /// **Errors**
    /// - Returns an error string if serialization fails (e.g., circular references, unsupported types).
    fn to_json(&self, content: Value) -> Result<String, String>;
}
