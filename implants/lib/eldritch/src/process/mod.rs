mod info_impl;
mod kill_impl;
mod list_impl;
mod name_impl;
mod netstat_impl;

use starlark::{
    environment::MethodsBuilder,
    starlark_module,
    values::{dict::Dict, none::NoneType, starlark_value, Heap},
};

/*
 * Define our library for this module.
 */
crate::eldritch_lib!(ProcessLibrary, "process_library");

/*
 * Below, we define starlark wrappers for all of our library methods.
 * The functions must be defined here to be present on our library.
 */
#[starlark_module]
#[rustfmt::skip]
#[allow(clippy::needless_lifetimes, clippy::type_complexity, clippy::too_many_arguments)]
fn methods(builder: &mut MethodsBuilder) {
    #[allow(unused_variables)]
    fn kill(this: &ProcessLibrary, pid: i32) -> anyhow::Result<NoneType> {
        kill_impl::kill(pid)?;
        Ok(NoneType{})
    }

    #[allow(unused_variables)]
    fn list<'v>(this: &ProcessLibrary, starlark_heap: &'v Heap) -> anyhow::Result<Vec<Dict<'v>>> {
        list_impl::list(starlark_heap)
    }

    #[allow(unused_variables)]
    fn info<'v>(this: &ProcessLibrary, starlark_heap: &'v Heap, pid: Option<usize>) -> anyhow::Result<Dict<'v>> {
        info_impl::info(starlark_heap, pid)
    }

    #[allow(unused_variables)]
    fn name(this: &ProcessLibrary, pid: i32) -> anyhow::Result<String> {
         name_impl::name(pid)
    }

    #[allow(unused_variables)]
    fn netstat<'v>(this: &ProcessLibrary, starlark_heap: &'v Heap) -> anyhow::Result<Vec<Dict<'v>>> {
        netstat_impl::netstat(starlark_heap)
    }
}
