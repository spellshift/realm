use allocative_derive::Allocative;
use derive_more::Display;
use starlark::environment::{MethodsBuilder, MethodsStatic, Methods};
use starlark::values::{UnpackValue, Value, ValueLike};
use starlark::{starlark_simple_value, values::StarlarkValue};
use starlark_derive::NoSerialize;
use starlark_derive::starlark_module;
use starlark_derive::starlark_value;
use starlark_derive::ProvidesStaticType;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display, ProvidesStaticType, NoSerialize, Allocative)]
pub enum FileType {
    File,
    Directory,
    Link,
    Unknown,
}

#[allow(non_upper_case_globals)]
#[starlark_value(type = "file_metadata_type")]
impl<'v> StarlarkValue<'v> for FileType {
}

impl<'v> UnpackValue<'v> for FileType {
    fn expected() -> String {
        FileType::get_type_value_static().as_str().to_owned()
    }

    fn unpack_value(value: Value<'v>) -> Option<Self> {
        let tmp = value.downcast_ref::<FileType>().unwrap();
        Some(*tmp)
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Display, ProvidesStaticType, NoSerialize, Allocative)]
#[display(fmt = "{} {} {} {} {} {} {}", name, file_type, size, owner, group, permissions, time_modified)]
pub struct FileMetadata {
    pub name: String,
    pub file_type: FileType,
    pub size: u64,
    pub owner: String,
    pub group: String,
    pub permissions: String,
    pub time_modified: String,
}

starlark_simple_value!(FileMetadata);

#[allow(non_upper_case_globals)]
#[starlark_value(type = "file_metadata")]
impl<'v> StarlarkValue<'v> for FileMetadata {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new();
        RES.methods(methods)
    }
}

impl<'v> UnpackValue<'v> for FileMetadata {
    fn expected() -> String {
        FileMetadata::get_type_value_static().as_str().to_owned()
    }

    fn unpack_value(value: Value<'v>) -> Option<Self> {
        let tmp = value.downcast_ref::<FileMetadata>().unwrap();
        Some(FileMetadata { 
            name: tmp.name.clone(), 
            file_type: tmp.file_type.clone(), 
            size: tmp.size,
            owner: tmp.owner.clone(),
            group: tmp.group.clone(),
            permissions: tmp.permissions.clone(),
            time_modified: tmp.time_modified.clone(), 
        })
    }
}


#[starlark_module]
fn methods(builder: &mut MethodsBuilder) {
    fn name(this: FileMetadata) -> anyhow::Result<String> {
        Ok(this.name)
    }
    fn file_type(this: FileMetadata) -> anyhow::Result<String> {
        Ok(this.file_type.to_string())
    }
    fn size(this: FileMetadata) -> anyhow::Result<u64> {
        Ok(this.size)
    }
    fn group(this: FileMetadata) -> anyhow::Result<String> {
        Ok(this.group)
    }
    fn permissions(this: FileMetadata) -> anyhow::Result<String> {
        Ok(this.permissions)
    }
    fn time_modified(this: FileMetadata) -> anyhow::Result<String> {
        Ok(this.time_modified)
    }
}