extern crate alloc;
use alloc::string::String;
use eldritch_macros::{eldritch_deprecated, eldritch_library, eldritch_method};

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
    fn to_epoch(&self, input: String, format: String) -> Result<i64, String>;

    #[eldritch_method]
    #[eldritch_deprecated(
        "time.format_to_epoch has been deprecated and will be removed in a future version of eldritch. Please use time.to_epoch instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."
    )]
    /// Deprecated; use `time.to_epoch` instead.
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
    fn format(&self, input: i64, format: String) -> Result<String, String>;

    #[eldritch_method]
    #[eldritch_deprecated(
        "time.format_to_readable has been deprecated and will be removed in a future version of eldritch. Please use time.format instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."
    )]
    /// Deprecated; use `time.format` instead.
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
    /// - `secs` (`float`): Seconds to sleep.
    ///
    /// **Returns**
    /// - `None`
    fn sleep(&self, secs: f64) -> Result<(), String>;
}
