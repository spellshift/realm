use super::ReportLibrary;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(ReportLibrary)]
pub struct ReportLibraryFake;

impl ReportLibrary for ReportLibraryFake {
    fn file(&self, _path: String) -> Result<(), String> {
        Ok(())
    }

    fn process_list(&self, _list: Vec<BTreeMap<String, Value>>) -> Result<(), String> {
        Ok(())
    }

    fn ssh_key(&self, _username: String, _key: String) -> Result<(), String> {
        Ok(())
    }

    fn user_password(&self, _username: String, _password: String) -> Result<(), String> {
        Ok(())
    }

    fn ntlm_hash(&self, _username: String, _hash: String) -> Result<(), String> {
        Ok(())
    }

    fn screenshot(&self) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_report_fake() {
        let report = ReportLibraryFake;
        report.file("path".into()).unwrap();
    }
}
