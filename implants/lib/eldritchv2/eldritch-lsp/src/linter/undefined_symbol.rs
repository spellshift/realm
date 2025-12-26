use eldritch_core::{Argument, ExprKind, Param, Stmt, StmtKind};
use eldritchv2::Interpreter;
use lsp_types::{Diagnostic, DiagnosticSeverity};
use std::collections::HashSet;

use super::{utils::span_to_range, LintRule};

pub struct UndefinedSymbolRule;

impl LintRule for UndefinedSymbolRule {
    fn name(&self) -> &str {
        "undefined_symbol"
    }

    fn check(&self, stmts: &[Stmt], source: &str, interp: &Interpreter) -> Vec<Diagnostic> {
        let mut visitor = SymbolVisitor::new(source, interp);
        visitor.visit_stmts(stmts);
        visitor.diagnostics
    }
}

struct Scope {
    vars: HashSet<String>,
}

struct SymbolVisitor<'a> {
    scopes: Vec<Scope>,
    diagnostics: Vec<Diagnostic>,
    source: &'a str,
    interp: &'a Interpreter,
}

impl<'a> SymbolVisitor<'a> {
    fn new(source: &'a str, interp: &'a Interpreter) -> Self {
        Self {
            scopes: vec![Scope {
                vars: HashSet::new(),
            }], // Module scope
            diagnostics: Vec::new(),
            source,
            interp,
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(Scope {
            vars: HashSet::new(),
        });
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn define(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.vars.insert(name.to_string());
        }
    }

    fn is_defined(&self, name: &str) -> bool {
        // Check local scopes
        for scope in self.scopes.iter().rev() {
            if scope.vars.contains(name) {
                return true;
            }
        }
        // Check interpreter globals/builtins
        self.interp.lookup_variable(name).is_some()
    }

    fn visit_stmts(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            self.visit_stmt(stmt);
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Expression(expr) => self.visit_expr(expr),
            StmtKind::Assignment(lhs, type_hint, rhs) => {
                // Visit rhs first (usage)
                self.visit_expr(rhs);
                // Visit type hint (usage)
                if let Some(th) = type_hint {
                    self.visit_expr(th);
                }
                // Define vars in lhs
                self.visit_assignment_target(lhs);
            }
            StmtKind::AugmentedAssignment(lhs, _, rhs) => {
                self.visit_expr(rhs);
                // lhs is also read in augmented assignment
                self.visit_expr(lhs);
            }
            StmtKind::If(cond, then_branch, else_branch) => {
                self.visit_expr(cond);
                self.visit_stmts(then_branch);
                if let Some(else_stmts) = else_branch {
                    self.visit_stmts(else_stmts);
                }
            }
            StmtKind::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    self.visit_expr(expr);
                }
            }
            StmtKind::Def(name, params, ret_hint, body) => {
                // Function name defined in current scope
                self.define(name);

                // Default values and type hints evaluated in current scope
                for param in params {
                    match param {
                        Param::Normal(_, hint) => {
                            if let Some(h) = hint {
                                self.visit_expr(h);
                            }
                        }
                        Param::WithDefault(_, hint, default) => {
                            if let Some(h) = hint {
                                self.visit_expr(h);
                            }
                            self.visit_expr(default);
                        }
                        Param::Star(_, hint) => {
                            if let Some(h) = hint {
                                self.visit_expr(h);
                            }
                        }
                        Param::StarStar(_, hint) => {
                            if let Some(h) = hint {
                                self.visit_expr(h);
                            }
                        }
                    }
                }
                if let Some(h) = ret_hint {
                    self.visit_expr(h);
                }

                self.push_scope();
                for param in params {
                    match param {
                        Param::Normal(n, _) => self.define(n),
                        Param::WithDefault(n, _, _) => self.define(n),
                        Param::Star(n, _) => self.define(n),
                        Param::StarStar(n, _) => self.define(n),
                    }
                }
                self.visit_stmts(body);
                self.pop_scope();
            }
            StmtKind::For(vars, iterable, body) => {
                self.visit_expr(iterable);
                self.push_scope();
                for v in vars {
                    self.define(v);
                }
                self.visit_stmts(body);
                self.pop_scope();
            }
            StmtKind::Break => {}
            StmtKind::Continue => {}
            StmtKind::Pass => {}
        }
    }

    fn visit_assignment_target(&mut self, expr: &eldritch_core::Expr) {
        match &expr.kind {
            ExprKind::Identifier(name) => self.define(name),
            ExprKind::Tuple(exprs) | ExprKind::List(exprs) => {
                for e in exprs {
                    self.visit_assignment_target(e);
                }
            }
            ExprKind::Index(obj, idx) => {
                // Not defining obj or idx, but using them
                self.visit_expr(obj);
                self.visit_expr(idx);
            }
            ExprKind::GetAttr(obj, _) => {
                // Attribute assignment: obj.attr = val. 'obj' is used.
                self.visit_expr(obj);
            }
            _ => {
                // Other expressions on LHS are likely invalid but if valid, treat as usages?
                // Actually if they appear on LHS they might be fancy unpacking or just invalid.
                // We'll visit them as usages just in case.
                self.visit_expr(expr);
            }
        }
    }

    fn visit_expr(&mut self, expr: &eldritch_core::Expr) {
        match &expr.kind {
            ExprKind::Identifier(name) => {
                if !self.is_defined(name) {
                    self.diagnostics.push(Diagnostic {
                        range: span_to_range(expr.span, self.source),
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: format!("NameError: name '{}' is not defined", name),
                        ..Default::default()
                    });
                }
            }
            ExprKind::Literal(_) => {}
            ExprKind::BinaryOp(lhs, _, rhs) => {
                self.visit_expr(lhs);
                self.visit_expr(rhs);
            }
            ExprKind::UnaryOp(_, op) => self.visit_expr(op),
            ExprKind::LogicalOp(lhs, _, rhs) => {
                self.visit_expr(lhs);
                self.visit_expr(rhs);
            }
            ExprKind::Call(callee, args) => {
                self.visit_expr(callee);
                for arg in args {
                    match arg {
                        Argument::Positional(e) => self.visit_expr(e),
                        Argument::Keyword(_, e) => self.visit_expr(e),
                        Argument::StarArgs(e) => self.visit_expr(e),
                        Argument::KwArgs(e) => self.visit_expr(e),
                    }
                }
            }
            ExprKind::List(exprs) => {
                for e in exprs {
                    self.visit_expr(e);
                }
            }
            ExprKind::Tuple(exprs) => {
                for e in exprs {
                    self.visit_expr(e);
                }
            }
            ExprKind::Dictionary(kv_pairs) => {
                for (k, v) in kv_pairs {
                    self.visit_expr(k);
                    self.visit_expr(v);
                }
            }
            ExprKind::Set(exprs) => {
                for e in exprs {
                    self.visit_expr(e);
                }
            }
            ExprKind::Index(obj, idx) => {
                self.visit_expr(obj);
                self.visit_expr(idx);
            }
            ExprKind::GetAttr(obj, _) => {
                self.visit_expr(obj);
            }
            ExprKind::Slice(obj, start, end, step) => {
                self.visit_expr(obj);
                if let Some(e) = start {
                    self.visit_expr(e);
                }
                if let Some(e) = end {
                    self.visit_expr(e);
                }
                if let Some(e) = step {
                    self.visit_expr(e);
                }
            }
            ExprKind::FString(segments) => {
                for seg in segments {
                    if let eldritch_core::FStringSegment::Expression(e) = seg {
                        self.visit_expr(e);
                    }
                }
            }
            ExprKind::ListComp {
                body,
                var,
                iterable,
                cond,
            } => {
                self.visit_expr(iterable);
                self.push_scope();
                self.define(var);
                if let Some(c) = cond {
                    self.visit_expr(c);
                }
                self.visit_expr(body);
                self.pop_scope();
            }
            ExprKind::DictComp {
                key,
                value,
                var,
                iterable,
                cond,
            } => {
                self.visit_expr(iterable);
                self.push_scope();
                self.define(var);
                if let Some(c) = cond {
                    self.visit_expr(c);
                }
                self.visit_expr(key);
                self.visit_expr(value);
                self.pop_scope();
            }
            ExprKind::SetComp {
                body,
                var,
                iterable,
                cond,
            } => {
                self.visit_expr(iterable);
                self.push_scope();
                self.define(var);
                if let Some(c) = cond {
                    self.visit_expr(c);
                }
                self.visit_expr(body);
                self.pop_scope();
            }
            ExprKind::Lambda { params, body } => {
                for param in params {
                    match param {
                        Param::WithDefault(_, hint, default) => {
                            if let Some(h) = hint {
                                self.visit_expr(h);
                            }
                            self.visit_expr(default);
                        }
                        Param::Normal(_, hint)
                        | Param::Star(_, hint)
                        | Param::StarStar(_, hint) => {
                            if let Some(h) = hint {
                                self.visit_expr(h);
                            }
                        }
                    }
                }
                self.push_scope();
                for param in params {
                    match param {
                        Param::Normal(n, _) => self.define(n),
                        Param::WithDefault(n, _, _) => self.define(n),
                        Param::Star(n, _) => self.define(n),
                        Param::StarStar(n, _) => self.define(n),
                    }
                }
                self.visit_expr(body);
                self.pop_scope();
            }
            ExprKind::If {
                cond,
                then_branch,
                else_branch,
            } => {
                self.visit_expr(cond);
                self.visit_expr(then_branch);
                self.visit_expr(else_branch);
            }
        }
    }
}
