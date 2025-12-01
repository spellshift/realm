
use eldritch_core::Value;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::eldritch_library_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(PivotLibrary)]
pub struct PivotLibraryFake;

impl PivotLibrary for PivotLibraryFake {
    fn list(&self) -> Result<Vec<BTreeMap<String, Value>>, String> {
        Ok(Vec::new())
    }

    fn start_tcp(&self, _bind_addr: String) -> Result<String, String> {
        Ok(String::from("pivot-id"))
    }

    fn stop(&self, _id: String) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {


    #[test]
    fn test_pivot_fake() {
        let pivot = PivotLibraryFake::default();
        assert_eq!(pivot.start_tcp("0.0.0.0:80".into()).unwrap(), "pivot-id");
    }
}
