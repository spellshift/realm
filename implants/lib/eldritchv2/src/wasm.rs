use wasm_bindgen::prelude::*;
use crate::{Interpreter, Value};
use alloc::string::ToString;
use alloc::string::String;
use alloc::format;

#[wasm_bindgen]
extern "C" {
    fn repl_print(s: &str);
}

fn wasm_print(args: &[Value]) -> Result<Value, String> {
    let mut out = String::new();
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        out.push_str(&arg.to_string());
    }
    repl_print(&out);
    Ok(Value::None)
}

#[wasm_bindgen]
pub struct WasmInterpreter {
    interp: Interpreter,
}

#[wasm_bindgen]
impl WasmInterpreter {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmInterpreter {
        let mut interp = Interpreter::new();
        interp.register_function("print", wasm_print);
        WasmInterpreter { interp }
    }

    pub fn run(&mut self, code: &str) -> String {
        match self.interp.interpret(code) {
            Ok(v) => {
                if let Value::None = v {
                    String::new()
                } else {
                    v.to_string()
                }
            },
            Err(e) => format!("Error: {}", e),
        }
    }
}
