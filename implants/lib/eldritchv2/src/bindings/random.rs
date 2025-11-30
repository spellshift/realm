use eldritch_macros::{eldritch_library, eldritch_library_impl, eldritch_method};
use alloc::string::String;

#[eldritch_library("random")]
pub trait RandomLibrary {
    #[eldritch_method]
    fn bool(&self) -> Result<bool, String>;

    #[eldritch_method]
    fn int(&self, min: i64, max: i64) -> Result<i64, String>;

    #[eldritch_method]
    fn string(&self, length: i64, charset: Option<String>) -> Result<String, String>;
}

#[cfg(feature = "fake_bindings")]
#[derive(Default, Debug)]
#[eldritch_library_impl(RandomLibrary)]
pub struct RandomLibraryFake;

#[cfg(feature = "fake_bindings")]
impl RandomLibrary for RandomLibraryFake {
    fn bool(&self) -> Result<bool, String> {
        Ok(true) // not random but deterministic for fake
    }

    fn int(&self, min: i64, _max: i64) -> Result<i64, String> {
        Ok(min)
    }

    fn string(&self, length: i64, _charset: Option<String>) -> Result<String, String> {
        use alloc::vec;
        let mut v = vec![0u8; length as usize];
        for i in 0..length as usize {
            v[i] = b'a';
        }
        Ok(String::from_utf8(v).unwrap())
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
        assert_eq!(rnd.string(5, None).unwrap().len(), 5);
    }
}
