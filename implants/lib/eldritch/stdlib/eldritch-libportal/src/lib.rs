extern crate alloc;
use alloc::string::String;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("portal")]
/// The `portal` library provides bi-directional portal stream management.
pub trait PortalLibrary {
    #[eldritch_method]
    /// Opens a portal bi-directional stream.
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if failure occurs.
    fn create(&self) -> Result<(), String>;
}
