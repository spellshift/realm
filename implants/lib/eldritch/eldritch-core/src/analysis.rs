use super::ast::{Argument, Expr, ExprKind, FStringSegment, Param, Stmt, StmtKind};
use super::token::Span;

#[derive(Debug, Clone)]
pub enum Node<'a> {
    Stmt(&'a Stmt),
    Expr(&'a Expr),
}

impl<'a> Node<'a> {
    pub fn span(&self) -> Span {
        match self {
            Node::Stmt(s) => s.span,
            Node::Expr(e) => e.span,
        }
    }
}

pub fn find_node_at_offset(ast: &[Stmt], offset: usize) -> Option<Node<'_>> {
    for stmt in ast {
        if stmt.span.start <= offset && offset <= stmt.span.end {
            if let Some(node) = find_in_stmt(stmt, offset) {
                return Some(node);
            }
            return Some(Node::Stmt(stmt));
        }
    }
    None
}

fn find_in_stmt(stmt: &Stmt, offset: usize) -> Option<Node<'_>> {
    match &stmt.kind {
        StmtKind::Expression(expr) => find_in_expr(expr, offset),
        StmtKind::Assignment(lhs, type_annot, rhs) => {
            if let Some(n) = find_in_expr(lhs, offset) {
                return Some(n);
            }
            if let Some(annot) = type_annot {
                if let Some(n) = find_in_expr(annot, offset) {
                    return Some(n);
                }
            }
            if let Some(n) = find_in_expr(rhs, offset) {
                return Some(n);
            }
            None
        }
        StmtKind::AugmentedAssignment(lhs, _, rhs) => {
            if let Some(n) = find_in_expr(lhs, offset) {
                return Some(n);
            }
            if let Some(n) = find_in_expr(rhs, offset) {
                return Some(n);
            }
            None
        }
        StmtKind::If(cond, then_block, else_block) => {
            if let Some(n) = find_in_expr(cond, offset) {
                return Some(n);
            }
            if let Some(n) = find_node_at_offset(then_block, offset) {
                return Some(n);
            }
            if let Some(block) = else_block {
                if let Some(n) = find_node_at_offset(block, offset) {
                    return Some(n);
                }
            }
            None
        }
        StmtKind::Return(Some(expr)) => find_in_expr(expr, offset),
        StmtKind::Def(_, params, return_annot, body) => {
            for param in params {
                match param {
                    Param::Normal(_, annot) | Param::Star(_, annot) | Param::StarStar(_, annot) => {
                        if let Some(a) = annot {
                            if let Some(n) = find_in_expr(a, offset) {
                                return Some(n);
                            }
                        }
                    }
                    Param::WithDefault(_, annot, default) => {
                        if let Some(a) = annot {
                            if let Some(n) = find_in_expr(a, offset) {
                                return Some(n);
                            }
                        }
                        if let Some(n) = find_in_expr(default, offset) {
                            return Some(n);
                        }
                    }
                }
            }
            if let Some(annot) = return_annot {
                if let Some(n) = find_in_expr(annot, offset) {
                    return Some(n);
                }
            }
            find_node_at_offset(body, offset)
        }
        StmtKind::For(_, iterable, body) => {
            if let Some(n) = find_in_expr(iterable, offset) {
                return Some(n);
            }
            find_node_at_offset(body, offset)
        }
        _ => None,
    }
}

fn find_in_expr(expr: &Expr, offset: usize) -> Option<Node<'_>> {
    if !(expr.span.start <= offset && offset <= expr.span.end) {
        return None;
    }

    let child = match &expr.kind {
        ExprKind::BinaryOp(l, _, r) | ExprKind::LogicalOp(l, _, r) => {
            find_in_expr(l, offset).or_else(|| find_in_expr(r, offset))
        }
        ExprKind::UnaryOp(_, e) => find_in_expr(e, offset),
        ExprKind::Call(callee, args) => find_in_expr(callee, offset).or_else(|| {
            for arg in args {
                let res = match arg {
                    Argument::Positional(e) | Argument::StarArgs(e) | Argument::KwArgs(e) => {
                        find_in_expr(e, offset)
                    }
                    Argument::Keyword(_, e) => find_in_expr(e, offset),
                };
                if res.is_some() {
                    return res;
                }
            }
            None
        }),
        ExprKind::List(exprs) | ExprKind::Tuple(exprs) | ExprKind::Set(exprs) => {
            for e in exprs {
                if let Some(n) = find_in_expr(e, offset) {
                    return Some(n);
                }
            }
            None
        }
        ExprKind::Dictionary(entries) => {
            for (k, v) in entries {
                if let Some(n) = find_in_expr(k, offset) {
                    return Some(n);
                }
                if let Some(n) = find_in_expr(v, offset) {
                    return Some(n);
                }
            }
            None
        }
        ExprKind::Index(obj, idx) => {
            find_in_expr(obj, offset).or_else(|| find_in_expr(idx, offset))
        }
        ExprKind::GetAttr(obj, _) => find_in_expr(obj, offset),
        ExprKind::Slice(obj, start, stop, step) => find_in_expr(obj, offset)
            .or_else(|| start.as_ref().and_then(|e| find_in_expr(e, offset)))
            .or_else(|| stop.as_ref().and_then(|e| find_in_expr(e, offset)))
            .or_else(|| step.as_ref().and_then(|e| find_in_expr(e, offset))),
        ExprKind::FString(segments) => {
            for seg in segments {
                if let FStringSegment::Expression(e) = seg {
                    if let Some(n) = find_in_expr(e, offset) {
                        return Some(n);
                    }
                }
            }
            None
        }
        ExprKind::ListComp {
            body,
            iterable,
            cond,
            ..
        }
        | ExprKind::SetComp {
            body,
            iterable,
            cond,
            ..
        } => find_in_expr(body, offset)
            .or_else(|| find_in_expr(iterable, offset))
            .or_else(|| cond.as_ref().and_then(|e| find_in_expr(e, offset))),
        ExprKind::DictComp {
            key,
            value,
            iterable,
            cond,
            ..
        } => find_in_expr(key, offset)
            .or_else(|| find_in_expr(value, offset))
            .or_else(|| find_in_expr(iterable, offset))
            .or_else(|| cond.as_ref().and_then(|e| find_in_expr(e, offset))),
        ExprKind::Lambda { params, body } => {
            for param in params {
                match param {
                    Param::Normal(_, annot) | Param::Star(_, annot) | Param::StarStar(_, annot) => {
                        if let Some(a) = annot {
                            if let Some(n) = find_in_expr(a, offset) {
                                return Some(n);
                            }
                        }
                    }
                    Param::WithDefault(_, annot, default) => {
                        if let Some(a) = annot {
                            if let Some(n) = find_in_expr(a, offset) {
                                return Some(n);
                            }
                        }
                        if let Some(n) = find_in_expr(default, offset) {
                            return Some(n);
                        }
                    }
                }
            }
            find_in_expr(body, offset)
        }
        ExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => find_in_expr(cond, offset)
            .or_else(|| find_in_expr(then_branch, offset))
            .or_else(|| find_in_expr(else_branch, offset)),
        _ => None,
    };

    if let Some(c) = child {
        Some(c)
    } else {
        Some(Node::Expr(expr))
    }
}
