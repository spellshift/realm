use wasm_bindgen::prelude::*;
use crate::{Interpreter, Value};
use crate::repl::{Repl, Input, ReplAction};
use alloc::string::ToString;
use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;

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
pub struct WasmRepl {
    interp: Interpreter,
    repl: Repl,
}

#[wasm_bindgen]
pub struct RenderState {
    prompt: String,
    buffer: String,
    cursor: usize,
}

#[wasm_bindgen]
impl RenderState {
    #[wasm_bindgen(getter)]
    pub fn prompt(&self) -> String { self.prompt.clone() }
    #[wasm_bindgen(getter)]
    pub fn buffer(&self) -> String { self.buffer.clone() }
    #[wasm_bindgen(getter)]
    pub fn cursor(&self) -> usize { self.cursor }
}

#[wasm_bindgen]
pub struct ExecutionResult {
    output: Option<String>,
    echo: Option<String>,
}

#[wasm_bindgen]
impl ExecutionResult {
    #[wasm_bindgen(getter)]
    pub fn output(&self) -> Option<String> { self.output.clone() }
    #[wasm_bindgen(getter)]
    pub fn echo(&self) -> Option<String> { self.echo.clone() }
}

#[wasm_bindgen]
impl WasmRepl {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmRepl {
        let mut interp = Interpreter::new();
        interp.register_function("print", wasm_print);
        WasmRepl {
            interp,
            repl: Repl::new(),
        }
    }

    pub fn load_history(&mut self, js_history: Vec<JsValue>) {
        let history: Vec<String> = js_history.iter().filter_map(|x| x.as_string()).collect();
        self.repl.load_history(history);
    }

    pub fn get_history(&self) -> Vec<JsValue> {
        self.repl.get_history().iter().map(|s| JsValue::from_str(s)).collect()
    }

    pub fn get_state(&self) -> RenderState {
        let s = self.repl.get_render_state();
        RenderState {
            prompt: s.prompt,
            buffer: s.buffer,
            cursor: s.cursor,
        }
    }

    pub fn handle_key(&mut self, key: &str, ctrl: bool, _alt: bool, _meta: bool, shift: bool) -> ExecutionResult {
        let input = match key {
            "Enter" => if shift { Input::ForceEnter } else { Input::Enter },
            "Backspace" => Input::Backspace,
            "Delete" => Input::Delete,
            "ArrowLeft" => Input::Left,
            "ArrowRight" => Input::Right,
            "ArrowUp" => Input::Up,
            "ArrowDown" => Input::Down,
            "Home" => Input::Home,
            "End" => Input::End,
            "Tab" => Input::Tab,
            "c" if ctrl => Input::Cancel,
            "l" if ctrl => Input::ClearScreen,
            "u" if ctrl => Input::KillLine,
            "k" if ctrl => Input::KillToEnd,
            "w" if ctrl => Input::WordBackspace,
            _ => {
                if key.len() == 1 && !ctrl {
                    Input::Char(key.chars().next().unwrap())
                } else {
                    return ExecutionResult { output: None, echo: None };
                }
            }
        };

        self.process_input(input)
    }

    pub fn handle_paste(&mut self, text: &str) -> ExecutionResult {
        let mut final_res = ExecutionResult { output: None, echo: None };
        let mut echo_acc = String::new();
        let mut output_acc = String::new();

        if !text.contains('\n') {
            for c in text.chars() {
                self.repl.handle_input(Input::Char(c));
            }
            return final_res;
        }

        // Split by lines but keep newlines to drive logic?
        // Actually we can process char by char.
        for c in text.chars() {
            let input = if c == '\n' { Input::Enter } else { Input::Char(c) };
            let res = self.process_input(input);

            if let Some(e) = res.echo {
                if !echo_acc.is_empty() { echo_acc.push('\n'); }
                echo_acc.push_str(&e);
            }
            if let Some(o) = res.output {
                if !output_acc.is_empty() { output_acc.push('\n'); }
                output_acc.push_str(&o);
            }
        }

        if !echo_acc.is_empty() { final_res.echo = Some(echo_acc); }
        if !output_acc.is_empty() { final_res.output = Some(output_acc); }
        final_res
    }

    fn process_input(&mut self, input: Input) -> ExecutionResult {
        match self.repl.handle_input(input) {
            ReplAction::Submit { code, last_line, prompt } => {
                let echo = format!("{}{}", prompt, last_line);
                let res = self.execute(&code);
                ExecutionResult {
                    echo: Some(echo),
                    output: res.output
                }
            },
            ReplAction::AcceptLine { line, prompt } => {
                ExecutionResult {
                    output: None,
                    echo: Some(format!("{}{}", prompt, line))
                }
            },
            ReplAction::Render => ExecutionResult { output: None, echo: None },
            ReplAction::None => ExecutionResult { output: None, echo: None },
            ReplAction::Quit => ExecutionResult { output: Some("Use 'exit' or close tab.".to_string()), echo: None },
        }
    }

    fn execute(&mut self, code: &str) -> ExecutionResult {
        match self.interp.interpret(code) {
            Ok(v) => {
                if let Value::None = v {
                    ExecutionResult { output: None, echo: None }
                } else {
                    ExecutionResult { output: Some(v.to_string()), echo: None }
                }
            },
            Err(e) => ExecutionResult { output: Some(format!("Error: {}", e)), echo: None },
        }
    }
}
