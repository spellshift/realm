extern crate alloc;
use alloc::string::String;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("time")]
/// The `time` library provides time measurement, formatting, and sleep capabilities.
pub trait TimeLibrary {
    #[eldritch_method]
    /// Converts a formatted time string to a Unix timestamp (epoch seconds).
    ///
    /// **Parameters**
    /// - `input` (`str`): The time string (e.g., "2023-01-01 12:00:00").
    /// - `format` (`str`): The format string (e.g., "%Y-%m-%d %H:%M:%S").
    ///
    /// **Returns**
    /// - `int`: The timestamp.
    ///
    /// **Errors**
    /// - Returns an error string if parsing fails.
    fn format_to_epoch(&self, input: String, format: String) -> Result<i64, String>;

    #[eldritch_method]
    /// Converts a Unix timestamp to a readable string.
    ///
    /// **Parameters**
    /// - `input` (`int`): The timestamp (epoch seconds).
    /// - `format` (`str`): The desired output format.
    ///
    /// **Returns**
    /// - `str`: The formatted time string.
    fn format_to_readable(&self, input: i64, format: String) -> Result<String, String>;

    #[eldritch_method]
    /// Returns the current time as a Unix timestamp.
    ///
    /// **Returns**
    /// - `int`: Current epoch seconds.
    fn now(&self) -> Result<i64, String>;

    #[eldritch_method]
    /// Pauses execution for the specified number of seconds.
    ///
    /// **Parameters**
    /// - `secs` (`int`): Seconds to sleep.
    ///
    /// **Returns**
    /// - `None`
    fn sleep(&self, secs: i64) -> Result<(), String>;
}
