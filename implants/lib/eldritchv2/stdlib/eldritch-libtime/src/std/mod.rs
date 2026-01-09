use super::TimeLibrary;
use alloc::string::String;
use eldritch_macros::eldritch_library_impl;

pub mod format_to_epoch_impl;
pub mod format_to_readable_impl;
pub mod now_impl;
pub mod sleep_impl;

#[derive(Debug, Default)]
#[eldritch_library_impl(TimeLibrary)]
pub struct StdTimeLibrary;

impl TimeLibrary for StdTimeLibrary {
    fn format_to_epoch(&self, input: String, format: String) -> Result<i64, String> {
        format_to_epoch_impl::format_to_epoch(input, format)
    }

    fn format_to_readable(&self, input: i64, format: String) -> Result<String, String> {
        format_to_readable_impl::format_to_readable(input, format)
    }

    fn now(&self) -> Result<i64, String> {
        now_impl::now()
    }

    fn sleep(&self, secs: i64) -> Result<(), String> {
        sleep_impl::sleep(secs)
    }
}
