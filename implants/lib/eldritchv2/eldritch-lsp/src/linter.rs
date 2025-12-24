use eldritch_core::{ExprKind, Span, Stmt, StmtKind};
use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

pub trait LintRule {
    fn name(&self) -> &str;
    fn check(&self, stmts: &[Stmt], source: &str) -> Vec<Diagnostic>;
}

pub struct Linter {
    rules: Vec<Box<dyn LintRule + Send + Sync>>,
}

impl Linter {
    pub fn new() -> Self {
        Self {
            rules: vec![Box::new(DeprecationRule)],
        }
    }

    pub fn check(&self, stmts: &[Stmt], source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for rule in &self.rules {
            diagnostics.extend(rule.check(stmts, source));
        }
        diagnostics
    }
}

struct DeprecationRule;
impl LintRule for DeprecationRule {
    fn name(&self) -> &str {
        "deprecation"
    }

    fn check(&self, stmts: &[Stmt], source: &str) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        visit_stmts(stmts, &mut |stmt| {
            if let StmtKind::Expression(expr) = &stmt.kind {
                // Check for os.system -> sys.exec
                // ExprKind::Call(callee, args)
                if let ExprKind::Call(callee, _) = &expr.kind {
                    // ExprKind::GetAttr(obj, name)
                    if let ExprKind::GetAttr(obj, name) = &callee.kind {
                        // ExprKind::Identifier(name)
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

pub fn span_to_range(span: Span, source: &str) -> Range {
    // Span: start (byte), end (byte), line (1-based)
    // We need to calculate columns.

    // Find start line start-offset
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
    // If we reached here without breaking, use last offset
    if current_line < start_line_idx {
        line_start_offset = offset;
    }

    let start_col = span.start.saturating_sub(line_start_offset);

    // For end line, we rely on span.end.
    // We need to find which line span.end is on.
    // Simpler: just count newlines between start and end?
    // Or just convert byte offset to (line, col)

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
}
