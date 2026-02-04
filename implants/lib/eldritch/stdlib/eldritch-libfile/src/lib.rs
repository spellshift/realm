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

#[eldritch_library("file")]
/// The `file` library provides comprehensive filesystem operations.
///
/// It supports:
/// - reading and writing files (text and binary).
/// - file manipulation (copy, move, remove).
/// - directory operations (mkdir, list).
/// - compression and decompression (gzip).
/// - content searching and replacement.
pub trait FileLibrary {
    #[eldritch_method]
    /// Appends content to a file.
    ///
    /// If the file does not exist, it will be created.
    ///
    /// **Parameters**
    /// - `path` (`str`): The path to the file.
    /// - `content` (`str`): The string content to append.
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if the file cannot be opened or written to.
    fn append(&self, path: String, content: String) -> Result<(), String>;

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
    ///
    /// **Errors**
    /// - Returns an error string if the source doesn't exist or compression fails.
    fn compress(&self, src: String, dst: String) -> Result<(), String>;

    #[eldritch_method]
    /// Copies a file from source to destination.
    ///
    /// If the destination exists, it will be overwritten.
    ///
    /// **Parameters**
    /// - `src` (`str`): The source file path.
    /// - `dst` (`str`): The destination file path.
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if the source doesn't exist or copy fails.
    fn copy(&self, src: String, dst: String) -> Result<(), String>;

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
    ///
    /// **Errors**
    /// - Returns an error string if decompression fails.
    fn decompress(&self, src: String, dst: String) -> Result<(), String>;

    #[eldritch_method]
    /// Checks if a file or directory exists at the given path.
    ///
    /// **Parameters**
    /// - `path` (`str`): The path to check.
    ///
    /// **Returns**
    /// - `bool`: `True` if it exists, `False` otherwise.
    fn exists(&self, path: String) -> Result<bool, String>;

    #[eldritch_method]
    /// Follows a file (tail -f) and executes a callback function for each new line.
    ///
    /// This is useful for monitoring logs.
    ///
    /// **Parameters**
    /// - `path` (`str`): The file path to follow.
    /// - `fn` (`function(str)`): A callback function that takes a string (the new line) as an argument.
    ///
    /// **Returns**
    /// - `None` (This function may block indefinitely or until interrupted).
    ///
    /// **Errors**
    /// - Returns an error string if the file cannot be opened.
    fn follow(
        &self,
        interp: &mut eldritch_core::Interpreter,
        path: String,
        fn_val: Value,
    ) -> Result<(), String>; // fn is reserved

    #[eldritch_method]
    /// Checks if the path exists and is a directory.
    ///
    /// **Parameters**
    /// - `path` (`str`): The path to check.
    ///
    /// **Returns**
    /// - `bool`: `True` if it is a directory, `False` otherwise.
    fn is_dir(&self, path: String) -> Result<bool, String>;

    #[eldritch_method]
    /// Checks if the path exists and is a file.
    ///
    /// **Parameters**
    /// - `path` (`str`): The path to check.
    ///
    /// **Returns**
    /// - `bool`: `True` if it is a file, `False` otherwise.
    fn is_file(&self, path: String) -> Result<bool, String>;

    #[eldritch_method]
    /// Lists files and directories in the specified path.
    ///
    /// Supports globbing patterns (e.g., `/home/*/*.txt`).
    ///
    /// **Parameters**
    /// - `path` (`Option<str>`): The directory path or glob pattern. Defaults to current working directory.
    ///
    /// **Returns**
    /// - `List<Dict>`: A list of dictionaries containing file details:
    ///   - `file_name` (`str`)
    ///   - `absolute_path` (`str`)
    ///   - `size` (`int`)
    ///   - `owner` (`str`)
    ///   - `group` (`str`)
    ///   - `permissions` (`str`)
    ///   - `modified` (`str`)
    ///   - `type` (`str`: "File" or "Directory")
    ///
    /// **Errors**
    /// - Returns an error string if listing fails.
    fn list(&self, path: Option<String>) -> Result<Vec<BTreeMap<String, Value>>, String>;

    #[eldritch_method]
    /// Creates a new directory.
    ///
    /// **Parameters**
    /// - `path` (`str`): The directory path to create.
    /// - `parent` (`Option<bool>`): If `True`, creates parent directories as needed (like `mkdir -p`). Defaults to `False`.
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if creation fails.
    fn mkdir(&self, path: String, parent: Option<bool>) -> Result<(), String>;

    #[eldritch_method("move")]
    /// Moves or renames a file or directory.
    ///
    /// **Parameters**
    /// - `src` (`str`): The source path.
    /// - `dst` (`str`): The destination path.
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if the move fails.
    fn move_(&self, src: String, dst: String) -> Result<(), String>;

    #[eldritch_method]
    /// Returns the parent directory of the given path.
    ///
    /// **Parameters**
    /// - `path` (`str`): The file or directory path.
    ///
    /// **Returns**
    /// - `str`: The parent directory path.
    ///
    /// **Errors**
    /// - Returns an error string if the path is invalid or has no parent.
    fn parent_dir(&self, path: String) -> Result<String, String>;

    #[eldritch_method]
    /// Reads the entire content of a file as a string.
    ///
    /// Supports globbing; if multiple files match, reads the first one (or behavior may vary, usually reads specific file).
    /// *Note*: v1 docs say it errors if a directory matches.
    ///
    /// **Parameters**
    /// - `path` (`str`): The file path.
    ///
    /// **Returns**
    /// - `str`: The file content.
    ///
    /// **Errors**
    /// - Returns an error string if the file cannot be read or contains invalid UTF-8.
    fn read(&self, path: String) -> Result<String, String>;

    #[eldritch_method]
    /// Reads the entire content of a file as binary data.
    ///
    /// **Parameters**
    /// - `path` (`str`): The file path.
    ///
    /// **Returns**
    /// - `List<int>`: The file content as a list of bytes (u8).
    ///
    /// **Errors**
    /// - Returns an error string if the file cannot be read.
    fn read_binary(&self, path: String) -> Result<Vec<u8>, String>;

    #[eldritch_method]
    /// Returns the current working directory of the process.
    ///
    /// **Returns**
    /// - `Option<str>`: The current working directory path, or None if it cannot be determined.
    fn pwd(&self) -> Result<Option<String>, String>;

    #[eldritch_method]
    /// Deletes a file or directory recursively.
    ///
    /// **Parameters**
    /// - `path` (`str`): The path to remove.
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if removal fails.
    fn remove(&self, path: String) -> Result<(), String>;

    #[eldritch_method]
    /// Replaces the first occurrence of a regex pattern in a file with a replacement string.
    ///
    /// **Parameters**
    /// - `path` (`str`): The file path.
    /// - `pattern` (`str`): The regex pattern to match.
    /// - `value` (`str`): The replacement string.
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if the file cannot be modified or the regex is invalid.
    fn replace(&self, path: String, pattern: String, value: String) -> Result<(), String>;

    #[eldritch_method]
    /// Replaces all occurrences of a regex pattern in a file with a replacement string.
    ///
    /// **Parameters**
    /// - `path` (`str`): The file path.
    /// - `pattern` (`str`): The regex pattern to match.
    /// - `value` (`str`): The replacement string.
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if the file cannot be modified or the regex is invalid.
    fn replace_all(&self, path: String, pattern: String, value: String) -> Result<(), String>;

    #[eldritch_method]
    /// Creates a temporary file and returns its path.
    ///
    /// **Parameters**
    /// - `name` (`Option<str>`): Optional preferred filename. If None, a random name is generated.
    ///
    /// **Returns**
    /// - `str`: The absolute path to the temporary file.
    ///
    /// **Errors**
    /// - Returns an error string if creation fails.
    fn temp_file(&self, name: Option<String>) -> Result<String, String>;

    #[eldritch_method]
    /// Renders a Jinja2 template file to a destination path.
    ///
    /// **Parameters**
    /// - `template_path` (`str`): Path to the source template file.
    /// - `dst` (`str`): Destination path for the rendered file.
    /// - `args` (`Dict<str, Value>`): Variables to substitute in the template.
    /// - `autoescape` (`bool`): Whether to enable HTML auto-escaping (OWASP recommendations).
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if the template cannot be read, parsed, or written.
    fn template(
        &self,
        template_path: String,
        dst: String,
        args: BTreeMap<String, Value>,
        autoescape: bool,
    ) -> Result<(), String>;

    #[eldritch_method]
    /// Timestomps a file.
    ///
    /// Modifies the timestamps (modified, access, creation) of a file.
    /// Can use a reference file or specific values.
    ///
    /// **Parameters**
    /// - `path` (`str`): The target file to modify.
    /// - `mtime` (`Option<Value>`): New modification time (Int epoch or String).
    /// - `atime` (`Option<Value>`): New access time (Int epoch or String).
    /// - `ctime` (`Option<Value>`): New creation time (Int epoch or String). Windows only.
    /// - `ref_file` (`Option<str>`): Path to a reference file to copy timestamps from.
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if the operation fails or input is invalid.
    fn timestomp(
        &self,
        path: String,
        mtime: Option<Value>,
        atime: Option<Value>,
        ctime: Option<Value>,
        ref_file: Option<String>,
    ) -> Result<(), String>;

    #[eldritch_method]
    /// Writes content to a file, overwriting it if it exists.
    ///
    /// **Parameters**
    /// - `path` (`str`): The file path.
    /// - `content` (`str`): The string content to write.
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if writing fails.
    fn write(&self, path: String, content: String) -> Result<(), String>;

    #[eldritch_method]
    /// Finds files matching specific criteria.
    ///
    /// **Parameters**
    /// - `path` (`str`): The base directory to start searching from.
    /// - `name` (`Option<str>`): Filter by filename (substring match).
    /// - `file_type` (`Option<str>`): Filter by type ("file" or "dir").
    /// - `permissions` (`Option<int>`): Filter by permissions (Unix octal e.g., 777, Windows readonly check).
    /// - `modified_time` (`Option<int>`): Filter by modification time (epoch seconds).
    /// - `create_time` (`Option<int>`): Filter by creation time (epoch seconds).
    ///
    /// **Returns**
    /// - `List<str>`: A list of matching file paths.
    ///
    /// **Errors**
    /// - Returns an error string if the search encounters issues.
    fn find(
        &self,
        path: String,
        name: Option<String>,
        file_type: Option<String>,
        permissions: Option<i64>,
        modified_time: Option<i64>,
        create_time: Option<i64>,
    ) -> Result<Vec<String>, String>;
}
