#![allow(clippy::mutable_key_type)]
extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
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
/// - Custom headers and query parameters.
pub trait HttpLibrary {
    #[eldritch_method]
    /// Downloads a file from a URL to a local path.
    ///
    /// **Parameters**
    /// - `uri` (`str`): The URL to download from.
    /// - `dst` (`str`): The local destination path.
    /// - `allow_insecure` (`Option<bool>`): If true, ignore SSL certificate verification.
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if the download fails.
    fn download(&self, uri: String, dst: String, allow_insecure: Option<bool>)
        -> Result<(), String>;

    #[eldritch_method]
    /// Performs an HTTP GET request.
    ///
    /// **Parameters**
    /// - `uri` (`str`): The target URL.
    /// - `query_params` (`Option<Dict<str, str>>`): Optional query parameters.
    /// - `headers` (`Option<Dict<str, str>>`): Optional custom HTTP headers.
    /// - `allow_insecure` (`Option<bool>`): If true, ignore SSL certificate verification.
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
        uri: String,
        query_params: Option<BTreeMap<String, String>>,
        headers: Option<BTreeMap<String, String>>,
        allow_insecure: Option<bool>,
    ) -> Result<BTreeMap<String, Value>, String>;

    #[eldritch_method]
    /// Performs an HTTP POST request.
    ///
    /// **Parameters**
    /// - `uri` (`str`): The target URL.
    /// - `body` (`Option<str>`): The request body.
    /// - `form` (`Option<Dict<str, str>>`): Form data (application/x-www-form-urlencoded).
    /// - `headers` (`Option<Dict<str, str>>`): Optional custom HTTP headers.
    /// - `allow_insecure` (`Option<bool>`): If true, ignore SSL certificate verification.
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
        uri: String,
        body: Option<String>,
        form: Option<BTreeMap<String, String>>,
        headers: Option<BTreeMap<String, String>>,
        allow_insecure: Option<bool>,
    ) -> Result<BTreeMap<String, Value>, String>;
}
