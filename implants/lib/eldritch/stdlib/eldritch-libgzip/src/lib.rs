extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("gzip")]
/// The `gzip` library provides GZIP compression and decompression for byte buffers and files.
pub trait GzipLibrary {
    #[eldritch_method]
    /// Compresses byte data using GZIP.
    ///
    /// **Parameters**
    /// - `data` (`Bytes`): The raw bytes to compress.
    ///
    /// **Returns**
    /// - `Bytes`: The GZIP-compressed bytes.
    fn compress(&self, data: Vec<u8>) -> Result<Vec<u8>, String>;

    #[eldritch_method]
    /// Compresses a file or directory using GZIP.
    ///
    /// If `src` is a directory, it will be archived (tar) before compression.
    ///
    /// **Parameters**
    /// - `src` (`str`): The source file or directory path.
    /// - `dst` (`str`): The destination path for the compressed file (e.g., `archive.tar.gz`).
    ///
    /// **Returns**
    /// - `None`
    fn compress_file(&self, src: String, dst: String) -> Result<(), String>;

    #[eldritch_method]
    /// Decompresses GZIP-compressed byte data.
    ///
    /// **Parameters**
    /// - `data` (`Bytes`): The GZIP-compressed bytes.
    ///
    /// **Returns**
    /// - `Bytes`: The decompressed bytes.
    fn decompress(&self, data: Vec<u8>) -> Result<Vec<u8>, String>;

    #[eldritch_method]
    /// Decompresses a GZIP file.
    ///
    /// If the file is a tar archive, it will be extracted to the destination directory.
    ///
    /// **Parameters**
    /// - `src` (`str`): The source compressed file path.
    /// - `dst` (`str`): The destination path (file or directory).
    ///
    /// **Returns**
    /// - `None`
    fn decompress_file(&self, src: String, dst: String) -> Result<(), String>;
}
