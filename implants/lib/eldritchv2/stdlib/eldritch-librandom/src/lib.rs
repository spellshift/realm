extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("random")]
/// The `random` library provides cryptographically secure random value generation.
pub trait RandomLibrary {
    #[eldritch_method]
    /// Generates a random boolean value.
    ///
    /// **Returns**
    /// - `bool`: True or False.
    fn bool(&self) -> Result<bool, String>;

    #[eldritch_method]
    /// Generates a list of random bytes.
    ///
    /// **Parameters**
    /// - `len` (`int`): Number of bytes to generate.
    ///
    /// **Returns**
    /// - `List<int>`: The random bytes.
    fn bytes(&self, len: i64) -> Result<Vec<u8>, String>;

    #[eldritch_method]
    /// Generates a random integer within a range.
    ///
    /// **Parameters**
    /// - `min` (`int`): Minimum value (inclusive).
    /// - `max` (`int`): Maximum value (exclusive).
    ///
    /// **Returns**
    /// - `int`: The random integer.
    fn int(&self, min: i64, max: i64) -> Result<i64, String>;

    #[eldritch_method]
    /// Generates a random string.
    ///
    /// **Parameters**
    /// - `len` (`int`): Length of the string.
    /// - `charset` (`Option<str>`): Optional string of characters to use. If `None`, defaults to alphanumeric.
    ///
    /// **Returns**
    /// - `str`: The random string.
    fn string(&self, len: i64, charset: Option<String>) -> Result<String, String>;

    #[eldritch_method]
    /// Generates a random UUID (v4).
    ///
    /// **Returns**
    /// - `str`: The UUID string.
    fn uuid(&self) -> Result<String, String>;
}
