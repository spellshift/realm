use eldritch_core::{ExprKind, Stmt, StmtKind};
use eldritchv2::Interpreter;
use lsp_types::{Diagnostic, DiagnosticSeverity};

use super::{utils::span_to_range, visitors::visit_stmts, LintRule};

pub struct DeprecationRule;

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
