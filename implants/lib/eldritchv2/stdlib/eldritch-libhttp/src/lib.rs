#![allow(clippy::mutable_key_type)]
extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("http")]
/// The `http` library enables the agent to make HTTP requests.
///
/// It supports:
/// - GET and POST requests.
/// - File downloading.
/// - Custom headers.
///
/// **Note**: TLS validation behavior depends on the underlying agent configuration and may not be exposed per-request in this version of the library (unlike v1 which had `allow_insecure` arg).
pub trait HttpLibrary {
    #[eldritch_method]
    /// Downloads a file from a URL to a local path.
    ///
    /// **Parameters**
    /// - `url` (`str`): The URL to download from.
    /// - `path` (`str`): The local destination path.
    /// - `insecure` (`Option<bool>`): If true, ignore SSL certificate verification (insecure).
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if the download fails.
    fn download(&self, url: String, path: String, insecure: Option<bool>) -> Result<(), String>;

    #[eldritch_method]
    /// Performs an HTTP GET request.
    ///
    /// **Parameters**
    /// - `url` (`str`): The target URL.
    /// - `headers` (`Option<Dict<str, str>>`): Optional custom HTTP headers.
    ///
    /// **Returns**
    /// - `Dict`: A dictionary containing the response:
    ///   - `status_code` (`int`): HTTP status code.
    ///   - `body` (`Bytes`): The response body.
    ///   - `headers` (`Dict<str, str>`): Response headers.
    ///
    /// **Errors**
    /// - Returns an error string if the request fails.
    fn get(
        &self,
        url: String,
        headers: Option<BTreeMap<String, String>>,
    ) -> Result<BTreeMap<String, Value>, String>;

    #[eldritch_method]
    /// Performs an HTTP POST request.
    ///
    /// **Parameters**
    /// - `url` (`str`): The target URL.
    /// - `body` (`Option<Bytes>`): The request body.
    /// - `headers` (`Option<Dict<str, str>>`): Optional custom HTTP headers.
    ///
    /// **Returns**
    /// - `Dict`: A dictionary containing the response:
    ///   - `status_code` (`int`): HTTP status code.
    ///   - `body` (`Bytes`): The response body.
    ///   - `headers` (`Dict<str, str>`): Response headers.
    ///
    /// **Errors**
    /// - Returns an error string if the request fails.
    fn post(
        &self,
        url: String,
        body: Option<Vec<u8>>,
        headers: Option<BTreeMap<String, String>>,
    ) -> Result<BTreeMap<String, Value>, String>;
}
