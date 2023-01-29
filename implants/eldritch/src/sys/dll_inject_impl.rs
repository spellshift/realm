use anyhow::Result;
use starlark::values::none::NoneType;
use std::process::Command;
use std::str;

pub fn dll_inject(dll_path: String, pid: i32) -> Result<NoneType> {
    unimplemented!("Method unimplemented")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_dll_inject() -> anyhow::Result<()>{
        unimplemented!("Method unimplemented")
    }
}