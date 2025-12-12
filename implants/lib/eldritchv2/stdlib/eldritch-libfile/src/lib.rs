extern crate alloc;
use eldritch_core::Value;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;
#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("file")]
pub trait FileLibrary {
    #[eldritch_method]
    fn append(&self, path: String, content: String) -> Result<(), String>;

    #[eldritch_method]
    fn compress(&self, src: String, dst: String) -> Result<(), String>;

    #[eldritch_method]
    fn copy(&self, src: String, dst: String) -> Result<(), String>;

    #[eldritch_method]
    fn decompress(&self, src: String, dst: String) -> Result<(), String>;

    #[eldritch_method]
    fn exists(&self, path: String) -> Result<bool, String>;

    #[eldritch_method]
    fn follow(&self, path: String, fn_val: Value) -> Result<(), String>; // fn is reserved

    #[eldritch_method]
    fn is_dir(&self, path: String) -> Result<bool, String>;

    #[eldritch_method]
    fn is_file(&self, path: String) -> Result<bool, String>;

    #[eldritch_method]
    fn list(&self, path: String) -> Result<Vec<BTreeMap<String, Value>>, String>;

    #[eldritch_method]
    fn mkdir(&self, path: String, parent: Option<bool>) -> Result<(), String>;

    #[eldritch_method("move")]
    fn move_(&self, src: String, dst: String) -> Result<(), String>;

    #[eldritch_method]
    fn parent_dir(&self, path: String) -> Result<String, String>;

    #[eldritch_method]
    fn read(&self, path: String) -> Result<String, String>;

    #[eldritch_method]
    fn read_binary(&self, path: String) -> Result<Vec<u8>, String>;

    #[eldritch_method]
    fn remove(&self, path: String) -> Result<(), String>;

    #[eldritch_method]
    fn replace(&self, path: String, pattern: String, value: String) -> Result<(), String>;

    #[eldritch_method]
    fn replace_all(&self, path: String, pattern: String, value: String) -> Result<(), String>;

    #[eldritch_method]
    fn temp_file(&self, name: Option<String>) -> Result<String, String>;

    #[eldritch_method]
    fn template(
        &self,
        template_path: String,
        dst: String,
        args: BTreeMap<String, Value>,
        autoescape: bool,
    ) -> Result<(), String>;

    #[eldritch_method]
    fn timestomp(&self, src: String, dst: String) -> Result<(), String>;

    #[eldritch_method]
    fn write(&self, path: String, content: String) -> Result<(), String>;

    #[eldritch_method]
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
