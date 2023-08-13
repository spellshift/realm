use allocative_derive::Allocative;
use derive_more::Display;
use starlark::environment::GlobalsBuilder;
use starlark::values::starlark_value_as_type::StarlarkValueAsType;
use starlark::values::{AllocValue, Value, Heap};
use starlark::{starlark_simple_value, values::StarlarkValue};
use starlark_derive::{NoSerialize, starlark_module};
use starlark_derive::starlark_value;
use starlark_derive::ProvidesStaticType;

#[derive(Debug, PartialEq, Eq, Display, ProvidesStaticType, NoSerialize, Allocative)]
#[display(fmt = "stdout: {}, stderr: {}, status: {}", stdout, stderr, status)]
struct CommandOutput {
    stdout: String,
    stderr: String,
    status: i32,
}
starlark_simple_value!(CommandOutput);

#[allow(non_upper_case_globals)]
#[starlark_value(type = "command_output")]
impl<'v> StarlarkValue<'v> for CommandOutput {
    // How we add them
}

// impl<'v> AllocValue<'v> for CommandOutput {
//     fn alloc_value(self, heap: &'v Heap) -> Value<'v> {
//         heap.alloc_simple(self)
//     }
// }

#[starlark_module]
fn compiler_args_globals(globals: &mut GlobalsBuilder) {
    const CommandOutput: StarlarkValueAsType<CommandOutput> = StarlarkValueAsType::new();

    fn command_output( stdout: String, stderr: String, status: i32) -> anyhow::Result<CommandOutput> {
        Ok(CommandOutput{ stdout, stderr, status })
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use starlark::environment::{Module, LibraryExtension};
    use starlark::eval::Evaluator;
    use starlark::syntax::{AstModule, Dialect};
    use eldritch::get_eldritch;
    use starlark::values::Value;

    #[test]
    fn test_command_output_type_creation() -> anyhow::Result<()>{
        let tome_filename = "test.eld";
//         let tome_contents = r#"
// res = CommandOutput(stdout="Hello",stderr="World",status=0)
// print(res.stdout)
// "#;
        let tome_contents = r#"
print(type(test_command))
res = command_output(stdout='hi',stderr='',status=0)
print(res)
"#;

        let ast =  match AstModule::parse(
            tome_filename,
            tome_contents.to_string(),
            &Dialect::Extended
        ) {
            Ok(res) => res,
            Err(err) => return Err(anyhow::anyhow!("[eldritch] Unable to parse eldritch tome: {}: {} {}", err.to_string(), tome_filename.clone(), tome_contents.clone())),
        };

        // let globals = match get_eldritch() {
        //     Ok(local_globals) => local_globals,
        //     Err(local_error) => return Err(anyhow::anyhow!("[eldritch] Failed to get_eldritch globals: {}", local_error.to_string())),
        // };

        let globals = GlobalsBuilder::extended_by(
            &[
                LibraryExtension::StructType,
                LibraryExtension::RecordType,
                LibraryExtension::EnumType,
                LibraryExtension::Map,
                LibraryExtension::Filter,
                LibraryExtension::Partial,
                LibraryExtension::ExperimentalRegex,
                LibraryExtension::Debug,
                LibraryExtension::Print,
                LibraryExtension::Breakpoint,
                LibraryExtension::Json,
                LibraryExtension::Abs,
                LibraryExtension::Typing,
            ]
        ).with(compiler_args_globals).build();        

        let module: Module = Module::new();
        module.set(
            "test_command", 
            module.heap().alloc(CommandOutput{ 
                stdout: "Hello".to_string(), 
                stderr: "World".to_string(), 
                status: 0, 
            })
        );

        let mut eval: Evaluator = Evaluator::new(&module);

        let res: Value = match eval.eval_module(ast, &globals) {
            Ok(eval_val) => eval_val,
            Err(eval_error) => return Err(anyhow::anyhow!("[eldritch] Eldritch eval_module failed:\n{}", eval_error)),
        };

        println!("{}", res);
    
        Ok(())
    }
}