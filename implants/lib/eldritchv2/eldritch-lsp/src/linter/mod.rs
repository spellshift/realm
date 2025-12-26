use eldritch_core::{Stmt, Value};
use eldritchv2::Interpreter;
use lsp_types::Diagnostic;
use spin::RwLock;
use std::collections::BTreeMap;
use std::sync::Arc;

mod deprecation;
mod type_check;
mod undefined_symbol;
pub mod utils;
mod visitors;

use self::deprecation::DeprecationRule;
use self::type_check::TypeCheckRule;
use self::undefined_symbol::UndefinedSymbolRule;

pub trait LintRule {
    #[allow(dead_code)]
    fn name(&self) -> &str;
    fn check(&self, stmts: &[Stmt], source: &str, interp: &Interpreter) -> Vec<Diagnostic>;
}

pub struct Linter {
    rules: Vec<Box<dyn LintRule + Send + Sync>>,
}

impl Linter {
    pub fn new() -> Self {
        Self {
            rules: vec![
                Box::new(DeprecationRule),
                Box::new(TypeCheckRule),
                Box::new(UndefinedSymbolRule),
            ],
        }
    }

    pub fn check(&self, stmts: &[Stmt], source: &str) -> Vec<Diagnostic> {
        let mut interp = Interpreter::new().with_default_libs().with_fake_agent();

        // Inject input_params
        #[allow(clippy::mutable_key_type)]
        let params = BTreeMap::new();
        let params_val = Value::Dictionary(Arc::new(RwLock::new(params)));
        interp.define_variable("input_params", params_val);

        let mut diagnostics = Vec::new();
        for rule in &self.rules {
            diagnostics.extend(rule.check(stmts, source, &interp));
        }
        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eldritch_core::{Lexer, Parser};
    use lsp_types::DiagnosticSeverity;

    #[test]
    fn test_deprecation_rule() {
        let code = "os.system('ls')";
        let mut lexer = Lexer::new(code.to_string());
        let tokens = lexer.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse().unwrap();

        let linter = Linter::new();
        let diagnostics = linter.check(&stmts, code);

        // We expect at least one diagnostic warning about deprecation
        let warnings: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.message.contains("os.system is deprecated"))
            .collect();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].severity, Some(DiagnosticSeverity::WARNING));
    }

    #[test]
    fn test_type_check_missing_method() {
        let code = "agent.not_a_method()";
        let mut lexer = Lexer::new(code.to_string());
        let tokens = lexer.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse().unwrap();

        let linter = Linter::new();
        let diagnostics = linter.check(&stmts, code);

        assert!(!diagnostics.is_empty());
        assert!(diagnostics[0]
            .message
            .contains("has no attribute 'not_a_method'"));
    }

    #[test]
    fn test_type_check_binary_op() {
        let code = "x = [] + \"a\"";
        let mut lexer = Lexer::new(code.to_string());
        let tokens = lexer.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse().unwrap();

        let linter = Linter::new();
        let diagnostics = linter.check(&stmts, code);

        assert!(!diagnostics.is_empty());
        assert!(diagnostics[0].message.contains("Unsupported operand types"));
    }

    #[test]
    fn test_type_check_wrong_arg_type() {
        let code = "sys.exec({'what': 'adict'})";
        let mut lexer = Lexer::new(code.to_string());
        let tokens = lexer.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse().unwrap();

        let linter = Linter::new();
        let diagnostics = linter.check(&stmts, code);

        if diagnostics.is_empty() {
            panic!("No diagnostics found");
        }
        println!("Diagnostics: {:?}", diagnostics);

        assert!(!diagnostics.is_empty());
        let found = diagnostics.iter().any(|d| {
            d.message
                .contains("TypeError: Argument 'path' expected type 'str', got 'Dictionary'")
        });
        assert!(found, "Expected error not found");
    }

    #[test]
    fn test_type_check_missing_args() {
        let code = "sys.exec()";
        let mut lexer = Lexer::new(code.to_string());
        let tokens = lexer.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse().unwrap();

        let linter = Linter::new();
        let diagnostics = linter.check(&stmts, code);

        assert!(!diagnostics.is_empty());
        // sys.exec takes path, args.
        let found = diagnostics.iter().any(|d| {
            d.message
                .contains("TypeError: Missing required arguments: path, args")
        });
        assert!(
            found,
            "Expected missing args error not found. Diagnostics: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_undefined_symbol_basic() {
        let code = "print(not_defined)";
        let mut lexer = Lexer::new(code.to_string());
        let tokens = lexer.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse().unwrap();

        let linter = Linter::new();
        let diagnostics = linter.check(&stmts, code);

        assert!(!diagnostics.is_empty());
        assert!(diagnostics.iter().any(|d| d
            .message
            .contains("NameError: name 'not_defined' is not defined")));
    }

    #[test]
    fn test_undefined_symbol_defined() {
        let code = "x = 1; print(x)";
        let mut lexer = Lexer::new(code.to_string());
        let tokens = lexer.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse().unwrap();

        let linter = Linter::new();
        let diagnostics = linter.check(&stmts, code);

        // Filter out irrelevant diagnostics (like type checks if any)
        let name_errors: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.message.contains("NameError"))
            .collect();
        assert!(name_errors.is_empty(), "Found NameError: {:?}", name_errors);
    }

    #[test]
    fn test_undefined_symbol_function_scope() {
        let code = "def foo(a):\n    print(a)\n    print(b) # Error";
        let mut lexer = Lexer::new(code.to_string());
        let tokens = lexer.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse().unwrap();

        let linter = Linter::new();
        let diagnostics = linter.check(&stmts, code);

        let name_errors: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.message.contains("NameError"))
            .collect();
        assert_eq!(name_errors.len(), 1);
        assert!(name_errors[0].message.contains("'b' is not defined"));
    }

    #[test]
    fn test_undefined_symbol_for_loop() {
        let code = "for i in [1, 2]:\n    print(i)\nprint(i) # Error: Loop var local to loop";
        let mut lexer = Lexer::new(code.to_string());
        let tokens = lexer.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse().unwrap();

        let linter = Linter::new();
        let diagnostics = linter.check(&stmts, code);

        let name_errors: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.message.contains("NameError"))
            .collect();
        assert_eq!(name_errors.len(), 1);
        assert!(name_errors[0].message.contains("'i' is not defined"));
    }

    #[test]
    fn test_input_params_defined() {
        let code = "print(input_params)";
        let mut lexer = Lexer::new(code.to_string());
        let tokens = lexer.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse().unwrap();

        let linter = Linter::new();
        let diagnostics = linter.check(&stmts, code);

        let name_errors: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.message.contains("NameError"))
            .collect();
        assert!(
            name_errors.is_empty(),
            "Found NameError for input_params: {:?}",
            name_errors
        );
    }
}
