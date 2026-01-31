use super::RandomLibrary;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::eldritch_library_impl;

pub mod bool_impl;
pub mod bytes_impl;
pub mod int_impl;
pub mod string_impl;
pub mod uuid_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(RandomLibrary)]
pub struct StdRandomLibrary;

impl RandomLibrary for StdRandomLibrary {
    fn bool(&self) -> Result<bool, String> {
        bool_impl::bool()
    }

    fn bytes(&self, len: i64) -> Result<Vec<u8>, String> {
        bytes_impl::bytes(len)
    }

    fn int(&self, min: i64, max: i64) -> Result<i64, String> {
        int_impl::int(min, max)
    }

    fn string(&self, len: i64, charset: Option<String>) -> Result<String, String> {
        string_impl::string(len, charset)
    }

    fn uuid(&self) -> Result<String, String> {
        uuid_impl::uuid()
    }
}
