use super::GzipLibrary;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::eldritch_library_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(GzipLibrary)]
pub struct GzipLibraryFake;

impl GzipLibrary for GzipLibraryFake {
    fn compress(&self, data: Vec<u8>) -> Result<Vec<u8>, String> {
        let mut d = data;
        d.reverse();
        Ok(d)
    }

    fn compress_file(&self, _src: String, _dst: String) -> Result<(), String> {
        Ok(())
    }

    fn decompress(&self, data: Vec<u8>) -> Result<Vec<u8>, String> {
        let mut d = data;
        d.reverse();
        Ok(d)
    }

    fn decompress_file(&self, _src: String, _dst: String) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_gzip_fake() {
        let gz = GzipLibraryFake;
        let data = vec![1, 2, 3];
        let comp = gz.compress(data.clone()).unwrap();
        let decomp = gz.decompress(comp).unwrap();
        assert_eq!(data, decomp);
        assert!(gz.compress_file("src".into(), "dst".into()).is_ok());
        assert!(gz.decompress_file("src".into(), "dst".into()).is_ok());
    }
}
