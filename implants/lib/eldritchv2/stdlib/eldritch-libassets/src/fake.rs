use super::AssetsLibrary;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::eldritch_library_impl;

#[derive(Debug)]
#[eldritch_library_impl(AssetsLibrary)]
pub struct FakeAssetsLibrary;

impl AssetsLibrary for FakeAssetsLibrary {
    fn read_binary(&self, _name: String) -> Result<Vec<u8>, String> {
        Ok(b"fake_binary_content".to_vec())
    }

    fn read(&self, _name: String) -> Result<String, String> {
        Ok("fake_text_content".to_string())
    }

    fn copy(&self, _src: String, _dest: String) -> Result<(), String> {
        Ok(())
    }

    fn list(&self) -> Result<Vec<String>, String> {
        Ok(alloc::vec!["fake_file.txt".to_string()])
    }
}
