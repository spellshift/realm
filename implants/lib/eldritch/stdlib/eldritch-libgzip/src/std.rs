use super::GzipLibrary;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::eldritch_library_impl;

pub mod compress_file_impl;
pub mod compress_impl;
pub mod decompress_file_impl;
pub mod decompress_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(GzipLibrary)]
pub struct StdGzipLibrary;

impl GzipLibrary for StdGzipLibrary {
    fn compress(&self, data: Vec<u8>) -> Result<Vec<u8>, String> {
        compress_impl::compress_bytes(data)
    }

    fn compress_file(&self, src: String, dst: String) -> Result<(), String> {
        compress_file_impl::compress_file(src, dst)
    }

    fn decompress(&self, data: Vec<u8>) -> Result<Vec<u8>, String> {
        decompress_impl::decompress_bytes(data)
    }

    fn decompress_file(&self, src: String, dst: String) -> Result<(), String> {
        decompress_file_impl::decompress_file(src, dst)
    }
}
