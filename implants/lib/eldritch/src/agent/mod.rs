mod eval_impl;

use starlark::{
    environment::MethodsBuilder,
    eval::Evaluator,
    starlark_module,
    values::{none::NoneType, starlark_value},
};

/*
 * Define our library for this module.
 */
crate::eldritch_lib!(AgentLibrary, "agent_library");

/*
 * Below, we define starlark wrappers for all of our library methods.
 * The functions must be defined here to be present on our library.
 */
#[starlark_module]
#[rustfmt::skip]
#[allow(clippy::needless_lifetimes, clippy::type_complexity, clippy::too_many_arguments)]
fn methods(builder: &mut MethodsBuilder) {
    #[allow(unused_variables)]
    fn eval(this: &AgentLibrary, starlark_eval: &mut Evaluator<'v, '_>, script: String) -> anyhow::Result<NoneType> {
        let env = crate::runtime::Environment::from_extra(starlark_eval.extra)?;
        eval_impl::eval(env, script)?;
        Ok(NoneType{})
    }
}
