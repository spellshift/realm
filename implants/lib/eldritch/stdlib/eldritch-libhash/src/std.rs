use super::HashLibrary;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::eldritch_library_impl;

pub mod file_impl;
pub mod md5_impl;
pub mod sha1_impl;
pub mod sha256_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(HashLibrary)]
pub struct StdHashLibrary;

impl HashLibrary for StdHashLibrary {
    fn file(&self, file: String, algo: String) -> Result<String, String> {
        file_impl::hash_file(file, algo)
    }

    fn md5(&self, data: Vec<u8>) -> Result<String, String> {
        md5_impl::md5(data)
    }

    fn sha1(&self, data: Vec<u8>) -> Result<String, String> {
        sha1_impl::sha1(data)
    }

    fn sha256(&self, data: Vec<u8>) -> Result<String, String> {
        sha256_impl::sha256(data)
    }
}
