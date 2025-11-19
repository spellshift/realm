use crate::ast::{Expression, Program, Statement};
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Integer(i64),
    Boolean(bool),
    ReturnValue(Box<Object>),
}

pub struct Evaluator {
    env: BTreeMap<String, Object>,
}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {
            env: BTreeMap::new(),
        }
    }

    pub fn eval_program(&mut self, program: &Program) -> Option<Object> {
        let mut result = None;
        for statement in &program.statements {
            result = self.eval_statement(statement);
            if let Some(Object::ReturnValue(value)) = result {
                return Some(*value);
            }
        }
        result
    }

    fn eval_statement(&mut self, statement: &Statement) -> Option<Object> {
        match statement {
            Statement::Simple(stmts) => {
                let mut result = None;
                for stmt in stmts {
                    result = self.eval_small_statement(stmt);
                    if let Some(Object::ReturnValue(_)) = result {
                        return result;
                    }
                }
                result
            }
            Statement::If {
                condition,
                consequence,
                ..
            } => {
                let condition = self.eval_expression(condition)?;
                if self.is_truthy(condition) {
                    self.eval_program(&crate::ast::Program {
                        statements: consequence.statements.clone(),
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn eval_small_statement(&mut self, statement: &crate::ast::SmallStmt) -> Option<Object> {
        match statement {
            crate::ast::SmallStmt::Expr(exprs) => {
                let mut result = None;
                for expr in exprs {
                    result = self.eval_expression(expr);
                }
                result
            }
            crate::ast::SmallStmt::Return(Some(expressions)) => {
                let value = self.eval_expression(&expressions[0])?;
                Some(Object::ReturnValue(Box::new(value)))
            }
            _ => None,
        }
    }

    fn eval_expression(&mut self, expression: &Expression) -> Option<Object> {
        match expression {
            Expression::Primary(primary) => match primary {
                crate::ast::PrimaryExpr::Operand(operand) => match operand {
                    crate::ast::Operand::Int(value) => Some(Object::Integer(*value)),
                    crate::ast::Operand::Identifier(name) => match name.as_str() {
                        "true" => Some(Object::Boolean(true)),
                        "false" => Some(Object::Boolean(false)),
                        _ => self.env.get(name).cloned(),
                    },
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        }
    }

    fn is_truthy(&self, object: Object) -> bool {
        match object {
            Object::Boolean(value) => value,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests;
