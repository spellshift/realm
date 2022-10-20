extern crate golem;
extern crate eldritch;

use clap::{Command, Arg};
use tokio::task;
use std::fs;

use starlark::{starlark_module};
use starlark::environment::{GlobalsBuilder, Globals, Module};
use starlark::syntax::{AstModule, Dialect};
use starlark::eval::Evaluator;
use starlark::values::Value;

use eldritch::file::FileLibrary;
use eldritch::process::ProcessLibrary;
use eldritch::sys::SysLibrary;

fn eldritch_run(tome_filename: String, tome_contents: String) -> Result<String, golem::Error> {
    let ast: AstModule = AstModule::parse(
        &tome_filename, 
        tome_contents.as_str().to_owned(), 
        &Dialect::Standard
    ).unwrap();

    #[starlark_module]
    fn eldritch(builder: &mut GlobalsBuilder) {
        const file: FileLibrary = FileLibrary();
        const process: ProcessLibrary = ProcessLibrary();
        const sys: SysLibrary = SysLibrary();
    }

    let globals = GlobalsBuilder::extended().with(eldritch).build();
    let module: Module = Module::new();

    let mut eval: Evaluator = Evaluator::new(&module);

    let res: Value = eval.eval_module(ast, &globals).unwrap();

    println!("{:?}", res.unpack_str());

    // let mut eld = Evaluator::eval_function(
    //     &mut self, "dir(file)", positional, named
    // );

//     let mut a = Assert::new();
//     a.globals_add(globals);
//     a.all_true(
//         r#"
// dir(file) == ["append", "copy", "download", "exists", "hash", "is_dir", "mkdir", "read", "remove", "rename", "replace", "replace_all", "timestomp", "write"]
// dir(process) == ["kill", "list", "name"]
// dir(sys) == ["exec", "is_linux", "is_macos", "is_windows", "shell"]
// "#,
//     );
    Ok("Ran tome".to_string())
}

async fn interactive() -> Result<(), golem::Error> {
    Ok(())
}

async fn run(tome_path: String) -> Result<String, golem::Error> {
    println!("Executing {}", tome_path);
    // Read a tome script
    let tome_contents = fs::read_to_string(tome_path.clone())?;
    // Execute a tome script
    let tome_results: String = eldritch_run(tome_path, tome_contents)?;
    // Return script output
    return Ok(tome_results)
}

#[tokio::main]
async fn main() -> Result<(), golem::Error> {
    let matches = Command::new("golem")
        .arg(Arg::with_name("INPUT")
            .help("Set the tomes to run")
            .multiple_occurrences(true)
            .required(false)
        ).get_matches();

    if matches.contains_id("INPUT") {
        // Get list of files
        let res = matches.try_get_many::<String>("INPUT")
            .unwrap()
            .unwrap();


        // Queue async tasks
        let mut all_tome_futures: Vec<_> = vec![];
        for tome in res{
            println!("Queueing {}", tome.clone().to_string());
            let tome_execution_task = run(tome.to_string());
            all_tome_futures.push(task::spawn(tome_execution_task))
        }

        // Collect results
        let mut result: Vec<String> = Vec::new();
        for tome_task in all_tome_futures {
            match tome_task.await.unwrap() {
                Ok(res) => result.push(res),
                Err(_err) => continue,
            }
        }

        println!("{:?}", result);

    } else {
        match interactive().await {
            Ok(_) => todo!(),
            Err(_) => todo!(),
        }
    }
    Ok(())
}
