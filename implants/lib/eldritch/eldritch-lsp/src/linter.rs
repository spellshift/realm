use eldritch_core::Stmt;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

/// A trait defining a single linting rule.
///
/// Rules follow a two-phase check for performance:
/// 1. `should_lint`: A lightweight string check.
/// 2. `check`: A deeper AST or full-text analysis.
pub trait LintRule: Send + Sync {
    /// Returns the unique name of the rule (e.g., "no-forbidden-runes").
    fn name(&self) -> &'static str;

    /// Quickly determines if this rule is relevant for the given source code.
    /// This optimization prevents expensive AST traversals for rules that definitely won't match.
    fn should_lint(&self, _source: &str) -> bool {
        // Default to true for safety, but implementations should override this
        // with fast checks (e.g., source.contains("forbidden"))
        true
    }

    /// Runs the linting logic.
    ///
    /// * `ast`: The parsed AST, if available. Some lints might work purely on text.
    /// * `source`: The raw source code.
    fn check(&self, ast: Option<&[Stmt]>, source: &str) -> Vec<Diagnostic>;
}

/// Registry to hold and manage all active lint rules.
pub struct LintRegistry {
    rules: Vec<Box<dyn LintRule>>,
}

impl LintRegistry {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn register<R: LintRule + 'static>(&mut self, rule: R) {
        self.rules.push(Box::new(rule));
    }

    /// Runs all registered rules against the source/AST.
    pub fn run(&self, ast: Option<&[Stmt]>, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for rule in &self.rules {
            if rule.should_lint(source) {
                diagnostics.extend(rule.check(ast, source));
            }
        }

        diagnostics
    }
}

impl Default for LintRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        registry.register(NoForbiddenRunes);
        registry
    }
}

// --- Example Rule: NoForbiddenRunes ---

struct NoForbiddenRunes;

impl LintRule for NoForbiddenRunes {
    fn name(&self) -> &'static str {
        "no-forbidden-runes"
    }

    fn should_lint(&self, source: &str) -> bool {
        source.contains("vecna")
    }

    fn check(&self, _ast: Option<&[Stmt]>, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (line_idx, line) in source.lines().enumerate() {
            if let Some(col_idx) = line.find("vecna") {
                let range = Range {
                    start: Position {
                        line: line_idx as u32,
                        character: col_idx as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (col_idx + 5) as u32, // "vecna".len()
                    },
                };

                diagnostics.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::WARNING),
                    code: Some(tower_lsp::lsp_types::NumberOrString::String(
                        self.name().to_string(),
                    )),
                    source: Some("eldritch-lint".to_string()),
                    message: "The use of 'vecna' is forbidden. It summons unwanted attention.".to_string(),
                    ..Default::default()
                });
            }
        }

        diagnostics
    }
}
