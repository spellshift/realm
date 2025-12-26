use eldritch_core::{ExprKind, Stmt, StmtKind, Value};
use eldritchv2::Interpreter;
use lsp_types::{Diagnostic, DiagnosticSeverity};

use super::{utils::span_to_range, visitors::visit_stmts, LintRule};

pub struct DeprecationRule;

impl LintRule for DeprecationRule {
    fn name(&self) -> &str {
        "deprecation"
    }

    fn check(&self, stmts: &[Stmt], source: &str, interp: &Interpreter) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        visit_stmts(stmts, &mut |stmt| {
            if let StmtKind::Expression(expr) = &stmt.kind {
                // Check for deprecated method calls
                if let ExprKind::Call(callee, _) = &expr.kind {
                    if let ExprKind::GetAttr(obj, name) = &callee.kind {
                        // Attempt to resolve the object
                        if let ExprKind::Identifier(var_name) = &obj.kind {
                            // We can only check global variables or imported modules if they are present in the interpreter
                            // This is a best-effort check since we don't have full type inference.
                            if let Some(val) = interp.lookup_variable(var_name) {
                                if let Value::Foreign(foreign_obj) = val {
                                    if let Some(sig) = foreign_obj.get_method_signature(name) {
                                        if let Some(reason) = sig.deprecated {
                                            diags.push(Diagnostic {
                                                range: span_to_range(expr.span, source),
                                                severity: Some(DiagnosticSeverity::WARNING),
                                                message: format!("Method '{}.{}' is deprecated: {}", var_name, name, reason),
                                                ..Default::default()
                                            });
                                        }
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
