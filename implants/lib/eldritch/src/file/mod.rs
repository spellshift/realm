mod append_impl;
mod compress_impl;
mod copy_impl;
mod decompress_impl;
mod exists_impl;
mod find_impl;
mod follow_impl;
mod is_dir_impl;
mod is_file_impl;
mod list_impl;
mod mkdir_impl;
mod moveto_impl;
mod parent_dir_impl;
mod read_binary_impl;
mod read_impl;
mod remove_impl;
mod replace_all_impl;
mod replace_impl;
mod temp_file_impl;
mod template_impl;
mod timestomp_impl;
mod write_impl;

use derive_more::Display;
use starlark::{
    collections::SmallMap,
    environment::MethodsBuilder,
    eval::Evaluator,
    starlark_module,
    values::{dict::Dict, none::NoneType, starlark_value, Heap, Value},
};

#[derive(Debug, Display)]
enum FileType {
    File,
    Directory,
    Link,
    Unknown,
}

#[derive(Debug, Display)]
#[display(
    fmt = "{} {} {} {} {} {} {} {}",
    name,
    absolute_path,
    file_type,
    size,
    owner,
    group,
    permissions,
    time_modified
)]
struct File {
    name: String,
    absolute_path: String,
    file_type: FileType,
    size: u64,
    owner: String,
    group: String,
    permissions: String,
    time_modified: String,
}

/*
 * Define our library for this module.
 */
crate::eldritch_lib!(FileLibrary, "file_library");

/*
 * Below, we define starlark wrappers for all of our library methods.
 * The functions must be defined here to be present on our library.
 */
#[starlark_module]
#[rustfmt::skip]
#[allow(clippy::needless_lifetimes, clippy::type_complexity, clippy::too_many_arguments)]
fn methods(builder: &mut MethodsBuilder) {
    #[allow(unused_variables)]
    fn append(this: &FileLibrary, path: String, content: String) -> anyhow::Result<NoneType> {
        append_impl::append(path, content)?;
        Ok(NoneType{})
    }

    #[allow(unused_variables)]
    fn copy(this: &FileLibrary, src: String, dst: String) -> anyhow::Result<NoneType> {
        copy_impl::copy(src, dst)?;
        Ok(NoneType{})
    }

    #[allow(unused_variables)]
    fn compress(this: &FileLibrary, src: String, dst: String) -> anyhow::Result<NoneType> {
        compress_impl::compress(src, dst)?;
        Ok(NoneType{})
    }

    #[allow(unused_variables)]
    fn decompress(this: &FileLibrary, src: String, dst: String) -> anyhow::Result<NoneType> {
        decompress_impl::decompress(src, dst)?;
        Ok(NoneType{})
    }

    #[allow(unused_variables)]
    fn exists(this: &FileLibrary, path: String) -> anyhow::Result<bool> {
        exists_impl::exists(path)
    }

    #[allow(unused_variables)]
    fn is_dir(this: &FileLibrary, path: String) -> anyhow::Result<bool> {
        is_dir_impl::is_dir(path)
    }

    #[allow(unused_variables)]
    fn is_file(this: &FileLibrary, path: String) -> anyhow::Result<bool> {
        is_file_impl::is_file(path)
    }

    #[allow(unused_variables)]
    fn list<'v>(this: &FileLibrary, starlark_heap: &'v Heap, path: String) ->  anyhow::Result<Vec<Dict<'v>>>  {
        list_impl::list(starlark_heap, path)
    }

    #[allow(unused_variables)]
    fn mkdir(this: &FileLibrary, path: String, parent: Option<bool>) -> anyhow::Result<NoneType> {
        mkdir_impl::mkdir(path, parent)?;
        Ok(NoneType{})
    }

    #[allow(unused_variables)]
    fn read(this: &FileLibrary, path: String) -> anyhow::Result<String> {
        read_impl::read(path)
    }

    #[allow(unused_variables)]
    fn read_binary(this: &FileLibrary, path: String) -> anyhow::Result<Vec<u32>> {
        read_binary_impl::read_binary(path)
    }

    #[allow(unused_variables)]
    fn remove(this: &FileLibrary, path: String) -> anyhow::Result<NoneType> {
        remove_impl::remove(path)?;
        Ok(NoneType{})
    }

    #[allow(unused_variables)]
    fn moveto(this: &FileLibrary, old: String, new: String) -> anyhow::Result<NoneType> {
        moveto_impl::moveto(old, new)?;
        Ok(NoneType{})
    }
    #[allow(unused_variables)]
    fn parent_dir(this: &FileLibrary, path: String) -> anyhow::Result<String> {
        parent_dir_impl::parent_dir(path)
    }

    #[allow(unused_variables)]
    fn replace_all(this: &FileLibrary, path: String, pattern: String, value: String) -> anyhow::Result<NoneType> {
        replace_all_impl::replace_all(path, pattern, value)?;
        Ok(NoneType{})
    }

    #[allow(unused_variables)]
    fn replace(this: &FileLibrary, path: String, pattern: String, value: String) -> anyhow::Result<NoneType> {
        replace_impl::replace(path, pattern, value)?;
        Ok(NoneType{})
    }

    #[allow(unused_variables)]
    fn timestomp(this: &FileLibrary, src: String, dst: String) -> anyhow::Result<NoneType> {
        timestomp_impl::timestomp(src, dst)?;
        Ok(NoneType{})
    }

    #[allow(unused_variables)]
    fn template(this: &FileLibrary, template_path: String, dst_path: String, args: SmallMap<String, Value>, autoescape: bool) -> anyhow::Result<NoneType> {
        template_impl::template(template_path, dst_path, args, autoescape)?;
        Ok(NoneType{})
    }

    #[allow(unused_variables)]
    fn write(this: &FileLibrary, path: String, content: String) -> anyhow::Result<NoneType> {
        write_impl::write(path, content)?;
        Ok(NoneType{})
    }

    #[allow(unused_variables)]
    fn find<'v>(this: &FileLibrary, starlark_eval: &mut Evaluator<'v, '_>, path: String, name: Option<String>, file_type: Option<String>, permissions: Option<u64>, modified_time: Option<u64>, create_time: Option<u64>) -> anyhow::Result<Vec<String>> {
        find_impl::find(starlark_eval, path, name, file_type, permissions, modified_time, create_time)
    }

    #[allow(unused_variables)]
    fn follow<'v>(this: &FileLibrary, path: String, f: Value<'v>, eval: &mut Evaluator<'v, '_>) -> anyhow::Result<NoneType> {
        follow_impl::follow(path, f, eval)?;
        Ok(NoneType{})
    }

    #[allow(unused_variables)]
    fn temp_file(this: &FileLibrary, name: Option<String>) -> anyhow::Result<String> {
        temp_file_impl::temp_file(name)
    }

}
