use eldritch_core::{ExprKind, Span, Stmt, StmtKind, Value, Argument};
use eldritchv2::Interpreter;
use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};
use std::collections::BTreeMap;

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
            ],
        }
    }

    pub fn check(&self, stmts: &[Stmt], source: &str) -> Vec<Diagnostic> {
        let interp = Interpreter::new().with_default_libs().with_fake_agent();
        let mut diagnostics = Vec::new();
        for rule in &self.rules {
            diagnostics.extend(rule.check(stmts, source, &interp));
        }
        diagnostics
    }
}

struct DeprecationRule;
impl LintRule for DeprecationRule {
    fn name(&self) -> &str {
        "deprecation"
    }

    fn check(&self, stmts: &[Stmt], source: &str, _interp: &Interpreter) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        visit_stmts(stmts, &mut |stmt| {
            if let StmtKind::Expression(expr) = &stmt.kind {
                // Check for os.system -> sys.exec
                if let ExprKind::Call(callee, _) = &expr.kind {
                    if let ExprKind::GetAttr(obj, name) = &callee.kind {
                        if let ExprKind::Identifier(v) = &obj.kind {
                            if v == "os" && name == "system" {
                                diags.push(Diagnostic {
                                    range: span_to_range(expr.span, source),
                                    severity: Some(DiagnosticSeverity::WARNING),
                                    message: "os.system is deprecated. Use sys.exec instead."
                                        .to_string(),
                                    ..Default::default()
                                });
                            }
                        }
                    }
                }
            }
        });
        diags
    }
}

struct TypeCheckRule;
impl LintRule for TypeCheckRule {
    fn name(&self) -> &str {
        "type_check"
    }

    fn check(&self, stmts: &[Stmt], source: &str, interp: &Interpreter) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        visit_stmts_exprs(stmts, &mut |expr| {
            // Check invalid binary ops
            if let ExprKind::BinaryOp(lhs, op, rhs) = &expr.kind {
                 if let (Some(l_type), Some(r_type)) = (infer_type(lhs), infer_type(rhs)) {
                     match op {
                         eldritch_core::TokenKind::Plus => {
                             if l_type == "List" && r_type == "String" {
                                 diags.push(Diagnostic {
                                     range: span_to_range(expr.span, source),
                                     severity: Some(DiagnosticSeverity::ERROR),
                                     message: format!("TypeError: Unsupported operand types for +: '{}' and '{}'", l_type, r_type),
                                     ..Default::default()
                                 });
                             }
                         }
                         _ => {}
                     }
                 }
            }

            // Check calls
            if let ExprKind::Call(callee, args) = &expr.kind {
                if let ExprKind::GetAttr(obj, method_name) = &callee.kind {
                    if let ExprKind::Identifier(var_name) = &obj.kind {
                        // Lookup variable in interpreter (libraries are globals)
                        if let Some(val) = interp.lookup_variable(var_name) {
                            if let Value::Foreign(foreign_obj) = val {
                                // 1. Check method existence
                                let methods = foreign_obj.method_names();
                                if !methods.contains(method_name) {
                                    diags.push(Diagnostic {
                                        range: span_to_range(callee.span, source),
                                        severity: Some(DiagnosticSeverity::ERROR),
                                        message: format!("AttributeError: '{}' object has no attribute '{}'", foreign_obj.type_name(), method_name),
                                        ..Default::default()
                                    });
                                } else {
                                    // 2. Check arguments if signature is available
                                    if let Some(sig) = foreign_obj.get_method_signature(method_name) {
                                        check_arguments(&sig, args, expr.span, source, &mut diags);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
        diags
    }
}

fn check_arguments(
    sig: &eldritch_core::MethodSignature,
    args: &[Argument],
    span: Span,
    source: &str,
    diags: &mut Vec<Diagnostic>
) {
    let mut positional_count = 0;
    let mut kw_args_present = BTreeMap::new();

    for arg in args {
        match arg {
            Argument::Positional(_) => positional_count += 1,
            Argument::Keyword(k, _) => { kw_args_present.insert(k.clone(), ()); },
            _ => return, // Give up on *args / **kwargs for now
        }
    }

    // Check argument count
    // Count required positionals
    let mut required_params = 0;
    let mut param_names = Vec::new();
    for p in &sig.params {
        if !p.is_optional && !p.is_kwargs && !p.is_variadic {
            required_params += 1;
        }
        param_names.push(p.name.clone());
    }

    // This is a naive check, doesn't handle mix of positional + keyword for same param perfectly
    // But good enough for basic cases
    if positional_count < required_params {
         // Check if missing are covered by kwargs
         let mut missing = Vec::new();
         for i in positional_count..sig.params.len() {
             let p = &sig.params[i];
             if !p.is_optional && !kw_args_present.contains_key(&p.name) {
                 missing.push(p.name.clone());
             }
         }

         if !missing.is_empty() {
             diags.push(Diagnostic {
                 range: span_to_range(span, source),
                 severity: Some(DiagnosticSeverity::ERROR),
                 message: format!("TypeError: Missing required arguments: {}", missing.join(", ")),
                 ..Default::default()
             });
         }
    }

    // Type checking for arguments
    let mut param_idx = 0;
    for arg in args {
        match arg {
            Argument::Positional(expr) => {
                if param_idx < sig.params.len() {
                    let param = &sig.params[param_idx];
                    check_arg_type(param, expr, source, diags);
                }
                param_idx += 1;
            }
            Argument::Keyword(name, expr) => {
                // Find param by name
                if let Some(param) = sig.params.iter().find(|p| &p.name == name) {
                     check_arg_type(param, expr, source, diags);
                }
            }
            _ => {}
        }
    }
}

fn check_arg_type(
    param: &eldritch_core::ParameterSignature,
    expr: &eldritch_core::Expr,
    source: &str,
    diags: &mut Vec<Diagnostic>
) {
    if let Some(expected_type_raw) = &param.type_name {
        // Clean up expected type (e.g. "Option < String >" -> "String", "Vec < String >" -> "List")
        let expected_type = clean_type_name(expected_type_raw);
        if let Some(actual_type) = infer_type(expr) {
            if !is_type_compatible(&expected_type, actual_type) {
                 diags.push(Diagnostic {
                     range: span_to_range(expr.span, source),
                     severity: Some(DiagnosticSeverity::ERROR),
                     message: format!("TypeError: Argument '{}' expected type '{}', got '{}'", param.name, expected_type, actual_type),
                     ..Default::default()
                 });
            }
        }
    }
}

fn clean_type_name(raw: &str) -> String {
    let raw = raw.replace("alloc :: string :: ", "").replace("alloc :: vec :: ", "");
    if raw.contains("Option <") {
        return raw.replace("Option <", "").replace(">", "").trim().to_string();
    }
    if raw.contains("Vec <") {
        return "List".to_string(); // Approximate Vec as List
    }
    if raw.contains("BTreeMap <") {
        return "Dictionary".to_string();
    }
    raw.replace("String", "str").replace("i64", "int").replace("f64", "float").replace("bool", "bool").replace("Vec < u8 >", "bytes")
}

fn is_type_compatible(expected: &str, actual: &str) -> bool {
    match expected {
        "str" | "String" => actual == "String",
        "int" | "i64" => actual == "Int",
        "float" | "f64" => actual == "Float" || actual == "Int", // Allow Int for Float
        "bool" => actual == "Bool",
        "List" => actual == "List",
        "Dictionary" => actual == "Dictionary",
        _ => true, // Unknown expected type, pass
    }
}

// Helper to infer simple types
fn infer_type(expr: &eldritch_core::Expr) -> Option<&'static str> {
    match &expr.kind {
        ExprKind::Literal(val) => match val {
            Value::String(_) => Some("String"),
            Value::List(_) => Some("List"),
            Value::Dictionary(_) => Some("Dictionary"),
            Value::Int(_) => Some("Int"),
            _ => None
        },
        ExprKind::List(_) => Some("List"),
        ExprKind::Dictionary(_) => Some("Dictionary"),
        ExprKind::Tuple(_) => Some("Tuple"),
        ExprKind::Set(_) => Some("Set"),
        _ => None
    }
}

fn visit_stmts<F>(stmts: &[Stmt], callback: &mut F)
where
    F: FnMut(&Stmt),
{
    for stmt in stmts {
        callback(stmt);
        match &stmt.kind {
            StmtKind::If(_, then_branch, else_branch) => {
                visit_stmts(then_branch, callback);
                if let Some(else_b) = else_branch {
                    visit_stmts(else_b, callback);
                }
            }
            StmtKind::For(_, _, body) => {
                visit_stmts(body, callback);
            }
            StmtKind::Def(_, _, _, body) => {
                visit_stmts(body, callback);
            }
            // Block variant does not exist in StmtKind, blocks are Vec<Stmt>
            _ => {}
        }
    }
}

fn visit_stmts_exprs<F>(stmts: &[Stmt], callback: &mut F)
where
    F: FnMut(&eldritch_core::Expr),
{
    for stmt in stmts {
        match &stmt.kind {
            StmtKind::Expression(expr) => {
                visit_expr(expr, callback);
            }
            StmtKind::Assignment(lhs, _, rhs) => {
                visit_expr(lhs, callback);
                visit_expr(rhs, callback);
            }
            StmtKind::If(cond, then_b, else_b) => {
                visit_expr(cond, callback);
                visit_stmts_exprs(then_b, callback);
                if let Some(b) = else_b {
                     visit_stmts_exprs(b, callback);
                }
            }
             StmtKind::For(_, iter, body) => {
                visit_expr(iter, callback);
                visit_stmts_exprs(body, callback);
            }
             StmtKind::Def(_, _, _, body) => {
                visit_stmts_exprs(body, callback);
            }
            _ => {}
        }
    }
}

fn visit_expr<F>(expr: &eldritch_core::Expr, callback: &mut F)
where
    F: FnMut(&eldritch_core::Expr),
{
    callback(expr);
    match &expr.kind {
        ExprKind::BinaryOp(lhs, _, rhs) => {
            visit_expr(lhs, callback);
            visit_expr(rhs, callback);
        }
        ExprKind::Call(callee, args) => {
            visit_expr(callee, callback);
            for arg in args {
                match arg {
                     Argument::Positional(e) => visit_expr(e, callback),
                     Argument::Keyword(_, e) => visit_expr(e, callback),
                     _ => {}
                }
            }
        }
        _ => {}
    }
}

pub fn span_to_range(span: Span, source: &str) -> Range {
    let start_line_idx = span.line.saturating_sub(1);
    let mut current_line = 0;
    let mut offset = 0;
    let mut line_start_offset = 0;

    for (i, b) in source.bytes().enumerate() {
        if current_line == start_line_idx {
            line_start_offset = offset;
            break;
        }
        if b == b'\n' {
            current_line += 1;
            offset = i + 1;
        }
    }
    if current_line < start_line_idx {
        line_start_offset = offset;
    }

    let start_col = span.start.saturating_sub(line_start_offset);
    let (end_line, end_col) = byte_offset_to_pos(span.end, source);

    Range::new(
        Position::new(start_line_idx as u32, start_col as u32),
        Position::new(end_line as u32, end_col as u32),
    )
}

fn byte_offset_to_pos(offset: usize, source: &str) -> (usize, usize) {
    let mut line = 0;
    let mut last_line_start = 0;
    for (i, b) in source.bytes().enumerate() {
        if i == offset {
            return (line, i - last_line_start);
        }
        if b == b'\n' {
            line += 1;
            last_line_start = i + 1;
        }
    }
    (line, offset.saturating_sub(last_line_start))
}

#[cfg(test)]
mod tests {
    use super::*;
    use eldritch_core::{Lexer, Parser};

    #[test]
    fn test_deprecation_rule() {
        let code = "os.system('ls')";
        let mut lexer = Lexer::new(code.to_string());
        let tokens = lexer.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse().unwrap();

        let linter = Linter::new();
        let diagnostics = linter.check(&stmts, code);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::WARNING));
        assert!(diagnostics[0].message.contains("os.system is deprecated"));
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
         assert!(diagnostics[0].message.contains("has no attribute 'not_a_method'"));
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
         let found = diagnostics.iter().any(|d| d.message.contains("TypeError: Argument 'path' expected type 'str', got 'Dictionary'"));
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
         let found = diagnostics.iter().any(|d| d.message.contains("TypeError: Missing required arguments: path, args"));
         assert!(found, "Expected missing args error not found. Diagnostics: {:?}", diagnostics);
    }
}
