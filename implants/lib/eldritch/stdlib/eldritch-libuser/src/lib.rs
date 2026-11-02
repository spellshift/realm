extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("user")]
/// The `user` library provides methods for inspecting user accounts on the system.
pub trait UserLibrary {
    #[eldritch_method]
    /// Returns information about the current user.
    ///
    /// **Returns**
    /// - `Dict`: User details (uid, gid, name, groups).
    fn info(&self) -> Result<BTreeMap<String, Value>, String>;

    #[eldritch_method]
    /// Lists all user accounts on the system.
    ///
    /// **Returns**
    /// - `List<Dict>`: List of user accounts.
    fn list(&self) -> Result<Vec<BTreeMap<String, Value>>, String>;
}
