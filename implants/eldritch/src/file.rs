mod append_impl;
mod copy_impl;
mod download_impl;
mod exists_impl;
mod hash_impl;
mod is_dir_impl;
mod mkdir_impl;
mod read_impl;
mod remove_impl;
mod moveto_impl;
mod replace_all_impl;
mod replace_impl;
mod timestomp_impl;
mod write_impl;

use derive_more::Display;

use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::values::{StarlarkValue, Value, UnpackValue, ValueLike};
use starlark::values::none::NoneType;
use starlark::{starlark_type, starlark_simple_value, starlark_module};

#[derive(Copy, Clone, Debug, PartialEq, Display)]
#[display(fmt = "FileLibrary")]
pub struct FileLibrary();
starlark_simple_value!(FileLibrary);

impl<'v> StarlarkValue<'v> for FileLibrary {
    starlark_type!("file_library");

    fn get_methods(&self) -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new();
        RES.methods(methods)
    }
}

impl<'v> UnpackValue<'v> for FileLibrary {
    fn expected() -> String {
        FileLibrary::get_type_value_static().as_str().to_owned()
    }

    fn unpack_value(value: Value<'v>) -> Option<Self> {
        Some(*value.downcast_ref::<FileLibrary>().unwrap())
    }
}

// This is where all of the "file.X" impl methods are bound
#[starlark_module]
fn methods(builder: &mut MethodsBuilder) {
    fn append(_this: FileLibrary, path: String, content: String) -> NoneType {
        append_impl::append(path, content)?;
        Ok(NoneType{})
    }
    fn copy(_this: FileLibrary, src: String, dst: String) -> NoneType {
        copy_impl::copy(src, dst)?;
        Ok(NoneType{})
    }
    fn download(_this: FileLibrary, uri: String, dst: String) -> NoneType {
        download_impl::download(uri, dst)?;
        Ok(NoneType{})
    }
    fn exists(_this: FileLibrary, path: String) -> bool {
        exists_impl::exists(path)
    }
    fn hash(_this: FileLibrary, path: String) -> String {
        hash_impl::hash(path)
    }
    fn is_dir(_this: FileLibrary, path: String) -> bool {
        is_dir_impl::is_dir(path)
    }
    fn mkdir(_this: FileLibrary, path: String) -> NoneType {
        mkdir_impl::mkdir(path)?;
        Ok(NoneType{})
    }
    fn read(_this: FileLibrary, path: String) -> String {
        read_impl::read(path)
    }
    fn remove(_this: FileLibrary, path: String) -> NoneType {
        remove_impl::remove(path)?;
        Ok(NoneType{})
    }
    fn rename(_this: FileLibrary, old: String, new: String) -> NoneType {
        moveto_impl::moveto(old, new)?;
        Ok(NoneType{})
    }
    fn replace_all(_this: FileLibrary, path: String, pattern: String, value: String) -> NoneType {
        replace_all_impl::replace_all(path, pattern, value)?;
        Ok(NoneType{})
    }
    fn replace(_this: FileLibrary, path: String, pattern: String, value: String) -> NoneType {
        replace_impl::replace(path, pattern, value)?;
        Ok(NoneType{})
    }
    fn timestomp(_this: FileLibrary, src: String, dst: String) -> NoneType {
        timestomp_impl::timestomp(src, dst)?;
        Ok(NoneType{})
    }
    fn write(_this: FileLibrary, path: String, content: String) -> NoneType {
        write_impl::write(path, content)?;
        Ok(NoneType{})
    }
}