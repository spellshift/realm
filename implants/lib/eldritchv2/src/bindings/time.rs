use eldritch_macros::{eldritch_library, eldritch_library_impl, eldritch_method};
use alloc::string::String;

#[eldritch_library("time")]
pub trait TimeLibrary {
    #[eldritch_method]
    fn format_to_epoch(&self, input: String, format: String) -> Result<i64, String>;

    #[eldritch_method]
    fn format_to_readable(&self, input: i64, format: String) -> Result<String, String>;

    #[eldritch_method]
    fn now(&self) -> Result<i64, String>;

    #[eldritch_method]
    fn sleep(&self, secs: i64) -> Result<(), String>;
}

#[cfg(feature = "fake_bindings")]
#[derive(Default, Debug)]
#[eldritch_library_impl(TimeLibrary)]
pub struct TimeLibraryFake;

#[cfg(feature = "fake_bindings")]
impl TimeLibrary for TimeLibraryFake {
    fn format_to_epoch(&self, _input: String, _format: String) -> Result<i64, String> {
        Ok(0)
    }

    fn format_to_readable(&self, _input: i64, _format: String) -> Result<String, String> {
        Ok(String::from("1970-01-01 00:00:00"))
    }

    fn now(&self) -> Result<i64, String> {
        Ok(1600000000)
    }

    fn sleep(&self, _secs: i64) -> Result<(), String> { Ok(()) }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_time_fake() {
        let time = TimeLibraryFake::default();
        assert_eq!(time.now().unwrap(), 1600000000);
        time.sleep(1).unwrap();
    }
}
