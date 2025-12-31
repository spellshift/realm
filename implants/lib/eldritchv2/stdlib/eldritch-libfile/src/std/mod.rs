use super::FileLibrary;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;

pub mod append_impl;
pub mod compress_impl;
pub mod copy_impl;
pub mod decompress_impl;
pub mod exists_impl;
pub mod find_impl;
pub mod follow_impl;
pub mod is_dir_impl;
pub mod is_file_impl;
pub mod list_impl;
pub mod mkdir_impl;
pub mod move_impl;
pub mod parent_dir_impl;
pub mod pwd_impl;
pub mod read_binary_impl;
pub mod read_impl;
pub mod remove_impl;
pub mod replace_all_impl;
pub mod replace_impl;
pub mod temp_file_impl;
pub mod template_impl;
pub mod timestomp_impl;
pub mod write_impl;

#[derive(Debug, Default)]
#[eldritch_library_impl(FileLibrary)]
pub struct StdFileLibrary;

impl FileLibrary for StdFileLibrary {
    fn append(&self, path: String, content: String) -> Result<(), String> {
        append_impl::append(path, content)
    }

    fn compress(&self, src: String, dst: String) -> Result<(), String> {
        compress_impl::compress(src, dst)
    }

    fn copy(&self, src: String, dst: String) -> Result<(), String> {
        copy_impl::copy(src, dst)
    }

    fn decompress(&self, src: String, dst: String) -> Result<(), String> {
        decompress_impl::decompress(src, dst)
    }

    fn exists(&self, path: String) -> Result<bool, String> {
        exists_impl::exists(path)
    }

    fn follow(&self, path: String, fn_val: Value) -> Result<(), String> {
        follow_impl::follow(path, fn_val)
    }

    fn is_dir(&self, path: String) -> Result<bool, String> {
        is_dir_impl::is_dir(path)
    }

    fn is_file(&self, path: String) -> Result<bool, String> {
        is_file_impl::is_file(path)
    }

    fn list(&self, path: Option<String>) -> Result<Vec<BTreeMap<String, Value>>, String> {
        list_impl::list(path)
    }

    fn mkdir(&self, path: String, parent: Option<bool>) -> Result<(), String> {
        mkdir_impl::mkdir(path, parent)
    }

    fn move_(&self, src: String, dst: String) -> Result<(), String> {
        move_impl::move_(src, dst)
    }

    fn parent_dir(&self, path: String) -> Result<String, String> {
        parent_dir_impl::parent_dir(path)
    }

    fn read(&self, path: String) -> Result<String, String> {
        read_impl::read(path)
    }

    fn read_binary(&self, path: String) -> Result<Vec<u8>, String> {
        read_binary_impl::read_binary(path)
    }

    fn pwd(&self) -> Result<Option<String>, String> {
        pwd_impl::pwd()
    }

    fn remove(&self, path: String) -> Result<(), String> {
        remove_impl::remove(path)
    }

    fn replace(&self, path: String, pattern: String, value: String) -> Result<(), String> {
        replace_impl::replace(path, pattern, value)
    }

    fn replace_all(&self, path: String, pattern: String, value: String) -> Result<(), String> {
        replace_all_impl::replace_all(path, pattern, value)
    }

    fn temp_file(&self, name: Option<String>) -> Result<String, String> {
        temp_file_impl::temp_file(name)
    }

    fn template(
        &self,
        template_path: String,
        dst: String,
        args: BTreeMap<String, Value>,
        autoescape: bool,
    ) -> Result<(), String> {
        template_impl::template(template_path, dst, args, autoescape)
    }

    fn timestomp(
        &self,
        path: String,
        mtime: Option<Value>,
        atime: Option<Value>,
        ctime: Option<Value>,
        ref_file: Option<String>,
    ) -> Result<(), String> {
        timestomp_impl::timestomp(path, mtime, atime, ctime, ref_file)
    }

    fn write(&self, path: String, content: String) -> Result<(), String> {
        write_impl::write(path, content)
    }

    fn find(
        &self,
        path: String,
        name: Option<String>,
        file_type: Option<String>,
        permissions: Option<i64>,
        modified_time: Option<i64>,
        create_time: Option<i64>,
    ) -> Result<Vec<String>, String> {
        find_impl::find(
            path,
            name,
            file_type,
            permissions,
            modified_time,
            create_time,
        )
    }
}
