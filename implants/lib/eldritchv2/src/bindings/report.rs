use eldritch_macros::{eldritch_library, eldritch_library_impl, eldritch_method};
use crate::ast::Value;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

#[eldritch_library("report")]
pub trait ReportLibrary {
    #[eldritch_method]
    fn file(&self, path: String) -> Result<(), String>;

    #[eldritch_method]
    fn process_list(&self, list: Vec<BTreeMap<String, Value>>) -> Result<(), String>;

    #[eldritch_method]
    fn ssh_key(&self, username: String, key: String) -> Result<(), String>;

    #[eldritch_method]
    fn user_password(&self, username: String, password: String) -> Result<(), String>;
}

#[cfg(feature = "fake_bindings")]
#[derive(Default, Debug)]
#[eldritch_library_impl(ReportLibrary)]
pub struct ReportLibraryFake;

#[cfg(feature = "fake_bindings")]
impl ReportLibrary for ReportLibraryFake {
    fn file(&self, _path: String) -> Result<(), String> { Ok(()) }

    fn process_list(&self, _list: Vec<BTreeMap<String, Value>>) -> Result<(), String> { Ok(()) }

    fn ssh_key(&self, _username: String, _key: String) -> Result<(), String> { Ok(()) }

    fn user_password(&self, _username: String, _password: String) -> Result<(), String> { Ok(()) }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_report_fake() {
        let report = ReportLibraryFake::default();
        report.file("path".into()).unwrap();
        report.process_list(vec![]).unwrap();
        report.ssh_key("u".into(), "k".into()).unwrap();
        report.user_password("u".into(), "p".into()).unwrap();
    }
}
