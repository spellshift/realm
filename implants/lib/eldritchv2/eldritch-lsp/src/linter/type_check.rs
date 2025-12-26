use eldritch_core::{Argument, ExprKind, Stmt, Value};
use eldritchv2::Interpreter;
use lsp_types::{Diagnostic, DiagnosticSeverity};
use std::collections::BTreeMap;

use super::{utils::span_to_range, visitors::visit_stmts_exprs, LintRule};

pub struct TypeCheckRule;

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
                                    message: format!(
                                        "TypeError: Unsupported operand types for +: '{}' and '{}'",
                                        l_type, r_type
                                    ),
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
                                        message: format!(
                                            "AttributeError: '{}' object has no attribute '{}'",
                                            foreign_obj.type_name(),
                                            method_name
                                        ),
                                        ..Default::default()
                                    });
                                } else {
                                    // 2. Check arguments if signature is available
                                    if let Some(sig) = foreign_obj.get_method_signature(method_name)
                                    {
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
    span: eldritch_core::Span,
    source: &str,
    diags: &mut Vec<Diagnostic>,
) {
    let mut positional_count = 0;
    let mut kw_args_present = BTreeMap::new();

    for arg in args {
        match arg {
            Argument::Positional(_) => positional_count += 1,
            Argument::Keyword(k, _) => {
                kw_args_present.insert(k.clone(), ());
            }
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
                message: format!(
                    "TypeError: Missing required arguments: {}",
                    missing.join(", ")
                ),
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
    diags: &mut Vec<Diagnostic>,
) {
    if let Some(expected_type_raw) = &param.type_name {
        // Clean up expected type (e.g. "Option < String >" -> "String", "Vec < String >" -> "List")
        let expected_type = clean_type_name(expected_type_raw);
        if let Some(actual_type) = infer_type(expr) {
            if !is_type_compatible(&expected_type, actual_type) {
                diags.push(Diagnostic {
                    range: span_to_range(expr.span, source),
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: format!(
                        "TypeError: Argument '{}' expected type '{}', got '{}'",
                        param.name, expected_type, actual_type
                    ),
                    ..Default::default()
                });
            }
        }
    }
}

fn clean_type_name(raw: &str) -> String {
    let raw = raw
        .replace("alloc :: string :: ", "")
        .replace("alloc :: vec :: ", "");
    if raw.contains("Option <") {
        return raw
            .replace("Option <", "")
            .replace(">", "")
            .trim()
            .to_string();
    }
    if raw.contains("Vec <") {
        return "List".to_string(); // Approximate Vec as List
    }
    if raw.contains("BTreeMap <") {
        return "Dictionary".to_string();
    }
    raw.replace("String", "str")
        .replace("i64", "int")
        .replace("f64", "float")
        .replace("bool", "bool")
        .replace("Vec < u8 >", "bytes")
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
            _ => None,
        },
        ExprKind::List(_) => Some("List"),
        ExprKind::Dictionary(_) => Some("Dictionary"),
        ExprKind::Tuple(_) => Some("Tuple"),
        ExprKind::Set(_) => Some("Set"),
        _ => None,
    }
}
