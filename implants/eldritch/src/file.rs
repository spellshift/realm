mod append_impl;
mod copy_impl;
mod download_impl;
mod exists_impl;
mod hash_impl;
mod is_dir_impl;
mod is_file_impl;
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
use starlark::values::none::NoneType;
use starlark::values::{StarlarkValue, Value, UnpackValue, ValueLike, ProvidesStaticType};
use starlark::{starlark_type, starlark_simple_value, starlark_module};
use serde::{Serialize,Serializer};

#[derive(Copy, Clone, Debug, PartialEq, Display, ProvidesStaticType)]
pub struct FileLibrary();
starlark_simple_value!(FileLibrary);

impl<'v> StarlarkValue<'v> for FileLibrary {
    starlark_type!("file_library");

    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new();
        RES.methods(methods)
    }
}

impl Serialize for FileLibrary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_none()
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
    fn append(this: FileLibrary, path: String, content: String) -> anyhow::Result<NoneType> {
        append_impl::append(path, content)?;
        Ok(NoneType)
    }
    fn copy(this: FileLibrary, src: String, dst: String) -> anyhow::Result<NoneType> {
        copy_impl::copy(src, dst)?;
        Ok(NoneType)
    }
    fn download(this: FileLibrary, uri: String, dst: String) -> anyhow::Result<NoneType> {
        download_impl::download(uri, dst)?;
        Ok(NoneType)
    }
    fn exists(this: FileLibrary, path: String) -> anyhow::Result<bool> {
        Ok(exists_impl::exists(path)?)
    }
    fn hash(this: FileLibrary, path: String) -> anyhow::Result<String> {
        Ok(hash_impl::hash(path)?)
    }
    fn is_dir(this: FileLibrary, path: String) -> anyhow::Result<bool> {
        Ok(is_dir_impl::is_dir(path)?)
    }
    fn is_file(_this: FileLibrary, path: String) -> anyhow::Result<bool> {
        is_file_impl::is_file(path)
    }
    fn mkdir(this: FileLibrary, path: String) -> anyhow::Result<NoneType> {
        mkdir_impl::mkdir(path)?;
        Ok(NoneType)
    }
    fn read(this: FileLibrary, path: String) -> anyhow::Result<String> {
        Ok(read_impl::read(path)?)
    }
    fn remove(this: FileLibrary, path: String) -> anyhow::Result<NoneType> {
        remove_impl::remove(path)?;
        Ok(NoneType)
    }
    fn rename(this: FileLibrary, old: String, new: String) -> anyhow::Result<NoneType> {
        moveto_impl::moveto(old, new)?;
        Ok(NoneType)
    }
    fn replace_all(this: FileLibrary, path: String, pattern: String, value: String) -> anyhow::Result<NoneType> {
        replace_all_impl::replace_all(path, pattern, value)?;
        Ok(NoneType)
    }
    fn replace(this: FileLibrary, path: String, pattern: String, value: String) -> anyhow::Result<NoneType> {
        replace_impl::replace(path, pattern, value)?;
        Ok(NoneType)
    }
    fn timestomp(this: FileLibrary, src: String, dst: String) -> anyhow::Result<NoneType> {
        timestomp_impl::timestomp(src, dst)?;
        Ok(NoneType)
    }
    fn write(this: FileLibrary, path: String, content: String) -> anyhow::Result<NoneType> {
        write_impl::write(path, content)?;
        Ok(NoneType)
    }
}