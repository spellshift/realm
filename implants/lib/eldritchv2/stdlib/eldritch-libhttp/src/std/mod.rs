use super::HttpLibrary;
use alloc::collections::BTreeMap;
use alloc::string::String;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;

pub mod download_impl;
pub mod get_impl;
pub mod post_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(HttpLibrary)]
pub struct StdHttpLibrary;

impl HttpLibrary for StdHttpLibrary {
    fn download(
        &self,
        uri: String,
        dst: String,
        allow_insecure: Option<bool>,
    ) -> Result<(), String> {
        download_impl::download(uri, dst, allow_insecure)
    }

    fn get(
        &self,
        uri: String,
        query_params: Option<BTreeMap<String, String>>,
        headers: Option<BTreeMap<String, String>>,
        allow_insecure: Option<bool>,
    ) -> Result<BTreeMap<String, Value>, String> {
        get_impl::get(uri, query_params, headers, allow_insecure)
    }

    fn post(
        &self,
        uri: String,
        body: Option<String>,
        form: Option<BTreeMap<String, String>>,
        headers: Option<BTreeMap<String, String>>,
        allow_insecure: Option<bool>,
    ) -> Result<BTreeMap<String, Value>, String> {
        post_impl::post(uri, body, form, headers, allow_insecure)
    }
}
