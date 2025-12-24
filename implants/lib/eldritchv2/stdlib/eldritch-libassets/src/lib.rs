extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;
#[cfg(feature = "stdlib")]
pub mod std;


#[eldritch_library("assets")]
/// The `assets` library provides access to files embedded directly within the agent binary.
///
/// This allows you to:
/// - Deploy tools or payloads without downloading them from the network.
/// - Read embedded configuration or scripts.
/// - List available embedded assets.
///
/// **Note**: Asset paths are typically relative to the embedding root (e.g., `sliver/agent-x64`).
pub trait AssetsLibrary {
    #[eldritch_method]
    /// Reads the content of an embedded asset as a list of bytes.
    ///
    /// **Parameters**
    /// - `name` (`str`): The name/path of the asset to read.
    ///
    /// **Returns**
    /// - `List<int>`: The asset content as a list of bytes (u8).
    ///
    /// **Errors**
    /// - Returns an error string if the asset does not exist.
    fn read_binary(&self, name: String) -> Result<Vec<u8>, String>;

    #[eldritch_method]
    /// Reads the content of an embedded asset as a UTF-8 string.
    ///
    /// **Parameters**
    /// - `name` (`str`): The name/path of the asset to read.
    ///
    /// **Returns**
    /// - `str`: The asset content as a string.
    ///
    /// **Errors**
    /// - Returns an error string if the asset does not exist or contains invalid UTF-8 data.
    fn read(&self, name: String) -> Result<String, String>;

    #[eldritch_method]
    /// Copies an embedded asset to a destination path on the disk.
    ///
    /// **Parameters**
    /// - `src` (`str`): The name/path of the source asset.
    /// - `dest` (`str`): The destination file path on the local system.
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if the asset does not exist or the file cannot be written (e.g., permission denied).
    fn copy(&self, src: String, dest: String) -> Result<(), String>;

    #[eldritch_method]
    /// Returns a list of all available asset names.
    ///
    /// **Returns**
    /// - `List<str>`: A list of asset names available in the agent.
    ///
    /// **Errors**
    /// - Returns an error string if the asset list cannot be retrieved.
    fn list(&self) -> Result<Vec<String>, String>;
}
