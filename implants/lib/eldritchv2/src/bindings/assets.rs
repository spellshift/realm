use eldritch_macros::{eldritch_library, eldritch_library_impl, eldritch_method};
use alloc::string::String;
use alloc::vec::Vec;

#[eldritch_library("assets")]
pub trait AssetsLibrary {
    #[eldritch_method]
    fn copy(&self, src: String, dst: String) -> Result<(), String>;

    #[eldritch_method]
    fn list(&self) -> Result<Vec<String>, String>;

    #[eldritch_method]
    fn read_binary(&self, src: String) -> Result<Vec<u8>, String>;

    #[eldritch_method]
    fn read(&self, src: String) -> Result<String, String>;
}

#[cfg(feature = "fake_bindings")]
#[derive(Default, Debug)]
#[eldritch_library_impl(AssetsLibrary)]
pub struct AssetsLibraryFake;

#[cfg(feature = "fake_bindings")]
impl AssetsLibrary for AssetsLibraryFake {
    fn copy(&self, _src: String, _dst: String) -> Result<(), String> {
        Ok(())
    }

    fn list(&self) -> Result<Vec<String>, String> {
        let mut v = Vec::new();
        v.push(String::from("fake_asset"));
        Ok(v)
    }

    fn read_binary(&self, _src: String) -> Result<Vec<u8>, String> {
        Ok(Vec::from([0xDE, 0xAD, 0xBE, 0xEF]))
    }

    fn read(&self, _src: String) -> Result<String, String> {
        Ok(String::from("fake content"))
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_assets_fake() {
        let assets = AssetsLibraryFake::default();
        assets.copy("src".into(), "dst".into()).unwrap();
        assert!(!assets.list().unwrap().is_empty());
        assert_eq!(assets.read_binary("src".into()).unwrap(), vec![0xDE, 0xAD, 0xBE, 0xEF]);
        assert_eq!(assets.read("src".into()).unwrap(), "fake content");
    }
}
