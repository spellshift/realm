mod append_impl;
mod compress_impl;
mod copy_impl;
mod download_impl;
mod exists_impl;
mod hash_impl;
mod is_dir_impl;
mod is_file_impl;
mod list_impl;
mod mkdir_impl;
mod moveto_impl;
mod read_impl;
mod remove_impl;
mod replace_all_impl;
mod replace_impl;
mod template_impl;
mod timestomp_impl;
mod write_impl;

use allocative::Allocative;
use derive_more::Display;

use serde::{Serialize, Serializer};
use starlark::collections::SmallMap;
use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::values::dict::Dict;
use starlark::values::none::NoneType;
use starlark::values::{
    starlark_value, Heap, ProvidesStaticType, StarlarkValue, UnpackValue, Value, ValueLike,
};
use starlark::{starlark_module, starlark_simple_value};

#[derive(Copy, Clone, Debug, PartialEq, Display, ProvidesStaticType, Allocative)]
#[display(fmt = "FileLibrary")]
pub struct FileLibrary();
starlark_simple_value!(FileLibrary);

#[allow(non_upper_case_globals)]
#[starlark_value(type = "file_library")]
impl<'v> StarlarkValue<'v> for FileLibrary {
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

#[derive(Debug, Display)]
enum FileType {
    File,
    Directory,
    Link,
    Unknown,
}

#[derive(Debug, Display)]
#[display(
    fmt = "{} {} {} {} {} {} {}",
    name,
    file_type,
    size,
    owner,
    group,
    permissions,
    time_modified
)]
struct File {
    name: String,
    file_type: FileType,
    size: u64,
    owner: String,
    group: String,
    permissions: String,
    time_modified: String,
}

// This is where all of the "file.X" impl methods are bound
#[starlark_module]
#[rustfmt::skip]
fn methods(builder: &mut MethodsBuilder) {
    fn append(this: FileLibrary, path: String, content: String) -> anyhow::Result<NoneType> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        append_impl::append(path, content)?;
        Ok(NoneType{})
    }
    fn copy(this: FileLibrary, src: String, dst: String) -> anyhow::Result<NoneType> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        copy_impl::copy(src, dst)?;
        Ok(NoneType{})
    }
    fn compress(this: FileLibrary, src: String, dst: String) -> anyhow::Result<NoneType> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        compress_impl::compress(src, dst)?;
        Ok(NoneType{})
    }
    fn download(this: FileLibrary, uri: String, dst: String) -> anyhow::Result<NoneType> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        download_impl::download(uri, dst)?;
        Ok(NoneType{})
    }
    fn exists(this: FileLibrary, path: String) -> anyhow::Result<bool> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        exists_impl::exists(path)
    }
    fn hash(this: FileLibrary, path: String) -> anyhow::Result<String> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        hash_impl::hash(path)
    }
    fn is_dir(this: FileLibrary, path: String) -> anyhow::Result<bool> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        is_dir_impl::is_dir(path)
    }
    fn is_file(this: FileLibrary, path: String) -> anyhow::Result<bool> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        is_file_impl::is_file(path)
    }
    fn list<'v>(this: FileLibrary, starlark_heap: &'v Heap, path: String) ->  anyhow::Result<Vec<Dict<'v>>>  {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        list_impl::list(starlark_heap, path)
    }
    fn mkdir(this: FileLibrary, path: String) -> anyhow::Result<NoneType> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        mkdir_impl::mkdir(path)?;
        Ok(NoneType{})
    }
    fn read(this: FileLibrary, path: String) -> anyhow::Result<String> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        read_impl::read(path)
    }
    fn remove(this: FileLibrary, path: String) -> anyhow::Result<NoneType> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        remove_impl::remove(path)?;
        Ok(NoneType{})
    }
    fn moveto(this: FileLibrary, old: String, new: String) -> anyhow::Result<NoneType> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        moveto_impl::moveto(old, new)?;
        Ok(NoneType{})
    }
    fn replace_all(this: FileLibrary, path: String, pattern: String, value: String) -> anyhow::Result<NoneType> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        replace_all_impl::replace_all(path, pattern, value)?;
        Ok(NoneType{})
    }
    fn replace(this: FileLibrary, path: String, pattern: String, value: String) -> anyhow::Result<NoneType> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        replace_impl::replace(path, pattern, value)?;
        Ok(NoneType{})
    }
    fn timestomp(this: FileLibrary, src: String, dst: String) -> anyhow::Result<NoneType> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        timestomp_impl::timestomp(src, dst)?;
        Ok(NoneType{})
    }
    fn template(this: FileLibrary, template_path: String, dst_path: String, args: SmallMap<String, Value>, autoescape: bool) -> anyhow::Result<NoneType> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        template_impl::template(template_path, dst_path, args, autoescape)?;
        Ok(NoneType{})
    }
    fn write(this: FileLibrary, path: String, content: String) -> anyhow::Result<NoneType> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        write_impl::write(path, content)?;
        Ok(NoneType{})
    }
}
