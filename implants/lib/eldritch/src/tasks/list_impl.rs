use anyhow::{Context, Result};
use starlark::{
    environment::Module,
    eval::Evaluator,
    values::{dict::Dict, Heap},
};

use crate::EldritchRuntime;

pub fn list<'a, 'v>(starlark_eval: &'a mut Evaluator<'a, 'v>) -> Result<Vec<i32>> {
    let a = starlark_eval;
    let tmp: EldritchTaskHandler = a.get_task_handler();

    println!("tmp: {:?}", tmp);
    Ok(Vec::new())
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use starlark::{
//         collections::SmallMap,
//         environment::GlobalsBuilder,
//         eval::Evaluator,
//         starlark_module,
//         syntax::{AstModule, Dialect},
//         values::{none::NoneType, AllocValue, Value},
//     };

//     #[test]
//     fn test_task_list_impl() -> anyhow::Result<()> {
//         let test_eldritch_script = format!(
//             r#"
// func_task_list()
// "#
//         );

//         let ast: AstModule;
//         match AstModule::parse(
//             "test.eldritch",
//             test_eldritch_script.to_owned(),
//             &Dialect::Standard,
//         ) {
//             Ok(res) => ast = res,
//             Err(err) => return Err(err),
//         }

//         #[starlark_module]
//         fn func_task_list(builder: &mut GlobalsBuilder) {
//             fn func_task_list<'a, 'v>(eval: &mut Evaluator) -> anyhow::Result<Vec<i32>> {
//                 list(eval)
//             }
//         }

//         let globals = GlobalsBuilder::standard().with(func_task_list).build();
//         let module: Module = Module::new();

//         let res: SmallMap<Value, Value> = SmallMap::new();
//         let mut input_params: Dict = Dict::new(res);
//         let target_pid_key = module
//             .heap()
//             .alloc_str("target_pid")
//             .to_value()
//             .get_hashed()?;
//         let target_pid_value = module.heap().alloc(0);
//         input_params.insert_hashed(target_pid_key, target_pid_value);

//         module.set("input_params", input_params.alloc_value(module.heap()));

//         let mut eval: Evaluator = Evaluator::new(&module);
//         let res: Value = eval.eval_module(ast, &globals).unwrap();
//         let _res_string = res.to_string();
//         Ok(())
//     }
// }
