use anyhow::Result;
use std::process;
use starlark::values::Heap;

pub fn get_pid(starlark_heap: &Heap) -> Result<u32> {
    Ok(process::id())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sys_get_pid() {
        let starlark_heap = Heap::new();
        let res = get_pid(&starlark_heap).unwrap();
        assert_eq!(res, process::id());
    }
}