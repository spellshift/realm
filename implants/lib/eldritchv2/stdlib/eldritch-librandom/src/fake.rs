use super::RandomLibrary;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::eldritch_library_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(RandomLibrary)]
pub struct RandomLibraryFake;

impl RandomLibrary for RandomLibraryFake {
    fn bool(&self) -> Result<bool, String> {
        Ok(true)
    }

    fn bytes(&self, len: i64) -> Result<Vec<u8>, String> {
        Ok(vec![0; len as usize])
    }

    fn int(&self, min: i64, _max: i64) -> Result<i64, String> {
        Ok(min)
    }

    fn string(&self, len: i64, _charset: Option<String>) -> Result<String, String> {
        Ok("a".repeat(len as usize))
    }

    fn uuid(&self) -> Result<String, String> {
        Ok(String::from("00000000-0000-0000-0000-000000000000"))
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_random_fake() {
        let rnd = RandomLibraryFake::default();
        assert!(rnd.bool().unwrap());
        assert_eq!(rnd.int(10, 20).unwrap(), 10);
    }

    #[test]
    fn test_bytes_fake() {
        let rnd = RandomLibraryFake::default();
        let b = rnd.bytes(5).unwrap();
        assert_eq!(b.len(), 5);
        assert_eq!(b, vec![0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_string_fake() {
        let rnd = RandomLibraryFake::default();
        let s = rnd.string(5, None).unwrap();
        assert_eq!(s, "aaaaa");
    }

    #[test]
    fn test_uuid_fake() {
        let rnd = RandomLibraryFake::default();
        let u = rnd.uuid().unwrap();
        assert_eq!(u, "00000000-0000-0000-0000-000000000000");
    }
}
