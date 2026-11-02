use crate::linter::LintRegistry;
use crate::stdlib::{FuncSpec, MethodSpec, ParamSpec, StdlibIndex};
use eldritch_core::{
    Argument, Expr, ExprKind, FStringSegment, Lexer, Param, Parser, Span, Stmt, StmtKind, Token,
    TokenKind, Value,
};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use tower_lsp::lsp_types::*;

/// Maps 0-based character offsets from `eldritch-core` `Span`s to 0-based UTF-16 LSP `Position`s.
#[derive(Debug, Clone)]
pub struct LineIndex {
    pub chars: Vec<char>,
    pub line_starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(source: &str) -> Self {
        let chars: Vec<char> = source.chars().collect();
        let mut line_starts = vec![0];
        for (idx, &ch) in chars.iter().enumerate() {
            if ch == '\n' {
                line_starts.push(idx + 1);
            }
        }
        Self { chars, line_starts }
    }

    pub fn char_offset_to_position(&self, offset: usize) -> Position {
        let clamped = offset.min(self.chars.len());
        let line_idx = self
            .line_starts
            .partition_point(|&start| start <= clamped)
            .saturating_sub(1);
        let line_start = self.line_starts[line_idx];
        let utf16_col: u32 = self.chars[line_start..clamped]
            .iter()
            .map(|c| c.len_utf16() as u32)
            .sum();
        Position {
            line: line_idx as u32,
            character: utf16_col,
        }
    }

    pub fn position_to_char_offset(&self, pos: Position) -> usize {
        let line_idx = (pos.line as usize).min(self.line_starts.len().saturating_sub(1));
        let line_start = self.line_starts[line_idx];
        let line_end = self
            .line_starts
            .get(line_idx + 1)
            .copied()
            .unwrap_or(self.chars.len());

        let mut utf16_acc = 0u32;
        let mut offset = line_start;
        while offset < line_end && utf16_acc < pos.character {
            let ch = self.chars[offset];
            if ch == '\n' || ch == '\r' {
                break;
            }
            utf16_acc += ch.len_utf16() as u32;
            offset += 1;
        }
        offset
    }

    pub fn span_to_range(&self, span: Span) -> Range {
        if span.start < span.end && span.end <= self.chars.len() {
            return Range {
                start: self.char_offset_to_position(span.start),
                end: self.char_offset_to_position(span.end),
            };
        }

        // Zero-width or EOF span: highlight single char or non-empty line range
        if span.start <= self.chars.len() && span.start > 0 {
            let prev = span.start - 1;
            if self.chars.get(prev).copied() != Some('\n') {
                return Range {
                    start: self.char_offset_to_position(prev),
                    end: self.char_offset_to_position(span.start),
                };
            }
        }

        let line_idx = span
            .line
            .saturating_sub(1)
            .min(self.line_starts.len().saturating_sub(1));
        let line_start = self.line_starts[line_idx];
        let mut line_end = self
            .line_starts
            .get(line_idx + 1)
            .copied()
            .unwrap_or(self.chars.len());
        while line_end > line_start && matches!(self.chars[line_end - 1], '\n' | '\r') {
            line_end -= 1;
        }
        Range {
            start: self.char_offset_to_position(line_start),
            end: self.char_offset_to_position(line_end),
        }
    }

    /// Converts a statement span to only cover its first line (header), avoiding multi-line squiggles.
    pub fn first_line_range(&self, span: Span) -> Range {
        let range = self.span_to_range(span);
        if range.start.line == range.end.line {
            return range;
        }
        let line_idx = range.start.line as usize;
        let line_start = self.line_starts.get(line_idx).copied().unwrap_or(0);
        let mut line_end = self
            .line_starts
            .get(line_idx + 1)
            .copied()
            .unwrap_or(self.chars.len());
        while line_end > line_start && matches!(self.chars[line_end - 1], '\n' | '\r') {
            line_end -= 1;
        }
        Range {
            start: range.start,
            end: self.char_offset_to_position(line_end),
        }
    }

    pub fn line_text(&self, line: u32) -> String {
        let line_idx = line as usize;
        let Some(&start) = self.line_starts.get(line_idx) else {
            return String::new();
        };
        let end = self
            .line_starts
            .get(line_idx + 1)
            .copied()
            .unwrap_or(self.chars.len());
        self.chars[start..end]
            .iter()
            .copied()
            .take_while(|&c| c != '\n' && c != '\r')
            .collect()
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TomeParamDef {
    pub name: String,
    #[serde(default, rename = "type")]
    pub param_type: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub placeholder: String,
    #[serde(default)]
    pub default: Option<serde_yaml::Value>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TomeMetadata {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub tactic: String,
    #[serde(default)]
    pub paramdefs: Vec<TomeParamDef>,
}

impl TomeMetadata {
    pub fn load_for_document(doc_path: &Path) -> Option<Self> {
        let parent = doc_path.parent()?;
        for candidate in ["metadata.yml", "metadata.yaml"] {
            let meta_path = parent.join(candidate);
            if meta_path.is_file() {
                if let Ok(contents) = std::fs::read_to_string(&meta_path) {
                    if let Ok(meta) = serde_yaml::from_str::<TomeMetadata>(&contents) {
                        return Some(meta);
                    }
                }
            }
        }
        None
    }

    pub fn get_param(&self, name: &str) -> Option<&TomeParamDef> {
        self.paramdefs.iter().find(|p| p.name == name)
    }
}

#[derive(Debug, Clone)]
pub struct UserParam {
    pub name: String,
    pub ty: Option<String>,
    pub default_repr: Option<String>,
    pub is_star: bool,
    pub is_star_star: bool,
}

impl UserParam {
    pub fn required(&self) -> bool {
        !self.is_star && !self.is_star_star && self.default_repr.is_none()
    }

    pub fn format_label(&self) -> String {
        let prefix = if self.is_star_star {
            "**"
        } else if self.is_star {
            "*"
        } else {
            ""
        };
        let mut s = format!("{}{}", prefix, self.name);
        if let Some(ty) = &self.ty {
            s.push_str(&format!(": {}", ty));
        }
        if let Some(def) = &self.default_repr {
            s.push_str(&format!(" = {}", def));
        }
        s
    }
}

#[derive(Debug, Clone)]
pub struct UserFuncInfo {
    pub name: String,
    pub params: Vec<UserParam>,
    pub return_type: Option<String>,
    pub docstring: Option<String>,
    pub def_span: Span,
    pub is_top_level: bool,
}

impl UserFuncInfo {
    pub fn min_args(&self) -> usize {
        self.params.iter().filter(|p| p.required()).count()
    }

    pub fn max_args(&self) -> Option<usize> {
        if self.params.iter().any(|p| p.is_star) {
            None
        } else {
            Some(self.params.iter().filter(|p| !p.is_star_star).count())
        }
    }

    pub fn accepts_kwargs(&self) -> bool {
        self.params.iter().any(|p| p.is_star_star)
    }

    pub fn signature_string(&self) -> String {
        let params_str: Vec<String> = self.params.iter().map(|p| p.format_label()).collect();
        let ret = self.return_type.as_deref().unwrap_or("Any");
        format!("def {}({}) -> {}", self.name, params_str.join(", "), ret)
    }

    pub fn snippet(&self) -> String {
        let req_params: Vec<&UserParam> = self.params.iter().filter(|p| p.required()).collect();
        if req_params.is_empty() {
            format!("{}()$0", self.name)
        } else {
            let placeholders: Vec<String> = req_params
                .iter()
                .enumerate()
                .map(|(i, p)| format!("${{{}:{}}}", i + 1, p.name))
                .collect();
            format!("{}({})$0", self.name, placeholders.join(", "))
        }
    }

    pub fn markdown_docs(&self) -> String {
        let mut md = format!("```python\n{}\n```", self.signature_string());
        if let Some(doc) = &self.docstring {
            md.push_str("\n\n");
            md.push_str(doc.trim());
        }
        md
    }
}

#[derive(Debug, Clone)]
pub struct UserVarInfo {
    pub name: String,
    pub inferred_type: Option<String>,
    pub def_span: Span,
    pub is_top_level: bool,
    pub is_param: bool,
}

#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub source: String,
    pub line_index: LineIndex,
    pub ast: Vec<Stmt>,
    pub diagnostics: Vec<Diagnostic>,
    pub functions: BTreeMap<String, UserFuncInfo>,
    pub variables: BTreeMap<String, UserVarInfo>,
    pub referenced_input_params: BTreeSet<String>,
    pub metadata: Option<TomeMetadata>,
}

#[derive(Debug, Clone)]
struct Scope {
    vars: BTreeMap<String, UserVarInfo>,
    funcs: BTreeMap<String, UserFuncInfo>,
    in_function: bool,
    in_loop: bool,
    is_top_level: bool,
}

impl Scope {
    fn new_module() -> Self {
        Self {
            vars: BTreeMap::new(),
            funcs: BTreeMap::new(),
            in_function: false,
            in_loop: false,
            is_top_level: true,
        }
    }

    fn child(&self, in_function: bool, in_loop: bool) -> Self {
        Self {
            vars: BTreeMap::new(),
            funcs: BTreeMap::new(),
            in_function: self.in_function || in_function,
            in_loop: self.in_loop || in_loop,
            is_top_level: false,
        }
    }
}

pub fn analyze_document(
    source: &str,
    stdlib: &StdlibIndex,
    linter: &LintRegistry,
    metadata: Option<TomeMetadata>,
) -> AnalysisResult {
    let line_index = LineIndex::new(source);
    let mut diagnostics = Vec::new();

    // 1. Lexing Phase
    let mut lexer = Lexer::new(source.to_string());
    let raw_tokens = lexer.scan_tokens();

    let mut clean_tokens = Vec::with_capacity(raw_tokens.len());
    for token in raw_tokens {
        collect_token_errors(&token, &line_index, &mut diagnostics);
        if !matches!(token.kind, TokenKind::Error(_)) {
            clean_tokens.push(token);
        }
    }

    // 2. Parsing Phase
    let mut parser = Parser::new(clean_tokens);
    let (ast_stmts, parse_errors) = parser.parse();

    for err in parse_errors {
        let range = line_index.span_to_range(err.span);
        diagnostics.push(Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String("syntax-error".to_string())),
            source: Some("eldritch-lsp".to_string()),
            message: err.message,
            ..Default::default()
        });
    }

    // 3. Semantic Analysis & Linting Phase
    let mut analyzer = SemanticAnalyzer {
        line_index: &line_index,
        stdlib,
        metadata: metadata.as_ref(),
        diagnostics: &mut diagnostics,
        all_functions: BTreeMap::new(),
        all_variables: BTreeMap::new(),
        referenced_input_params: BTreeSet::new(),
        shadowed_warnings_emitted: BTreeSet::new(),
    };

    let mut module_scope = Scope::new_module();
    analyzer.analyze_block(&ast_stmts, &mut module_scope);

    let all_functions = analyzer.all_functions;
    let all_variables = analyzer.all_variables;
    let referenced_input_params = analyzer.referenced_input_params;

    // 4. Custom Lint Rules
    let custom_diags = linter.run(Some(&ast_stmts), source);
    diagnostics.extend(custom_diags);

    AnalysisResult {
        source: source.to_string(),
        line_index,
        ast: ast_stmts,
        diagnostics,
        functions: all_functions,
        variables: all_variables,
        referenced_input_params,
        metadata,
    }
}

fn collect_token_errors(token: &Token, line_index: &LineIndex, diagnostics: &mut Vec<Diagnostic>) {
    match &token.kind {
        TokenKind::Error(msg) => {
            diagnostics.push(Diagnostic {
                range: line_index.span_to_range(token.span),
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("lexer-error".to_string())),
                source: Some("eldritch-lsp".to_string()),
                message: msg.clone(),
                ..Default::default()
            });
        }
        TokenKind::FStringContent(sub_tokens) => {
            for sub in sub_tokens {
                collect_token_errors(sub, line_index, diagnostics);
            }
        }
        _ => {}
    }
}

struct SemanticAnalyzer<'a> {
    line_index: &'a LineIndex,
    stdlib: &'a StdlibIndex,
    metadata: Option<&'a TomeMetadata>,
    diagnostics: &'a mut Vec<Diagnostic>,
    all_functions: BTreeMap<String, UserFuncInfo>,
    all_variables: BTreeMap<String, UserVarInfo>,
    referenced_input_params: BTreeSet<String>,
    shadowed_warnings_emitted: BTreeSet<(String, usize)>,
}

impl<'a> SemanticAnalyzer<'a> {
    fn analyze_block(&mut self, stmts: &[Stmt], scope_stack: &mut Scope) {
        let mut scopes = vec![scope_stack.clone()];
        self.analyze_stmts_in_scopes(stmts, &mut scopes);
        if let Some(top) = scopes.into_iter().next() {
            *scope_stack = top;
        }
    }

    fn hoist_functions_in_block(&mut self, stmts: &[Stmt], scopes: &mut [Scope]) {
        let is_top_level = scopes.last().map(|s| s.is_top_level).unwrap_or(false);
        for stmt in stmts {
            if let StmtKind::Def(name, params, return_ann, body) = &stmt.kind {
                let user_params: Vec<UserParam> = params.iter().map(convert_param).collect();
                let docstring = extract_docstring(body, self.line_index, stmt.span);
                let info = UserFuncInfo {
                    name: name.clone(),
                    params: user_params,
                    return_type: return_ann.as_deref().map(format_type_expr),
                    docstring,
                    def_span: stmt.span,
                    is_top_level,
                };
                self.check_shadowing(name, stmt.span, true);
                if let Some(current) = scopes.last_mut() {
                    current.funcs.insert(name.clone(), info.clone());
                }
                self.all_functions.insert(name.clone(), info);
            }
        }
    }

    fn analyze_stmts_in_scopes(&mut self, stmts: &[Stmt], scopes: &mut Vec<Scope>) {
        // Hoist all `def` declarations in this block first (matching Eldritch's `hoist_functions`).
        self.hoist_functions_in_block(stmts, scopes);

        let mut terminated = false;
        let mut warned_unreachable = false;
        for stmt in stmts.iter() {
            if terminated && !warned_unreachable {
                warned_unreachable = true;
                self.diagnostics.push(Diagnostic {
                    range: self.line_index.first_line_range(stmt.span),
                    severity: Some(DiagnosticSeverity::WARNING),
                    code: Some(NumberOrString::String("unreachable-code".to_string())),
                    source: Some("eldritch-lsp".to_string()),
                    message: "Unreachable code after control-flow transfer (`return`, `break`, `continue`, or `fail`).".to_string(),
                    tags: Some(vec![DiagnosticTag::UNNECESSARY]),
                    ..Default::default()
                });
            }

            self.analyze_stmt(stmt, scopes);

            if is_terminating_stmt(stmt) {
                terminated = true;
            }
        }
    }

    fn analyze_stmt(&mut self, stmt: &Stmt, scopes: &mut Vec<Scope>) {
        match &stmt.kind {
            StmtKind::Expression(expr) => {
                self.analyze_expr(expr, scopes);
            }
            StmtKind::Assignment(target, annotation, value) => {
                self.analyze_expr(value, scopes);
                let inferred = annotation
                    .as_deref()
                    .map(format_type_expr)
                    .or_else(|| self.infer_expr_type(value, scopes));
                self.bind_assignment_target(target, inferred, scopes, false);
            }
            StmtKind::AugmentedAssignment(target, _op, value) => {
                self.analyze_expr(value, scopes);
                match &target.kind {
                    ExprKind::Identifier(_) | ExprKind::Index(_, _) => {
                        self.analyze_expr(target, scopes);
                    }
                    _ => {
                        self.diagnostics.push(Diagnostic {
                            range: self.line_index.span_to_range(target.span),
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(NumberOrString::String(
                                "invalid-assignment-target".to_string(),
                            )),
                            source: Some("eldritch-lsp".to_string()),
                            message: "Invalid target for augmented assignment.".to_string(),
                            ..Default::default()
                        });
                    }
                }
            }
            StmtKind::If(cond, then_branch, else_branch) => {
                self.analyze_expr(cond, scopes);
                // In Eldritch, `if`/`else` executes in the current environment without creating a separate lexical scope
                self.analyze_stmts_in_scopes(then_branch, scopes);
                if let Some(else_stmts) = else_branch {
                    self.analyze_stmts_in_scopes(else_stmts, scopes);
                }
            }
            StmtKind::For(idents, iterable, body) => {
                self.analyze_expr(iterable, scopes);
                let parent = scopes.last().cloned().unwrap_or_else(Scope::new_module);
                let mut loop_scope = parent.child(false, true);
                for ident in idents {
                    self.check_shadowing(ident, stmt.span, false);
                    let var_info = UserVarInfo {
                        name: ident.clone(),
                        inferred_type: None,
                        def_span: stmt.span,
                        is_top_level: false,
                        is_param: false,
                    };
                    loop_scope.vars.insert(ident.clone(), var_info.clone());
                    self.all_variables.entry(ident.clone()).or_insert(var_info);
                }
                scopes.push(loop_scope);
                self.analyze_stmts_in_scopes(body, scopes);
                // Propagate variables assigned inside the loop body to the enclosing scope
                // (except the loop iteration variables themselves, which Eldritch scopes per iteration).
                if let Some(finished_loop_scope) = scopes.pop() {
                    if let Some(enclosing) = scopes.last_mut() {
                        for (k, v) in finished_loop_scope.vars {
                            if !idents.contains(&k) {
                                enclosing.vars.entry(k).or_insert(v);
                            }
                        }
                    }
                }
            }
            StmtKind::Def(_name, params, _return_ann, body) => {
                for param in params {
                    if let Param::WithDefault(_, _, default_expr) = param {
                        self.analyze_expr(default_expr, scopes);
                    }
                }
                let parent = scopes.last().cloned().unwrap_or_else(Scope::new_module);
                // Inside a function body, loop state resets (`in_loop = false`, `in_function = true`).
                let mut fn_scope = Scope {
                    vars: BTreeMap::new(),
                    funcs: BTreeMap::new(),
                    in_function: true,
                    in_loop: false,
                    is_top_level: false,
                };
                let _ = parent;
                for param in params {
                    let up = convert_param(param);
                    let var_info = UserVarInfo {
                        name: up.name.clone(),
                        inferred_type: up.ty.clone(),
                        def_span: stmt.span,
                        is_top_level: false,
                        is_param: true,
                    };
                    fn_scope.vars.insert(up.name.clone(), var_info.clone());
                    self.all_variables
                        .entry(up.name.clone())
                        .or_insert(var_info);
                }
                scopes.push(fn_scope);
                self.analyze_stmts_in_scopes(body, scopes);
                scopes.pop();
            }
            StmtKind::Return(expr_opt) => {
                let in_fn = scopes.iter().any(|s| s.in_function);
                if !in_fn {
                    self.diagnostics.push(Diagnostic {
                        range: self.line_index.span_to_range(stmt.span),
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("invalid-control-flow".to_string())),
                        source: Some("eldritch-lsp".to_string()),
                        message: "`return` statement outside of a function.".to_string(),
                        ..Default::default()
                    });
                }
                if let Some(expr) = expr_opt {
                    self.analyze_expr(expr, scopes);
                }
            }
            StmtKind::Break => {
                let in_loop = scopes.last().map(|s| s.in_loop).unwrap_or(false);
                if !in_loop {
                    self.diagnostics.push(Diagnostic {
                        range: self.line_index.span_to_range(stmt.span),
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("invalid-control-flow".to_string())),
                        source: Some("eldritch-lsp".to_string()),
                        message: "`break` statement outside of a `for` loop.".to_string(),
                        ..Default::default()
                    });
                }
            }
            StmtKind::Continue => {
                let in_loop = scopes.last().map(|s| s.in_loop).unwrap_or(false);
                if !in_loop {
                    self.diagnostics.push(Diagnostic {
                        range: self.line_index.span_to_range(stmt.span),
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("invalid-control-flow".to_string())),
                        source: Some("eldritch-lsp".to_string()),
                        message: "`continue` statement outside of a `for` loop.".to_string(),
                        ..Default::default()
                    });
                }
            }
            StmtKind::Pass | StmtKind::Error(_) => {}
        }
    }

    fn bind_assignment_target(
        &mut self,
        target: &Expr,
        inferred_type: Option<String>,
        scopes: &mut Vec<Scope>,
        in_comprehension: bool,
    ) {
        match &target.kind {
            ExprKind::Identifier(name) => {
                self.check_shadowing(name, target.span, false);
                let is_top_level =
                    scopes.last().map(|s| s.is_top_level).unwrap_or(false) && !in_comprehension;
                let info = UserVarInfo {
                    name: name.clone(),
                    inferred_type,
                    def_span: target.span,
                    is_top_level,
                    is_param: false,
                };
                if let Some(current) = scopes.last_mut() {
                    current.vars.insert(name.clone(), info.clone());
                }
                self.all_variables.entry(name.clone()).or_insert(info);
            }
            ExprKind::List(elements) | ExprKind::Tuple(elements) => {
                for elem in elements {
                    self.bind_assignment_target(elem, None, scopes, in_comprehension);
                }
            }
            ExprKind::Index(obj, idx) => {
                self.analyze_expr(obj, scopes);
                self.analyze_expr(idx, scopes);
            }
            _ => {
                self.diagnostics.push(Diagnostic {
                    range: self.line_index.span_to_range(target.span),
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String(
                        "invalid-assignment-target".to_string(),
                    )),
                    source: Some("eldritch-lsp".to_string()),
                    message: "Cannot assign to this expression.".to_string(),
                    ..Default::default()
                });
            }
        }
    }

    fn check_shadowing(&mut self, name: &str, span: Span, is_func: bool) {
        let key = (name.to_string(), span.line);
        if self.shadowed_warnings_emitted.contains(&key) {
            return;
        }
        if self.stdlib.is_module(name) {
            self.shadowed_warnings_emitted.insert(key);
            let kind_str = if is_func { "Function" } else { "Variable" };
            self.diagnostics.push(Diagnostic {
                range: self.line_index.first_line_range(span),
                severity: Some(DiagnosticSeverity::WARNING),
                code: Some(NumberOrString::String("shadowed-stdlib".to_string())),
                source: Some("eldritch-lsp".to_string()),
                message: format!(
                    "{} `{}` shadows the `{}` standard library module, preventing `{}.*` calls in this scope.",
                    kind_str, name, name, name
                ),
                ..Default::default()
            });
        }
    }

    fn analyze_expr(&mut self, expr: &Expr, scopes: &mut Vec<Scope>) {
        match &expr.kind {
            ExprKind::Literal(_) | ExprKind::Error(_) => {}
            ExprKind::Identifier(name) => {
                if !self.is_defined(name, scopes) {
                    let suggestion = self.suggest_identifier(name, scopes);
                    let msg = if let Some(sug) = suggestion {
                        format!("Undefined identifier `{}`. Did you mean `{}`?", name, sug)
                    } else {
                        format!("Undefined identifier `{}`.", name)
                    };
                    self.diagnostics.push(Diagnostic {
                        range: self.line_index.span_to_range(expr.span),
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("undefined-variable".to_string())),
                        source: Some("eldritch-lsp".to_string()),
                        message: msg,
                        ..Default::default()
                    });
                }
            }
            ExprKind::BinaryOp(left, _op, right) | ExprKind::LogicalOp(left, _op, right) => {
                self.analyze_expr(left, scopes);
                self.analyze_expr(right, scopes);
            }
            ExprKind::UnaryOp(_op, operand) => {
                self.analyze_expr(operand, scopes);
            }
            ExprKind::List(items) | ExprKind::Tuple(items) | ExprKind::Set(items) => {
                for item in items {
                    self.analyze_expr(item, scopes);
                }
            }
            ExprKind::Dictionary(entries) => {
                let mut seen_keys: BTreeMap<String, Span> = BTreeMap::new();
                for (k, v) in entries {
                    self.analyze_expr(k, scopes);
                    self.analyze_expr(v, scopes);
                    if let Some(const_key) = literal_key_repr(k) {
                        if seen_keys.contains_key(&const_key) {
                            self.diagnostics.push(Diagnostic {
                                range: self.line_index.span_to_range(k.span),
                                severity: Some(DiagnosticSeverity::WARNING),
                                code: Some(NumberOrString::String(
                                    "duplicate-dict-key".to_string(),
                                )),
                                source: Some("eldritch-lsp".to_string()),
                                message: format!(
                                    "Duplicate dictionary key `{}` will overwrite an earlier entry.",
                                    const_key
                                ),
                                ..Default::default()
                            });
                        } else {
                            seen_keys.insert(const_key, k.span);
                        }
                    }
                }
            }
            ExprKind::Index(obj, idx) => {
                self.analyze_expr(obj, scopes);
                self.analyze_expr(idx, scopes);
                if let ExprKind::Identifier(obj_name) = &obj.kind {
                    if obj_name == "input_params" {
                        if let ExprKind::Literal(Value::String(param_key)) = &idx.kind {
                            self.check_input_param_key(param_key, idx.span);
                        }
                    }
                }
            }
            ExprKind::Slice(obj, start, stop, step) => {
                self.analyze_expr(obj, scopes);
                if let Some(s) = start {
                    self.analyze_expr(s, scopes);
                }
                if let Some(s) = stop {
                    self.analyze_expr(s, scopes);
                }
                if let Some(s) = step {
                    self.analyze_expr(s, scopes);
                }
            }
            ExprKind::GetAttr(obj, attr) => {
                self.analyze_getattr(obj, attr, expr.span, scopes);
            }
            ExprKind::Call(callee, args) => {
                // Analyze callee and arguments
                self.analyze_call(callee, args, expr.span, scopes);
            }
            ExprKind::ListComp {
                body,
                vars,
                iterable,
                cond,
            } => {
                self.analyze_comprehension(body, None, vars, iterable, cond.as_deref(), scopes);
            }
            ExprKind::SetComp {
                body,
                vars,
                iterable,
                cond,
            } => {
                self.analyze_comprehension(body, None, vars, iterable, cond.as_deref(), scopes);
            }
            ExprKind::DictComp {
                key,
                value,
                vars,
                iterable,
                cond,
            } => {
                self.analyze_comprehension(
                    key,
                    Some(value),
                    vars,
                    iterable,
                    cond.as_deref(),
                    scopes,
                );
            }
            ExprKind::FString(segments) => {
                for seg in segments {
                    if let FStringSegment::Expression(sub_expr) = seg {
                        self.analyze_expr(sub_expr, scopes);
                    }
                }
            }
            ExprKind::Lambda { params, body } => {
                for param in params {
                    if let Param::WithDefault(_, _, default_expr) = param {
                        self.analyze_expr(default_expr, scopes);
                    }
                }
                let parent = scopes.last().cloned().unwrap_or_else(Scope::new_module);
                let mut fn_scope = parent.child(true, false);
                for param in params {
                    let up = convert_param(param);
                    fn_scope.vars.insert(
                        up.name.clone(),
                        UserVarInfo {
                            name: up.name,
                            inferred_type: up.ty,
                            def_span: expr.span,
                            is_top_level: false,
                            is_param: true,
                        },
                    );
                }
                scopes.push(fn_scope);
                self.analyze_expr(body, scopes);
                scopes.pop();
            }
            ExprKind::If {
                cond,
                then_branch,
                else_branch,
            } => {
                self.analyze_expr(cond, scopes);
                self.analyze_expr(then_branch, scopes);
                self.analyze_expr(else_branch, scopes);
            }
        }
    }

    fn analyze_comprehension(
        &mut self,
        body1: &Expr,
        body2: Option<&Expr>,
        vars: &[String],
        iterable: &Expr,
        cond: Option<&Expr>,
        scopes: &mut Vec<Scope>,
    ) {
        self.analyze_expr(iterable, scopes);
        let parent = scopes.last().cloned().unwrap_or_else(Scope::new_module);
        let mut comp_scope = parent.child(false, true);
        for var in vars {
            comp_scope.vars.insert(
                var.clone(),
                UserVarInfo {
                    name: var.clone(),
                    inferred_type: None,
                    def_span: iterable.span,
                    is_top_level: false,
                    is_param: false,
                },
            );
        }
        scopes.push(comp_scope);
        if let Some(c) = cond {
            self.analyze_expr(c, scopes);
        }
        self.analyze_expr(body1, scopes);
        if let Some(b2) = body2 {
            self.analyze_expr(b2, scopes);
        }
        scopes.pop();
    }

    fn analyze_getattr(
        &mut self,
        obj: &Expr,
        attr: &str,
        full_span: Span,
        scopes: &mut Vec<Scope>,
    ) {
        if let ExprKind::Identifier(obj_name) = &obj.kind {
            if let Some(mod_spec) = self.stdlib.get_module(obj_name) {
                let shadowed = self.is_user_defined(obj_name, scopes);
                if shadowed {
                    if mod_spec.functions.contains_key(attr)
                        && !self.stdlib.methods.contains_key(attr)
                    {
                        self.diagnostics.push(Diagnostic {
                            range: self.line_index.span_to_range(full_span),
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(NumberOrString::String(
                                "shadowed-stdlib-access".to_string(),
                            )),
                            source: Some("eldritch-lsp".to_string()),
                            message: format!(
                                "`{}` is shadowed by a variable or function in this scope; `{}.{}` cannot be called on the shadowed value.",
                                obj_name, obj_name, attr
                            ),
                            ..Default::default()
                        });
                    }
                    return;
                }

                // Check removed 0.4.0 functions first
                if let Some(&replacement) = mod_spec.removed_functions.get(attr) {
                    self.diagnostics.push(Diagnostic {
                        range: self.line_index.span_to_range(full_span),
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("removed-function".to_string())),
                        source: Some("eldritch-lsp".to_string()),
                        message: format!(
                            "`{}.{}` was removed in Eldritch 0.4.1. Use `{}` instead.",
                            obj_name, attr, replacement
                        ),
                        ..Default::default()
                    });
                    return;
                }

                // Check if function exists in module
                if let Some(fn_spec) = mod_spec.functions.get(attr) {
                    if let Some(repl) = fn_spec.deprecated_replacement {
                        self.diagnostics.push(Diagnostic {
                            range: self.line_index.span_to_range(full_span),
                            severity: Some(DiagnosticSeverity::WARNING),
                            code: Some(NumberOrString::String("deprecated-function".to_string())),
                            source: Some("eldritch-lsp".to_string()),
                            message: format!(
                                "`{}.{}` is deprecated in Eldritch 0.4.1. Use `{}` instead.",
                                obj_name, attr, repl
                            ),
                            tags: Some(vec![DiagnosticTag::DEPRECATED]),
                            ..Default::default()
                        });
                    } else if let Some(mod_repl) = mod_spec.deprecated_replacement {
                        self.diagnostics.push(Diagnostic {
                            range: self.line_index.span_to_range(full_span),
                            severity: Some(DiagnosticSeverity::WARNING),
                            code: Some(NumberOrString::String("deprecated-module".to_string())),
                            source: Some("eldritch-lsp".to_string()),
                            message: format!(
                                "Module `{}` is deprecated in Eldritch 0.4.1. {}",
                                obj_name, mod_repl
                            ),
                            tags: Some(vec![DiagnosticTag::DEPRECATED]),
                            ..Default::default()
                        });
                    }
                } else {
                    let candidates: Vec<&str> =
                        mod_spec.functions.keys().map(|s| s.as_str()).collect();
                    let suggestion = closest_match(attr, &candidates);
                    let msg = if let Some(sug) = suggestion {
                        format!(
                            "Module `{}` has no function `{}`. Did you mean `{}.{}`?",
                            obj_name, attr, obj_name, sug
                        )
                    } else {
                        format!("Module `{}` has no function `{}`.", obj_name, attr)
                    };
                    self.diagnostics.push(Diagnostic {
                        range: self.line_index.span_to_range(full_span),
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String(
                            "unknown-module-function".to_string(),
                        )),
                        source: Some("eldritch-lsp".to_string()),
                        message: msg,
                        ..Default::default()
                    });
                }
                return;
            }
        }

        self.analyze_expr(obj, scopes);
    }

    fn analyze_call(
        &mut self,
        callee: &Expr,
        args: &[Argument],
        call_span: Span,
        scopes: &mut Vec<Scope>,
    ) {
        self.analyze_expr(callee, scopes);

        // Analyze argument expressions and check for duplicate keyword arguments
        let mut seen_kwargs = BTreeSet::new();
        let mut has_star_unpack = false;
        let mut has_kw_unpack = false;
        let mut pos_count = 0usize;
        let mut kw_names: Vec<(String, Span)> = Vec::new();

        for arg in args {
            match arg {
                Argument::Positional(e) => {
                    pos_count += 1;
                    self.analyze_expr(e, scopes);
                }
                Argument::Keyword(k, e) => {
                    self.analyze_expr(e, scopes);
                    if !seen_kwargs.insert(k.clone()) {
                        self.diagnostics.push(Diagnostic {
                            range: self.line_index.span_to_range(e.span),
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(NumberOrString::String(
                                "duplicate-keyword-argument".to_string(),
                            )),
                            source: Some("eldritch-lsp".to_string()),
                            message: format!(
                                "Duplicate keyword argument `{}` in function call.",
                                k
                            ),
                            ..Default::default()
                        });
                    }
                    kw_names.push((k.clone(), e.span));
                }
                Argument::StarArgs(e) => {
                    has_star_unpack = true;
                    self.analyze_expr(e, scopes);
                }
                Argument::KwArgs(e) => {
                    has_kw_unpack = true;
                    self.analyze_expr(e, scopes);
                }
            }
        }

        // Check `input_params.get("key")`
        if let ExprKind::GetAttr(obj, method) = &callee.kind {
            if method == "get" {
                if let ExprKind::Identifier(obj_name) = &obj.kind {
                    if obj_name == "input_params" {
                        if let Some(Argument::Positional(first_arg)) = args.first() {
                            if let ExprKind::Literal(Value::String(param_key)) = &first_arg.kind {
                                self.check_input_param_key(param_key, first_arg.span);
                            }
                        }
                    }
                }
            }
        }

        if has_star_unpack || has_kw_unpack {
            return;
        }

        // Validate call arity & keyword argument names against known signatures
        match &callee.kind {
            ExprKind::GetAttr(obj, attr) => {
                if let ExprKind::Identifier(mod_name) = &obj.kind {
                    if !self.is_user_defined(mod_name, scopes) {
                        if let Some(fn_spec) = self.stdlib.get_module_func(mod_name, attr) {
                            self.validate_func_spec_call(fn_spec, pos_count, &kw_names, call_span);
                        }
                    }
                }
            }
            ExprKind::Identifier(fn_name) => {
                if let Some(user_fn) = self.lookup_user_func(fn_name, scopes) {
                    self.validate_user_func_call(&user_fn, pos_count, &kw_names, call_span);
                } else if !self.is_user_defined(fn_name, scopes) {
                    if let Some(builtin_spec) = self.stdlib.get_builtin(fn_name) {
                        self.validate_func_spec_call(builtin_spec, pos_count, &kw_names, call_span);
                    }
                }
            }
            _ => {}
        }
    }

    fn validate_func_spec_call(
        &mut self,
        spec: &FuncSpec,
        pos_count: usize,
        kw_names: &[(String, Span)],
        call_span: Span,
    ) {
        let total_args = pos_count + kw_names.len();
        let min_args = spec.min_args();
        let max_args = spec.max_args();

        // Check keyword argument names if function doesn't accept arbitrary **kwargs
        if !spec.kwargs && !spec.varargs {
            for (kw, kw_span) in kw_names {
                if let Some(param_idx) = spec.params.iter().position(|p| p.name == kw) {
                    if param_idx < pos_count {
                        self.diagnostics.push(Diagnostic {
                            range: self.line_index.span_to_range(*kw_span),
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(NumberOrString::String(
                                "multiple-argument-values".to_string(),
                            )),
                            source: Some("eldritch-lsp".to_string()),
                            message: format!(
                                "`{}()` got multiple values for argument `{}`.",
                                spec.qualified_name, kw
                            ),
                            ..Default::default()
                        });
                    }
                } else {
                    let valid_names: Vec<&str> = spec.params.iter().map(|p| p.name).collect();
                    let suggestion = closest_match(kw, &valid_names);
                    let msg = if let Some(sug) = suggestion {
                        format!(
                            "`{}()` got an unexpected keyword argument `{}`. Did you mean `{}`?",
                            spec.qualified_name, kw, sug
                        )
                    } else {
                        format!(
                            "`{}()` got an unexpected keyword argument `{}`.",
                            spec.qualified_name, kw
                        )
                    };
                    self.diagnostics.push(Diagnostic {
                        range: self.line_index.span_to_range(*kw_span),
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String(
                            "unknown-keyword-argument".to_string(),
                        )),
                        source: Some("eldritch-lsp".to_string()),
                        message: msg,
                        ..Default::default()
                    });
                }
            }
        }

        if total_args < min_args {
            let missing: Vec<&str> = spec
                .params
                .iter()
                .skip(pos_count)
                .filter(|p| p.required && !kw_names.iter().any(|(k, _)| k == p.name))
                .map(|p| p.name)
                .collect();
            let missing_desc = if missing.is_empty() {
                String::new()
            } else {
                format!(" (missing: `{}`)", missing.join("`, `"))
            };
            self.diagnostics.push(Diagnostic {
                range: self.line_index.span_to_range(call_span),
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("argument-count".to_string())),
                source: Some("eldritch-lsp".to_string()),
                message: format!(
                    "`{}()` requires at least {} argument(s), but {} provided{}.",
                    spec.qualified_name, min_args, total_args, missing_desc
                ),
                ..Default::default()
            });
        } else if let Some(max) = max_args {
            if !spec.kwargs && total_args > max {
                self.diagnostics.push(Diagnostic {
                    range: self.line_index.span_to_range(call_span),
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("argument-count".to_string())),
                    source: Some("eldritch-lsp".to_string()),
                    message: format!(
                        "`{}()` accepts at most {} argument(s), but {} provided.",
                        spec.qualified_name, max, total_args
                    ),
                    ..Default::default()
                });
            } else if pos_count > max {
                self.diagnostics.push(Diagnostic {
                    range: self.line_index.span_to_range(call_span),
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("argument-count".to_string())),
                    source: Some("eldritch-lsp".to_string()),
                    message: format!(
                        "`{}()` accepts at most {} positional argument(s), but {} provided.",
                        spec.qualified_name, max, pos_count
                    ),
                    ..Default::default()
                });
            }
        }
    }

    fn validate_user_func_call(
        &mut self,
        func: &UserFuncInfo,
        pos_count: usize,
        kw_names: &[(String, Span)],
        call_span: Span,
    ) {
        let total_args = pos_count + kw_names.len();
        let min_args = func.min_args();
        let max_args = func.max_args();

        if !func.accepts_kwargs() {
            let normal_params: Vec<&UserParam> = func
                .params
                .iter()
                .filter(|p| !p.is_star && !p.is_star_star)
                .collect();
            for (kw, kw_span) in kw_names {
                if let Some(idx) = normal_params.iter().position(|p| &p.name == kw) {
                    if idx < pos_count {
                        self.diagnostics.push(Diagnostic {
                            range: self.line_index.span_to_range(*kw_span),
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(NumberOrString::String(
                                "multiple-argument-values".to_string(),
                            )),
                            source: Some("eldritch-lsp".to_string()),
                            message: format!(
                                "`{}()` got multiple values for argument `{}`.",
                                func.name, kw
                            ),
                            ..Default::default()
                        });
                    }
                } else {
                    self.diagnostics.push(Diagnostic {
                        range: self.line_index.span_to_range(*kw_span),
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String(
                            "unknown-keyword-argument".to_string(),
                        )),
                        source: Some("eldritch-lsp".to_string()),
                        message: format!(
                            "`{}()` got an unexpected keyword argument `{}`.",
                            func.name, kw
                        ),
                        ..Default::default()
                    });
                }
            }
        }

        if total_args < min_args {
            self.diagnostics.push(Diagnostic {
                range: self.line_index.span_to_range(call_span),
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("argument-count".to_string())),
                source: Some("eldritch-lsp".to_string()),
                message: format!(
                    "`{}()` requires at least {} argument(s), but {} provided.",
                    func.name, min_args, total_args
                ),
                ..Default::default()
            });
        } else if let Some(max) = max_args {
            if !func.accepts_kwargs() && total_args > max {
                self.diagnostics.push(Diagnostic {
                    range: self.line_index.span_to_range(call_span),
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("argument-count".to_string())),
                    source: Some("eldritch-lsp".to_string()),
                    message: format!(
                        "`{}()` accepts at most {} argument(s), but {} provided.",
                        func.name, max, total_args
                    ),
                    ..Default::default()
                });
            }
        }
    }

    fn check_input_param_key(&mut self, key: &str, span: Span) {
        self.referenced_input_params.insert(key.to_string());
        if let Some(meta) = self.metadata {
            if meta.get_param(key).is_none() {
                let available: Vec<&str> = meta.paramdefs.iter().map(|p| p.name.as_str()).collect();
                let avail_str = if available.is_empty() {
                    "none declared".to_string()
                } else {
                    format!("declared: `{}`", available.join("`, `"))
                };
                self.diagnostics.push(Diagnostic {
                    range: self.line_index.span_to_range(span),
                    severity: Some(DiagnosticSeverity::WARNING),
                    code: Some(NumberOrString::String("unknown-input-param".to_string())),
                    source: Some("eldritch-lsp".to_string()),
                    message: format!(
                        "`input_params['{}']` is not declared in `metadata.yml` ({}).",
                        key, avail_str
                    ),
                    ..Default::default()
                });
            }
        }
    }

    fn is_user_defined(&self, name: &str, scopes: &[Scope]) -> bool {
        scopes
            .iter()
            .rev()
            .any(|s| s.vars.contains_key(name) || s.funcs.contains_key(name))
    }

    fn lookup_user_func(&self, name: &str, scopes: &[Scope]) -> Option<UserFuncInfo> {
        for s in scopes.iter().rev() {
            if let Some(f) = s.funcs.get(name) {
                return Some(f.clone());
            }
        }
        None
    }

    fn is_defined(&self, name: &str, scopes: &[Scope]) -> bool {
        if name == "input_params" {
            return true;
        }
        if self.is_user_defined(name, scopes) {
            return true;
        }
        if self.stdlib.is_builtin(name) || self.stdlib.is_module(name) {
            return true;
        }
        false
    }

    fn suggest_identifier(&self, name: &str, scopes: &[Scope]) -> Option<String> {
        let mut candidates: Vec<&str> = vec!["input_params"];
        for s in scopes {
            for k in s.vars.keys() {
                candidates.push(k.as_str());
            }
            for k in s.funcs.keys() {
                candidates.push(k.as_str());
            }
        }
        for k in self.stdlib.builtins.keys() {
            candidates.push(k.as_str());
        }
        for k in self.stdlib.modules.keys() {
            candidates.push(k.as_str());
        }
        closest_match(name, &candidates).map(|s| s.to_string())
    }

    fn infer_expr_type(&self, expr: &Expr, scopes: &[Scope]) -> Option<String> {
        match &expr.kind {
            ExprKind::Literal(val) => match val {
                Value::Int(_) => Some("int".to_string()),
                Value::Float(_) => Some("float".to_string()),
                Value::Bool(_) => Some("bool".to_string()),
                Value::String(_) => Some("str".to_string()),
                Value::Bytes(_) => Some("Bytes".to_string()),
                Value::List(_) => Some("List".to_string()),
                Value::Dictionary(_) => Some("Dict".to_string()),
                Value::Tuple(_) => Some("Tuple".to_string()),
                Value::Set(_) => Some("Set".to_string()),
                Value::None => Some("None".to_string()),
                _ => None,
            },
            ExprKind::FString(_) => Some("str".to_string()),
            ExprKind::List(_) | ExprKind::ListComp { .. } => Some("List".to_string()),
            ExprKind::Dictionary(_) | ExprKind::DictComp { .. } => Some("Dict".to_string()),
            ExprKind::Tuple(_) => Some("Tuple".to_string()),
            ExprKind::Set(_) | ExprKind::SetComp { .. } => Some("Set".to_string()),
            ExprKind::Identifier(name) => {
                if name == "input_params" {
                    return Some("Dict[str, Any]".to_string());
                }
                for s in scopes.iter().rev() {
                    if let Some(v) = s.vars.get(name) {
                        if v.inferred_type.is_some() {
                            return v.inferred_type.clone();
                        }
                    }
                }
                None
            }
            ExprKind::Call(callee, _) => match &callee.kind {
                ExprKind::GetAttr(obj, attr) => {
                    if let ExprKind::Identifier(mod_name) = &obj.kind {
                        if let Some(spec) = self.stdlib.get_module_func(mod_name, attr) {
                            return Some(spec.return_type.to_string());
                        }
                    }
                    if let Some(methods) = self.stdlib.methods.get(attr) {
                        if let Some(m) = methods.first() {
                            return Some(m.return_type.to_string());
                        }
                    }
                    None
                }
                ExprKind::Identifier(fn_name) => {
                    if let Some(uf) = self.lookup_user_func(fn_name, scopes) {
                        return uf.return_type;
                    }
                    if let Some(spec) = self.stdlib.get_builtin(fn_name) {
                        return Some(spec.return_type.to_string());
                    }
                    None
                }
                _ => None,
            },
            _ => None,
        }
    }
}

fn format_type_expr(expr: &Expr) -> String {
    match &expr.kind {
        ExprKind::Identifier(name) => name.clone(),
        ExprKind::Index(base, sub) => {
            format!("{}[{}]", format_type_expr(base), format_type_expr(sub))
        }
        ExprKind::Tuple(items) => {
            let parts: Vec<String> = items.iter().map(format_type_expr).collect();
            parts.join(", ")
        }
        ExprKind::Literal(Value::None) => "None".to_string(),
        _ => "Any".to_string(),
    }
}

fn convert_param(p: &Param) -> UserParam {
    match p {
        Param::Normal(name, ty) => UserParam {
            name: name.clone(),
            ty: ty.as_deref().map(format_type_expr),
            default_repr: None,
            is_star: false,
            is_star_star: false,
        },
        Param::Star(name, ty) => UserParam {
            name: name.clone(),
            ty: ty.as_deref().map(format_type_expr),
            default_repr: None,
            is_star: true,
            is_star_star: false,
        },
        Param::StarStar(name, ty) => UserParam {
            name: name.clone(),
            ty: ty.as_deref().map(format_type_expr),
            default_repr: None,
            is_star: false,
            is_star_star: true,
        },
        Param::WithDefault(name, ty, expr) => UserParam {
            name: name.clone(),
            ty: ty.as_deref().map(format_type_expr),
            default_repr: Some(expr_brief_repr(expr)),
            is_star: false,
            is_star_star: false,
        },
    }
}

fn expr_brief_repr(expr: &Expr) -> String {
    match &expr.kind {
        ExprKind::Literal(Value::None) => "None".to_string(),
        ExprKind::Literal(Value::Bool(true)) => "True".to_string(),
        ExprKind::Literal(Value::Bool(false)) => "False".to_string(),
        ExprKind::Literal(Value::Int(i)) => i.to_string(),
        ExprKind::Literal(Value::Float(f)) => f.to_string(),
        ExprKind::Literal(Value::String(s)) => format!("{:?}", s),
        ExprKind::List(items) if items.is_empty() => "[]".to_string(),
        ExprKind::Dictionary(items) if items.is_empty() => "{}".to_string(),
        ExprKind::Tuple(items) if items.is_empty() => "()".to_string(),
        _ => "...".to_string(),
    }
}

fn extract_docstring(body: &[Stmt], line_index: &LineIndex, def_span: Span) -> Option<String> {
    if let Some(first) = body.first() {
        if let StmtKind::Expression(expr) = &first.kind {
            if let ExprKind::Literal(Value::String(s)) = &expr.kind {
                return Some(s.trim().to_string());
            }
        }
    }
    // Check leading `#` comments immediately above `def`
    let def_line = line_index.char_offset_to_position(def_span.start).line;
    if def_line > 0 {
        let mut comment_lines = Vec::new();
        let mut line = def_line;
        while line > 0 {
            line -= 1;
            let text = line_index.line_text(line);
            let trimmed = text.trim();
            if let Some(stripped) = trimmed.strip_prefix('#') {
                comment_lines.push(stripped.trim().to_string());
            } else {
                break;
            }
        }
        if !comment_lines.is_empty() {
            comment_lines.reverse();
            return Some(comment_lines.join("\n"));
        }
    }
    None
}

fn is_terminating_stmt(stmt: &Stmt) -> bool {
    match &stmt.kind {
        StmtKind::Return(_) | StmtKind::Break | StmtKind::Continue => true,
        StmtKind::Expression(expr) => {
            if let ExprKind::Call(callee, _) = &expr.kind {
                if let ExprKind::Identifier(name) = &callee.kind {
                    if name == "fail" {
                        return true;
                    }
                }
            }
            false
        }
        _ => false,
    }
}

fn literal_key_repr(expr: &Expr) -> Option<String> {
    match &expr.kind {
        ExprKind::Literal(Value::String(s)) => Some(format!("{:?}", s)),
        ExprKind::Literal(Value::Int(i)) => Some(i.to_string()),
        ExprKind::Literal(Value::Bool(b)) => Some(if *b { "True" } else { "False" }.to_string()),
        _ => None,
    }
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b_chars.len()).collect();
    let mut curr = vec![0; b_chars.len() + 1];

    for (i, &ca) in a_chars.iter().enumerate() {
        curr[0] = i + 1;
        for (j, &cb) in b_chars.iter().enumerate() {
            let cost = usize::from(ca != cb);
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b_chars.len()]
}

fn closest_match<'a>(needle: &str, candidates: &[&'a str]) -> Option<&'a str> {
    if needle.len() < 2 {
        return None;
    }
    let max_dist = if needle.len() <= 4 { 1 } else { 2 };
    let mut best: Option<(&'a str, usize)> = None;
    for &cand in candidates {
        let dist = edit_distance(needle, cand);
        if dist > 0 && dist <= max_dist {
            if best.map(|(_, d)| dist < d).unwrap_or(true) {
                best = Some((cand, dist));
            }
        }
    }
    best.map(|(c, _)| c)
}

// ============================================================================
// Completion, Hover, Signature Help, Document Symbols, Go-to-Definition
// ============================================================================

pub fn complete_at(
    analysis: &AnalysisResult,
    pos: Position,
    stdlib: &StdlibIndex,
) -> Vec<CompletionItem> {
    let line = analysis.line_index.line_text(pos.line);
    let col = (pos.character as usize).min(line.len());
    let before_cursor = &line[..col];

    // 1. Check if inside `input_params['...` or `input_params["...` or `input_params.get('...`
    if is_input_params_key_context(before_cursor) {
        return complete_input_params(analysis);
    }

    // 2. Check if member completion `<receiver>.<prefix>`
    if let Some((receiver, _prefix)) = extract_member_access(before_cursor) {
        if let Some(mod_spec) = stdlib.get_module(receiver) {
            if !analysis.variables.contains_key(receiver)
                && !analysis.functions.contains_key(receiver)
            {
                return complete_module_members(mod_spec);
            }
        }
        // Receiver is a variable or expression: offer built-in methods
        let inferred_ty = analysis
            .variables
            .get(receiver)
            .and_then(|v| v.inferred_type.as_deref());
        return complete_value_methods(stdlib, inferred_ty);
    }

    let mut items = Vec::new();

    // 3. If inside a function call `foo(...)`, include keyword argument completions `param=`
    if let Some((call_target, _arg_idx)) =
        find_enclosing_call(&analysis.source, pos, &analysis.line_index)
    {
        if let Some((mod_name, fn_name)) = call_target.split_once('.') {
            if let Some(spec) = stdlib.get_module_func(mod_name, fn_name) {
                for (i, p) in spec.params.iter().enumerate() {
                    if !p.name.starts_with('*') {
                        items.push(CompletionItem {
                            label: format!("{}=", p.name),
                            kind: Some(CompletionItemKind::PROPERTY),
                            detail: Some(format!(
                                "parameter of `{}` ({})",
                                spec.qualified_name,
                                p.format_label()
                            )),
                            insert_text: Some(format!("{}=", p.name)),
                            sort_text: Some(format!("00_{:02}_{}", i, p.name)),
                            ..Default::default()
                        });
                    }
                }
            }
        } else if let Some(user_fn) = analysis.functions.get(&call_target) {
            for (i, p) in user_fn.params.iter().enumerate() {
                if !p.is_star && !p.is_star_star {
                    items.push(CompletionItem {
                        label: format!("{}=", p.name),
                        kind: Some(CompletionItemKind::PROPERTY),
                        detail: Some(format!(
                            "parameter of `{}` ({})",
                            user_fn.name,
                            p.format_label()
                        )),
                        insert_text: Some(format!("{}=", p.name)),
                        sort_text: Some(format!("00_{:02}_{}", i, p.name)),
                        ..Default::default()
                    });
                }
            }
        } else if let Some(builtin) = stdlib.get_builtin(&call_target) {
            for (i, p) in builtin.params.iter().enumerate() {
                if !p.name.starts_with('*') {
                    items.push(CompletionItem {
                        label: format!("{}=", p.name),
                        kind: Some(CompletionItemKind::PROPERTY),
                        detail: Some(format!(
                            "parameter of `{}` ({})",
                            builtin.name,
                            p.format_label()
                        )),
                        insert_text: Some(format!("{}=", p.name)),
                        sort_text: Some(format!("00_{:02}_{}", i, p.name)),
                        ..Default::default()
                    });
                }
            }
        }
    }

    // 4. User-defined variables & functions in document
    for var in analysis.variables.values() {
        items.push(CompletionItem {
            label: var.name.clone(),
            kind: Some(CompletionItemKind::VARIABLE),
            detail: Some(
                var.inferred_type
                    .clone()
                    .unwrap_or_else(|| "variable".to_string()),
            ),
            sort_text: Some(format!("01_{}", var.name)),
            ..Default::default()
        });
    }

    for func in analysis.functions.values() {
        items.push(CompletionItem {
            label: func.name.clone(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(func.signature_string()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: func.markdown_docs(),
            })),
            insert_text: Some(func.snippet()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            sort_text: Some(format!("01_{}", func.name)),
            ..Default::default()
        });
    }

    // 5. `input_params` global dictionary
    let mut input_params_doc =
        "Pre-populated global dictionary of Tome input parameters (`Dict[str, Any]`).".to_string();
    if let Some(meta) = &analysis.metadata {
        if !meta.paramdefs.is_empty() {
            input_params_doc.push_str("\n\n**Declared parameters in `metadata.yml`:**\n");
            for p in &meta.paramdefs {
                input_params_doc.push_str(&format!(
                    "- `input_params[\"{}\"]` (`{}`): {}\n",
                    p.name, p.param_type, p.label
                ));
            }
        }
    }
    items.push(CompletionItem {
        label: "input_params".to_string(),
        kind: Some(CompletionItemKind::VARIABLE),
        detail: Some("Dict[str, Any] (Tome input parameters)".to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: input_params_doc,
        })),
        sort_text: Some("01_input_params".to_string()),
        ..Default::default()
    });

    // 6. Standard library modules
    for mod_spec in stdlib.modules.values() {
        let is_depr = mod_spec.deprecated_replacement.is_some();
        items.push(CompletionItem {
            label: mod_spec.name.to_string(),
            kind: Some(CompletionItemKind::MODULE),
            detail: Some(if is_depr {
                format!("Eldritch stdlib module (deprecated)")
            } else {
                format!("Eldritch stdlib module `{}`", mod_spec.name)
            }),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: mod_spec.markdown_docs(),
            })),
            tags: if is_depr {
                Some(vec![CompletionItemTag::DEPRECATED])
            } else {
                None
            },
            sort_text: Some(format!(
                "{}_{}",
                if is_depr { "09" } else { "02" },
                mod_spec.name
            )),
            ..Default::default()
        });
    }

    // 7. Built-in functions
    for builtin in stdlib.builtins.values() {
        items.push(CompletionItem {
            label: builtin.name.to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(builtin.signature_string()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: builtin.markdown_docs(),
            })),
            insert_text: Some(builtin.snippet()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            sort_text: Some(format!("03_{}", builtin.name)),
            ..Default::default()
        });
    }

    // 8. Eldritch keywords and constants
    let keywords = [
        ("def", CompletionItemKind::KEYWORD, "Define a function"),
        ("if", CompletionItemKind::KEYWORD, "Conditional branch"),
        (
            "elif",
            CompletionItemKind::KEYWORD,
            "Else-if conditional branch",
        ),
        ("else", CompletionItemKind::KEYWORD, "Else branch"),
        (
            "for",
            CompletionItemKind::KEYWORD,
            "For loop over an iterable",
        ),
        (
            "in",
            CompletionItemKind::KEYWORD,
            "Membership / iteration operator",
        ),
        (
            "return",
            CompletionItemKind::KEYWORD,
            "Return from a function",
        ),
        ("break", CompletionItemKind::KEYWORD, "Break out of a loop"),
        (
            "continue",
            CompletionItemKind::KEYWORD,
            "Continue to next loop iteration",
        ),
        ("pass", CompletionItemKind::KEYWORD, "No-op statement"),
        ("and", CompletionItemKind::KEYWORD, "Logical AND operator"),
        ("or", CompletionItemKind::KEYWORD, "Logical OR operator"),
        ("not", CompletionItemKind::KEYWORD, "Logical NOT operator"),
        ("True", CompletionItemKind::CONSTANT, "Boolean True literal"),
        (
            "False",
            CompletionItemKind::CONSTANT,
            "Boolean False literal",
        ),
        ("None", CompletionItemKind::CONSTANT, "None literal"),
    ];
    for (kw, kind, desc) in keywords {
        items.push(CompletionItem {
            label: kw.to_string(),
            kind: Some(kind),
            detail: Some(desc.to_string()),
            sort_text: Some(format!("04_{}", kw)),
            ..Default::default()
        });
    }

    items
}

fn is_input_params_key_context(before_cursor: &str) -> bool {
    let trimmed = before_cursor.trim_end();
    for pattern in [
        "input_params['",
        "input_params[\"",
        "input_params.get('",
        "input_params.get(\"",
    ] {
        if let Some(idx) = trimmed.rfind(pattern) {
            let after = &trimmed[idx + pattern.len()..];
            if !after.contains('\'') && !after.contains('"') && !after.contains(']') {
                return true;
            }
        }
    }
    false
}

fn complete_input_params(analysis: &AnalysisResult) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    let mut seen = BTreeSet::new();

    if let Some(meta) = &analysis.metadata {
        for p in &meta.paramdefs {
            seen.insert(p.name.clone());
            let mut doc = format!("**Type:** `{}`", p.param_type);
            if !p.label.is_empty() {
                doc.push_str(&format!("\n\n**Label:** {}", p.label));
            }
            if !p.placeholder.is_empty() {
                doc.push_str(&format!("\n\n**Placeholder:** `{}`", p.placeholder));
            }
            items.push(CompletionItem {
                label: p.name.clone(),
                kind: Some(CompletionItemKind::FIELD),
                detail: Some(format!("input_param ({}) - {}", p.param_type, p.label)),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: doc,
                })),
                sort_text: Some(format!("00_{}", p.name)),
                ..Default::default()
            });
        }
    }

    for key in &analysis.referenced_input_params {
        if seen.insert(key.clone()) {
            items.push(CompletionItem {
                label: key.clone(),
                kind: Some(CompletionItemKind::FIELD),
                detail: Some("input_params key".to_string()),
                sort_text: Some(format!("01_{}", key)),
                ..Default::default()
            });
        }
    }

    items
}

fn extract_member_access(before_cursor: &str) -> Option<(&str, &str)> {
    let dot_idx = before_cursor.rfind('.')?;
    let prefix = &before_cursor[dot_idx + 1..];
    if !prefix
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return None;
    }
    let before_dot = &before_cursor[..dot_idx];
    let recv_start = before_dot
        .rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .map(|i| i + 1)
        .unwrap_or(0);
    let receiver = &before_dot[recv_start..];
    if receiver.is_empty() {
        return None;
    }
    Some((receiver, prefix))
}

fn complete_module_members(mod_spec: &crate::stdlib::ModuleSpec) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    for func in mod_spec.functions.values() {
        let is_depr = func.deprecated_replacement.is_some();
        let is_internal = func.name.starts_with('_');
        items.push(CompletionItem {
            label: func.name.to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(func.signature_string()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: func.markdown_docs(),
            })),
            insert_text: Some(func.snippet()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            tags: if is_depr {
                Some(vec![CompletionItemTag::DEPRECATED])
            } else {
                None
            },
            sort_text: Some(format!(
                "{}_{}",
                if is_depr {
                    "09"
                } else if is_internal {
                    "08"
                } else {
                    "00"
                },
                func.name
            )),
            ..Default::default()
        });
    }
    items
}

fn complete_value_methods(stdlib: &StdlibIndex, inferred_ty: Option<&str>) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    let normalized_ty = inferred_ty.map(|t| {
        if t.starts_with("Dict") {
            "Dict"
        } else if t.starts_with("List") {
            "List"
        } else if t.starts_with("Set") {
            "Set"
        } else if t.starts_with("Tuple") {
            "Tuple"
        } else {
            t
        }
    });

    for method_list in stdlib.methods.values() {
        for m in method_list {
            let matches_type = normalized_ty
                .map(|t| t.eq_ignore_ascii_case(m.receiver_type))
                .unwrap_or(true);
            if normalized_ty.is_some() && !matches_type {
                continue;
            }
            let req_params: Vec<&ParamSpec> = m
                .params
                .iter()
                .filter(|p| p.required && !p.name.starts_with('*'))
                .collect();
            let snippet = if req_params.is_empty() {
                format!("{}()$0", m.name)
            } else {
                let ph: Vec<String> = req_params
                    .iter()
                    .enumerate()
                    .map(|(i, p)| format!("${{{}:{}}}", i + 1, p.name))
                    .collect();
                format!("{}({})$0", m.name, ph.join(", "))
            };
            items.push(CompletionItem {
                label: m.name.to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some(m.signature_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: m.markdown_docs(),
                })),
                insert_text: Some(snippet),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                sort_text: Some(format!("01_{}_{}", m.name, m.receiver_type)),
                ..Default::default()
            });
        }
    }
    items
}

pub fn hover_at(analysis: &AnalysisResult, pos: Position, stdlib: &StdlibIndex) -> Option<Hover> {
    let line = analysis.line_index.line_text(pos.line);
    let (word, receiver, range) = word_and_receiver_at(&line, pos)?;

    // 1. Check `module.func`
    if let Some(recv) = receiver {
        if let Some(mod_spec) = stdlib.get_module(&recv) {
            if !analysis.variables.contains_key(&recv) {
                if let Some(fn_spec) = mod_spec.functions.get(&word) {
                    return Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: fn_spec.markdown_docs(),
                        }),
                        range: Some(range),
                    });
                }
            }
        }
        if let Some(methods) = stdlib.methods.get(&word) {
            let docs: Vec<String> = methods.iter().map(MethodSpec::markdown_docs).collect();
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: docs.join("\n\n---\n\n"),
                }),
                range: Some(range),
            });
        }
    }

    // 2. Check user-defined function
    if let Some(user_fn) = analysis.functions.get(&word) {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: user_fn.markdown_docs(),
            }),
            range: Some(range),
        });
    }

    // 3. Check `input_params`
    if word == "input_params" {
        let mut md =
            "```python\ninput_params: Dict[str, Any]\n```\n\nGlobal dictionary containing parameters passed to the Tome at execution time.".to_string();
        if let Some(meta) = &analysis.metadata {
            if !meta.paramdefs.is_empty() {
                md.push_str("\n\n**Parameters defined in `metadata.yml`:**\n");
                for p in &meta.paramdefs {
                    md.push_str(&format!(
                        "- `{}` (`{}`): {}\n",
                        p.name, p.param_type, p.label
                    ));
                }
            }
        }
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: md,
            }),
            range: Some(range),
        });
    }

    // 4. Check stdlib module
    if let Some(mod_spec) = stdlib.get_module(&word) {
        if !analysis.variables.contains_key(&word) {
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: mod_spec.markdown_docs(),
                }),
                range: Some(range),
            });
        }
    }

    // 5. Check builtin function
    if let Some(builtin) = stdlib.get_builtin(&word) {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: builtin.markdown_docs(),
            }),
            range: Some(range),
        });
    }

    // 6. Check user-defined variable
    if let Some(var) = analysis.variables.get(&word) {
        let ty = var.inferred_type.as_deref().unwrap_or("Any");
        let kind = if var.is_param {
            "parameter"
        } else {
            "variable"
        };
        let md = format!("```python\n({kind}) {}: {}\n```", var.name, ty);
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: md,
            }),
            range: Some(range),
        });
    }

    None
}

fn word_and_receiver_at(line: &str, pos: Position) -> Option<(String, Option<String>, Range)> {
    let chars: Vec<char> = line.chars().collect();
    if chars.is_empty() {
        return None;
    }
    let mut idx = (pos.character as usize).min(chars.len());
    if idx == chars.len() || !(chars[idx].is_ascii_alphanumeric() || chars[idx] == '_') {
        if idx > 0 && (chars[idx - 1].is_ascii_alphanumeric() || chars[idx - 1] == '_') {
            idx -= 1;
        } else {
            return None;
        }
    }

    let mut start = idx;
    while start > 0 && (chars[start - 1].is_ascii_alphanumeric() || chars[start - 1] == '_') {
        start -= 1;
    }
    let mut end = idx + 1;
    while end < chars.len() && (chars[end].is_ascii_alphanumeric() || chars[end] == '_') {
        end += 1;
    }

    let word: String = chars[start..end].iter().collect();
    let receiver = if start > 1 && chars[start - 1] == '.' {
        let mut r_start = start - 1;
        while r_start > 0
            && (chars[r_start - 1].is_ascii_alphanumeric() || chars[r_start - 1] == '_')
        {
            r_start -= 1;
        }
        if r_start < start - 1 {
            Some(chars[r_start..start - 1].iter().collect())
        } else {
            None
        }
    } else {
        None
    };

    let range = Range {
        start: Position {
            line: pos.line,
            character: start as u32,
        },
        end: Position {
            line: pos.line,
            character: end as u32,
        },
    };
    Some((word, receiver, range))
}

pub fn signature_help_at(
    analysis: &AnalysisResult,
    pos: Position,
    stdlib: &StdlibIndex,
) -> Option<SignatureHelp> {
    let (callee, active_param) = find_enclosing_call(&analysis.source, pos, &analysis.line_index)?;

    if let Some((mod_name, fn_name)) = callee.split_once('.') {
        if let Some(spec) = stdlib.get_module_func(mod_name, fn_name) {
            return Some(build_func_spec_signature_help(spec, active_param));
        }
    }

    if let Some(user_fn) = analysis.functions.get(&callee) {
        let params_info: Vec<ParameterInformation> = user_fn
            .params
            .iter()
            .map(|p| ParameterInformation {
                label: ParameterLabel::Simple(p.format_label()),
                documentation: None,
            })
            .collect();
        return Some(SignatureHelp {
            signatures: vec![SignatureInformation {
                label: user_fn.signature_string(),
                documentation: user_fn.docstring.as_ref().map(|d| {
                    Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: d.clone(),
                    })
                }),
                parameters: Some(params_info),
                active_parameter: Some(active_param as u32),
            }],
            active_signature: Some(0),
            active_parameter: Some(active_param as u32),
        });
    }

    if let Some(builtin) = stdlib.get_builtin(&callee) {
        return Some(build_func_spec_signature_help(builtin, active_param));
    }

    None
}

fn build_func_spec_signature_help(spec: &FuncSpec, active_param: usize) -> SignatureHelp {
    let params_info: Vec<ParameterInformation> = spec
        .params
        .iter()
        .map(|p| ParameterInformation {
            label: ParameterLabel::Simple(p.format_label()),
            documentation: None,
        })
        .collect();
    let clamped_active = if spec.params.is_empty() {
        0
    } else {
        active_param.min(spec.params.len() - 1) as u32
    };
    SignatureHelp {
        signatures: vec![SignatureInformation {
            label: spec.signature_string(),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: spec.markdown_docs(),
            })),
            parameters: Some(params_info),
            active_parameter: Some(clamped_active),
        }],
        active_signature: Some(0),
        active_parameter: Some(clamped_active),
    }
}

fn find_enclosing_call(
    source: &str,
    pos: Position,
    line_index: &LineIndex,
) -> Option<(String, usize)> {
    let cursor_offset = line_index.position_to_char_offset(pos);
    let chars = &line_index.chars[..cursor_offset.min(line_index.chars.len())];
    let _ = source;

    let mut depth = 0i32;
    let mut commas_at_zero = 0usize;
    let mut open_paren_idx = None;

    let mut i = chars.len();
    while i > 0 {
        i -= 1;
        match chars[i] {
            ')' | ']' | '}' => depth += 1,
            '(' => {
                if depth == 0 {
                    open_paren_idx = Some(i);
                    break;
                }
                depth -= 1;
            }
            '[' | '{' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            ',' if depth == 0 => {
                commas_at_zero += 1;
            }
            _ => {}
        }
    }

    let paren_idx = open_paren_idx?;
    let mut end = paren_idx;
    while end > 0 && chars[end - 1].is_whitespace() {
        end -= 1;
    }
    let mut start = end;
    while start > 0
        && (chars[start - 1].is_ascii_alphanumeric()
            || chars[start - 1] == '_'
            || chars[start - 1] == '.')
    {
        start -= 1;
    }
    if start == end {
        return None;
    }
    let callee: String = chars[start..end].iter().collect();
    Some((callee, commas_at_zero))
}

#[allow(deprecated)]
pub fn document_symbols(analysis: &AnalysisResult) -> Vec<DocumentSymbol> {
    let mut symbols = Vec::new();
    for stmt in &analysis.ast {
        match &stmt.kind {
            StmtKind::Def(name, params, ret, _body) => {
                let range = analysis.line_index.span_to_range(stmt.span);
                let sel_range = analysis.line_index.first_line_range(stmt.span);
                let p_labels: Vec<String> = params
                    .iter()
                    .map(|p| convert_param(p).format_label())
                    .collect();
                let detail = format!(
                    "({}) -> {}",
                    p_labels.join(", "),
                    ret.as_deref()
                        .map(format_type_expr)
                        .as_deref()
                        .unwrap_or("Any")
                );
                symbols.push(DocumentSymbol {
                    name: name.clone(),
                    detail: Some(detail),
                    kind: SymbolKind::FUNCTION,
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range: sel_range,
                    children: None,
                });
            }
            StmtKind::Assignment(target, ann, _val) => {
                if let ExprKind::Identifier(name) = &target.kind {
                    let range = analysis.line_index.span_to_range(stmt.span);
                    let sel_range = analysis.line_index.span_to_range(target.span);
                    symbols.push(DocumentSymbol {
                        name: name.clone(),
                        detail: ann.as_deref().map(format_type_expr),
                        kind: SymbolKind::VARIABLE,
                        tags: None,
                        deprecated: None,
                        range,
                        selection_range: sel_range,
                        children: None,
                    });
                }
            }
            _ => {}
        }
    }
    symbols
}

pub fn goto_definition(
    analysis: &AnalysisResult,
    pos: Position,
    uri: &Url,
) -> Option<GotoDefinitionResponse> {
    let line = analysis.line_index.line_text(pos.line);
    let (word, receiver, _) = word_and_receiver_at(&line, pos)?;
    if receiver.is_some() {
        return None;
    }
    if let Some(func) = analysis.functions.get(&word) {
        return Some(GotoDefinitionResponse::Scalar(Location {
            uri: uri.clone(),
            range: analysis.line_index.first_line_range(func.def_span),
        }));
    }
    if let Some(var) = analysis.variables.get(&word) {
        return Some(GotoDefinitionResponse::Scalar(Location {
            uri: uri.clone(),
            range: analysis.line_index.first_line_range(var.def_span),
        }));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn analyze(code: &str) -> AnalysisResult {
        let stdlib = StdlibIndex::new();
        let linter = LintRegistry::default();
        analyze_document(code, &stdlib, &linter, None)
    }

    #[test]
    fn test_all_tavern_tomes_have_zero_errors() {
        let stdlib = StdlibIndex::new();
        let linter = LintRegistry::default();
        let tomes_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../tavern/tomes");
        assert!(tomes_dir.is_dir(), "tavern/tomes dir should exist");

        let mut checked = 0;
        for entry in std::fs::read_dir(&tomes_dir).unwrap() {
            let entry = entry.unwrap();
            let eldritch_file = entry.path().join("main.eldritch");
            if eldritch_file.is_file() {
                let source = std::fs::read_to_string(&eldritch_file).unwrap();
                let meta = TomeMetadata::load_for_document(&eldritch_file);
                let res = analyze_document(&source, &stdlib, &linter, meta);
                let errors: Vec<_> = res
                    .diagnostics
                    .iter()
                    .filter(|d| d.severity == Some(DiagnosticSeverity::ERROR))
                    .collect();
                assert!(
                    errors.is_empty(),
                    "Expected 0 errors in {:?}, got: {:?}",
                    eldritch_file,
                    errors
                );
                checked += 1;
            }
        }
        assert!(
            checked >= 30,
            "Expected at least 30 tomes, checked {}",
            checked
        );
    }

    #[test]
    fn test_lexer_and_syntax_errors() {
        let res = analyze("x = \"unterminated\n");
        assert!(res.diagnostics.iter().any(|d| {
            d.code == Some(NumberOrString::String("lexer-error".to_string()))
                && d.message.contains("Unterminated string")
        }));

        let res2 = analyze("def broken(\n");
        assert!(
            res2.diagnostics
                .iter()
                .any(|d| { d.code == Some(NumberOrString::String("syntax-error".to_string())) })
        );
    }

    #[test]
    fn test_undefined_variable_and_suggestions() {
        let res = analyze("prnt(\"hello\")\n");
        assert_eq!(res.diagnostics.len(), 1);
        let diag = &res.diagnostics[0];
        assert_eq!(
            diag.code,
            Some(NumberOrString::String("undefined-variable".to_string()))
        );
        assert!(diag.message.contains("Did you mean `print`?"));
    }

    #[test]
    fn test_forward_function_hoisting() {
        let code = r#"
def caller():
    return helper(42)

def helper(x: int) -> int:
    return x + 1

print(caller())
"#;
        let res = analyze(code);
        assert!(
            res.diagnostics.is_empty(),
            "Expected no diagnostics for hoisted function call, got: {:?}",
            res.diagnostics
        );
    }

    #[test]
    fn test_stdlib_unknown_deprecated_and_removed_functions() {
        let code = r#"
a = file.exist("/tmp")
b = file.read_binary("/tmp/a")
c = sys.get_os()
d = crypto.md5("abc")
"#;
        let res = analyze(code);
        assert!(res.diagnostics.iter().any(|d| {
            d.code
                == Some(NumberOrString::String(
                    "unknown-module-function".to_string(),
                ))
                && d.message.contains("Did you mean `file.exists`?")
        }));
        assert!(res.diagnostics.iter().any(|d| {
            d.code == Some(NumberOrString::String("removed-function".to_string()))
                && d.message.contains("file.read(path)")
        }));
        assert!(res.diagnostics.iter().any(|d| {
            d.code == Some(NumberOrString::String("deprecated-function".to_string()))
                && d.message.contains("sys.info()")
        }));
        assert!(res.diagnostics.iter().any(|d| {
            d.code == Some(NumberOrString::String("deprecated-function".to_string()))
                && d.message.contains("hash.md5(data)")
        }));
    }

    #[test]
    fn test_call_argument_validation() {
        let code = r#"
file.write("/tmp/test")
file.exists("/tmp/a", "/tmp/b")
sys.exec("/bin/ls", ["-la"], bogus_flag=True)
"#;
        let res = analyze(code);
        assert_eq!(
            res.diagnostics
                .iter()
                .filter(|d| d.code == Some(NumberOrString::String("argument-count".to_string())))
                .count(),
            2
        );
        assert!(res.diagnostics.iter().any(|d| {
            d.code
                == Some(NumberOrString::String(
                    "unknown-keyword-argument".to_string(),
                ))
                && d.message.contains("bogus_flag")
        }));
    }

    #[test]
    fn test_control_flow_and_shadowing() {
        let code = r#"
return 1
break
for file in file.list("/tmp"):
    file.read(file["file_name"])
"#;
        let res = analyze(code);
        assert_eq!(
            res.diagnostics
                .iter()
                .filter(
                    |d| d.code == Some(NumberOrString::String("invalid-control-flow".to_string()))
                )
                .count(),
            2
        );
        assert!(
            res.diagnostics
                .iter()
                .any(|d| { d.code == Some(NumberOrString::String("shadowed-stdlib".to_string())) })
        );
        assert!(res.diagnostics.iter().any(|d| {
            d.code == Some(NumberOrString::String("shadowed-stdlib-access".to_string()))
        }));
    }

    #[test]
    fn test_completions_hover_and_signature_help() {
        let stdlib = StdlibIndex::new();
        let linter = LintRegistry::default();
        let meta = TomeMetadata {
            name: "Test Tome".to_string(),
            paramdefs: vec![TomeParamDef {
                name: "target_path".to_string(),
                param_type: "string".to_string(),
                label: "Target file path".to_string(),
                placeholder: "/etc/passwd".to_string(),
                default: None,
            }],
            ..Default::default()
        };
        let code = "def my_helper(host: str, port: int = 80) -> str:\n    \"\"\"Connects to host.\"\"\"\n    return host\n\nfile.\ninput_params['\nsys.exec(\n";
        let res = analyze_document(code, &stdlib, &linter, Some(meta));

        // 1. Module member completion on `file.` (line 4, col 5)
        let file_completions = complete_at(
            &res,
            Position {
                line: 4,
                character: 5,
            },
            &stdlib,
        );
        assert!(file_completions.iter().any(|c| c.label == "read"));
        assert!(file_completions.iter().any(|c| c.label == "write"));
        assert!(file_completions.iter().any(|c| c.label == "timestomp"));

        // 2. input_params completion on `input_params['` (line 5, col 14)
        let param_completions = complete_at(
            &res,
            Position {
                line: 5,
                character: 14,
            },
            &stdlib,
        );
        assert!(param_completions.iter().any(|c| c.label == "target_path"));

        // 3. Signature help on `sys.exec(` (line 6, col 9)
        let sig = signature_help_at(
            &res,
            Position {
                line: 6,
                character: 9,
            },
            &stdlib,
        )
        .expect("Expected signature help for sys.exec");
        assert!(sig.signatures[0].label.contains("sys.exec"));

        // 4. Hover on `my_helper` (line 0, col 6)
        let hov = hover_at(
            &res,
            Position {
                line: 0,
                character: 6,
            },
            &stdlib,
        )
        .expect("Expected hover for my_helper");
        if let HoverContents::Markup(mc) = hov.contents {
            assert!(
                mc.value
                    .contains("def my_helper(host: str, port: int = 80) -> str")
            );
            assert!(mc.value.contains("Connects to host."));
        } else {
            panic!("Expected markup hover");
        }
    }
}
