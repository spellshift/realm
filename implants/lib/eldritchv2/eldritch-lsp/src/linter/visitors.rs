use eldritch_core::{Argument, ExprKind, Stmt, StmtKind};

pub fn visit_stmts<F>(stmts: &[Stmt], callback: &mut F)
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

pub fn visit_stmts_exprs<F>(stmts: &[Stmt], callback: &mut F)
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

pub fn visit_expr<F>(expr: &eldritch_core::Expr, callback: &mut F)
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
