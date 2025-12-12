
use alloc::string::String;
use eldritch_macros::eldritch_library_impl;
use super::TimeLibrary;

#[derive(Default, Debug)]
#[eldritch_library_impl(TimeLibrary)]
pub struct TimeLibraryFake;

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

    fn sleep(&self, _secs: i64) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {


    #[test]
    fn test_time_fake() {
        let time = TimeLibraryFake::default();
        assert_eq!(time.now().unwrap(), 1600000000);
    }
}
