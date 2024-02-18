mod file_impl;
mod process_list_impl;
mod ssh_key_impl;
mod user_password_impl;

use starlark::{
    collections::SmallMap,
    environment::MethodsBuilder,
    eval::Evaluator,
    starlark_module,
    values::{list::UnpackList, none::NoneType, starlark_value, Value},
};

/*
 * Define our library for this module.
 */
crate::eldritch_lib!(ReportLibrary, "report_library");

/*
 * Below, we define starlark wrappers for all of our library methods.
 * The functions must be defined here to be present on our library.
 */
#[starlark_module]
#[rustfmt::skip]
#[allow(clippy::needless_lifetimes, clippy::type_complexity, clippy::too_many_arguments)]
fn methods(builder: &mut MethodsBuilder) {
    #[allow(unused_variables)]
    fn file(this: &ReportLibrary, starlark_eval: &mut Evaluator<'v, '_>, path: String) -> anyhow::Result<NoneType> {
        let env = crate::runtime::Environment::from_extra(starlark_eval.extra)?;
        file_impl::file(env, path)?;
        Ok(NoneType{})
    }

    #[allow(unused_variables)]
    fn process_list(this: &ReportLibrary, starlark_eval: &mut Evaluator<'v, '_>, process_list: UnpackList<SmallMap<String, Value>>) -> anyhow::Result<NoneType> {
        let env = crate::runtime::Environment::from_extra(starlark_eval.extra)?;
        process_list_impl::process_list(env, process_list.items)?;
        Ok(NoneType{})
    }

    #[allow(unused_variables)]
    fn ssh_key(this: &ReportLibrary, starlark_eval: &mut Evaluator<'v, '_>, username: String, key: String) -> anyhow::Result<NoneType> {
        let env = crate::runtime::Environment::from_extra(starlark_eval.extra)?;
        ssh_key_impl::ssh_key(env, username, key)?;
        Ok(NoneType{})
    }

    #[allow(unused_variables)]
    fn user_password(this: &ReportLibrary, starlark_eval: &mut Evaluator<'v, '_>, username: String, password: String) -> anyhow::Result<NoneType> {
        let env = crate::runtime::Environment::from_extra(starlark_eval.extra)?;
        user_password_impl::user_password(env, username, password)?;
        Ok(NoneType{})
    }
}
