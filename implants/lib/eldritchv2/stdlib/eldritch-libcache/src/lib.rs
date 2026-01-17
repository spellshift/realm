#![allow(clippy::mutable_key_type)]
#![allow(unexpected_cfgs)]
extern crate alloc;

use alloc::string::String;
use eldritch_core::Value;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("cache")]
/// The `cache` library provides a thread-safe in-memory cache shared across interpreters.
pub trait CacheLibrary {
    #[eldritch_method]
    /// Retrieves a value from the cache.
    ///
    /// **Parameters**
    /// - `key` (`str`): The key to look up.
    ///
    /// **Returns**
    /// - `Value`: The value if found, or `Null` (or undefined behavior depending on caller, but returns Value).
    ///           Actually, eldritch `Value` usually has a `Null` variant or we can return `Value::Null` equivalent.
    fn get(&self, key: String) -> Result<Value, String>;

    #[eldritch_method]
    /// Sets a value in the cache.
    ///
    /// **Parameters**
    /// - `key` (`str`): The key to set.
    /// - `val` (`Value`): The value to store.
    ///
    /// **Returns**
    /// - `None`
    fn set(&self, key: String, val: Value) -> Result<(), String>;

    #[eldritch_method]
    /// Deletes a value from the cache.
    ///
    /// **Parameters**
    /// - `key` (`str`): The key to remove.
    ///
    /// **Returns**
    /// - `Value`: The removed value, or error/null if not found.
    fn delete(&self, key: String) -> Result<Value, String>;
}
