use super::{Input, Repl, ReplAction};
use eldritch_core::{BufferPrinter, Interpreter, Value};
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;
use wasm_bindgen::prelude::*;

#[cfg(feature = "fake_bindings")]
use eldritchv2::{
    agent::fake::AgentLibraryFake, assets::fake::FakeAssetsLibrary,
    crypto::fake::CryptoLibraryFake, file::fake::FileLibraryFake, http::fake::HttpLibraryFake,
    pivot::fake::PivotLibraryFake, process::fake::ProcessLibraryFake,
    random::fake::RandomLibraryFake, regex::fake::RegexLibraryFake,
    report::fake::ReportLibraryFake, sys::fake::SysLibraryFake, time::fake::TimeLibraryFake,
};

#[wasm_bindgen]
extern "C" {
    fn repl_print(s: &str);
}

#[wasm_bindgen]
pub struct WasmRepl {
    interp: Interpreter,
    repl: Repl,
    printer: Arc<BufferPrinter>,
}

#[wasm_bindgen]
pub struct RenderState {
    prompt: String,
    buffer: String,
    cursor: usize,
    suggestions: Option<Vec<String>>,
    suggestion_idx: Option<usize>,
}

#[wasm_bindgen]
impl RenderState {
    #[wasm_bindgen(getter)]
    pub fn prompt(&self) -> String {
        self.prompt.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn buffer(&self) -> String {
        self.buffer.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn cursor(&self) -> usize {
        self.cursor
    }
    #[wasm_bindgen(getter)]
    pub fn suggestions(&self) -> Option<Vec<JsValue>> {
        self.suggestions
            .as_ref()
            .map(|v| v.iter().map(|s| JsValue::from_str(s)).collect())
    }
    #[wasm_bindgen(getter)]
    pub fn suggestion_idx(&self) -> Option<usize> {
        self.suggestion_idx
    }
}

#[wasm_bindgen]
pub struct ExecutionResult {
    output: Option<String>,
    echo: Option<String>,
    clear: bool,
}

#[wasm_bindgen]
impl ExecutionResult {
    #[wasm_bindgen(getter)]
    pub fn output(&self) -> Option<String> {
        self.output.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn echo(&self) -> Option<String> {
        self.echo.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn clear(&self) -> bool {
        self.clear
    }
}

#[wasm_bindgen]
impl WasmRepl {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmRepl {
        let printer = Arc::new(BufferPrinter::new());
        let mut interp = Interpreter::new_with_printer(printer.clone());

        #[cfg(feature = "fake_bindings")]
        {
            interp.register_lib(FileLibraryFake::default());
            interp.register_lib(ProcessLibraryFake::default());
            interp.register_lib(SysLibraryFake::default());
            interp.register_lib(HttpLibraryFake::default());
            interp.register_lib(CryptoLibraryFake::default());
            interp.register_lib(AgentLibraryFake::default());
            interp.register_lib(FakeAssetsLibrary::default());
            interp.register_lib(PivotLibraryFake::default());
            interp.register_lib(RandomLibraryFake::default());
            interp.register_lib(RegexLibraryFake::default());
            interp.register_lib(ReportLibraryFake::default());
            interp.register_lib(TimeLibraryFake::default());
        }

        WasmRepl {
            interp,
            repl: Repl::new(),
            printer,
        }
    }

    pub fn load_history(&mut self, js_history: Vec<JsValue>) {
        let history: Vec<String> = js_history.iter().filter_map(|x| x.as_string()).collect();
        self.repl.load_history(history);
    }

    pub fn get_history(&self) -> Vec<JsValue> {
        self.repl
            .get_history()
            .iter()
            .map(|s| JsValue::from_str(s))
            .collect()
    }

    pub fn get_state(&self) -> RenderState {
        let s = self.repl.get_render_state();
        RenderState {
            prompt: s.prompt,
            buffer: s.buffer,
            cursor: s.cursor,
            suggestions: s.suggestions,
            suggestion_idx: s.suggestion_idx,
        }
    }

    pub fn handle_key(
        &mut self,
        key: &str,
        ctrl: bool,
        _alt: bool,
        meta: bool,
        shift: bool,
    ) -> ExecutionResult {
        let input = match key {
            "Enter" => {
                if shift {
                    Input::ForceEnter
                } else {
                    Input::Enter
                }
            }
            "Backspace" => Input::Backspace,
            "Delete" => Input::Delete,
            "ArrowLeft" => Input::Left,
            "ArrowRight" => Input::Right,
            "ArrowUp" => Input::Up,
            "ArrowDown" => Input::Down,
            "Home" => Input::Home,
            "End" => Input::End,
            "Tab" => Input::Tab,
            "a" if ctrl => Input::Home,
            "e" if ctrl => Input::End,
            "c" if ctrl => Input::Cancel,
            "l" if ctrl => Input::ClearScreen,
            "u" if ctrl => Input::KillLine,
            "k" if ctrl => Input::KillToEnd,
            "w" if ctrl => Input::WordBackspace,
            "r" if ctrl => Input::HistorySearch,
            " " if ctrl => Input::ForceComplete,
            _ => {
                // If ctrl is pressed but not matched above, we might still want to pass it through if it's a char?
                // But generally ctrl+char are commands.
                // The original code:
                if key.len() == 1 && !ctrl && !meta {
                    Input::Char(key.chars().next().unwrap())
                } else if key.len() == 1 && (ctrl || meta) {
                    // For search, we might need ctrl chars later, but for now strict mapping
                    return ExecutionResult {
                        output: None,
                        echo: None,
                        clear: false,
                    };
                } else {
                    return ExecutionResult {
                        output: None,
                        echo: None,
                        clear: false,
                    };
                }
            }
        };

        self.process_input(input)
    }

    pub fn handle_paste(&mut self, text: &str) -> ExecutionResult {
        let mut final_res = ExecutionResult {
            output: None,
            echo: None,
            clear: false,
        };
        let mut echo_acc = String::new();
        let mut output_acc = String::new();

        if !text.contains('\n') {
            for c in text.chars() {
                self.repl.handle_input(Input::Char(c));
            }
            return final_res;
        }

        for c in text.chars() {
            let input = if c == '\n' {
                Input::Enter
            } else {
                Input::Char(c)
            };
            let res = self.process_input(input);

            if let Some(e) = res.echo {
                if !echo_acc.is_empty() {
                    echo_acc.push('\n');
                }
                echo_acc.push_str(&e);
            }
            if let Some(o) = res.output {
                if !output_acc.is_empty() {
                    output_acc.push('\n');
                }
                output_acc.push_str(&o);
            }
        }

        if !echo_acc.is_empty() {
            final_res.echo = Some(echo_acc);
        }
        if !output_acc.is_empty() {
            final_res.output = Some(output_acc);
        }
        final_res
    }

    fn process_input(&mut self, input: Input) -> ExecutionResult {
        match self.repl.handle_input(input) {
            ReplAction::Submit {
                code,
                last_line,
                prompt,
            } => {
                let echo = format!("{}{}", prompt, last_line);
                let res = self.execute(&code);
                ExecutionResult {
                    echo: Some(echo),
                    output: res.output,
                    clear: false,
                }
            }
            ReplAction::AcceptLine { line, prompt } => ExecutionResult {
                output: None,
                echo: Some(format!("{}{}", prompt, line)),
                clear: false,
            },
            ReplAction::Render => ExecutionResult {
                output: None,
                echo: None,
                clear: false,
            },
            ReplAction::ClearScreen => ExecutionResult {
                output: None,
                echo: None,
                clear: true,
            },
            ReplAction::Complete => {
                let s = self.repl.get_render_state();
                let (start, completions) = self.interp.complete(&s.buffer, s.cursor);
                self.repl.set_suggestions(completions, start);
                ExecutionResult {
                    output: None,
                    echo: None,
                    clear: false,
                }
            }
            ReplAction::None => ExecutionResult {
                output: None,
                echo: None,
                clear: false,
            },
            ReplAction::Quit => ExecutionResult {
                output: Some("Use 'exit' or close tab.".to_string()),
                echo: None,
                clear: false,
            },
        }
    }

    fn execute(&mut self, code: &str) -> ExecutionResult {
        self.printer.clear();

        match self.interp.interpret(code) {
            Ok(v) => {
                let mut out = self.printer.read();

                if let Value::None = v {
                    // Do not print None
                } else {
                    if !out.is_empty() {
                        out.push('\n');
                    }
                    out.push_str(&v.to_string());
                }

                if out.is_empty() {
                    ExecutionResult {
                        output: None,
                        echo: None,
                        clear: false,
                    }
                } else {
                    ExecutionResult {
                        output: Some(out),
                        echo: None,
                        clear: false,
                    }
                }
            }
            Err(e) => {
                let mut out = self.printer.read();
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str(&format!("Error: {}", e));
                ExecutionResult {
                    output: Some(out),
                    echo: None,
                    clear: false,
                }
            }
        }
    }
}
