mod append_impl;

use derive_more::Display;

use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::values::{StarlarkValue, Value, UnpackValue, ValueLike};
use starlark::values::none::NoneType;
use starlark::{starlark_type, starlark_simple_value, starlark_module};

#[derive(Copy, Clone, Debug, PartialEq, Display)]
#[display(fmt = "PivotLibrary")]
pub struct PivotLibrary();
starlark_simple_value!(PivotLibrary);

impl<'v> StarlarkValue<'v> for PivotLibrary {
    starlark_type!("pivot_library");

    fn get_methods(&self) -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new();
        RES.methods(methods)
    }
}

impl<'v> UnpackValue<'v> for PivotLibrary {
    fn expected() -> String {
        PivotLibrary::get_type_value_static().as_str().to_owned()
    }

    fn unpack_value(value: Value<'v>) -> Option<Self> {
        Some(*value.downcast_ref::<PivotLibrary>().unwrap())
    }
}

// This is where all of the "file.X" impl methods are bound
#[starlark_module]
fn methods(builder: &mut MethodsBuilder) {
    fn revshell(_this: PivotLibrary, path: String, content: String) -> NoneType {
        append_impl::append(path, content)?;
        Ok(NoneType{})
    }
    fn copy(_this: PivotLibrary, src: String, dst: String) -> NoneType {
        copy_impl::copy(src, dst)?;
        Ok(NoneType{})
    }
    fn download(_this: PivotLibrary, uri: String, dst: String) -> NoneType {
        download_impl::download(uri, dst)?;
        Ok(NoneType{})
    }
    fn exists(_this: PivotLibrary, path: String) -> bool {
        exists_impl::exists(path)
    }
    fn hash(_this: PivotLibrary, path: String) -> String {
        hash_impl::hash(path)
    }
    fn is_dir(_this: PivotLibrary, path: String) -> bool {
        is_dir_impl::is_dir(path)
    }
    fn mkdir(_this: PivotLibrary, path: String) -> NoneType {
        mkdir_impl::mkdir(path)?;
        Ok(NoneType{})
    }
    fn read(_this: PivotLibrary, path: String) -> String {
        read_impl::read(path)
    }
    fn remove(_this: PivotLibrary, path: String) -> NoneType {
        remove_impl::remove(path)?;
        Ok(NoneType{})
    }
    fn rename(_this: PivotLibrary, old: String, new: String) -> NoneType {
        rename_impl::rename(old, new)?;
        Ok(NoneType{})
    }
    fn replace_all(_this: PivotLibrary, path: String, pattern: String, value: String) -> NoneType {
        replace_all_impl::replace_all(path, pattern, value)?;
        Ok(NoneType{})
    }
    fn replace(_this: PivotLibrary, path: String, pattern: String, value: String) -> NoneType {
        replace_impl::replace(path, pattern, value)?;
        Ok(NoneType{})
    }
    fn timestomp(_this: PivotLibrary, src: String, dst: String) -> NoneType {
        timestomp_impl::timestomp(src, dst)?;
        Ok(NoneType{})
    }
    fn write(_this: PivotLibrary, path: String, content: String) -> NoneType {
        write_impl::write(path, content)?;
        Ok(NoneType{})
    }
}