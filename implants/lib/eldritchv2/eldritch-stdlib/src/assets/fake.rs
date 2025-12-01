
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::eldritch_library_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(AssetsLibrary)]
pub struct AssetsLibraryFake;

impl AssetsLibrary for AssetsLibraryFake {
    fn get(&self, name: String) -> Result<Vec<u8>, String> {
        Ok(name.into_bytes())
    }

    fn list(&self) -> Result<Vec<String>, String> {
        Ok(vec![String::from("asset1"), String::from("asset2")])
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {


    #[test]
    fn test_assets_fake() {
        let assets = AssetsLibraryFake::default();
        assert_eq!(assets.list().unwrap().len(), 2);
    }
}
