use super::token::Token;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Environment {
    pub parent: Option<Rc<RefCell<Environment>>>,
    pub values: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
    pub closure: Rc<RefCell<Environment>>,
}

pub type BuiltinFn = fn(&[Value]) -> Result<Value, String>;

#[derive(Debug, Clone)]
pub enum Value {
    None,
    Bool(bool),
    Int(i64),
    String(String),
    List(Rc<RefCell<Vec<Value>>>),
    Tuple(Vec<Value>), // New: Tuples are immutable, so plain Vec is fine
    Dictionary(Rc<RefCell<HashMap<String, Value>>>),
    Function(Function),
    NativeFunction(String, BuiltinFn),
    BoundMethod(Box<Value>, String),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::None, Value::None) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::List(a), Value::List(b)) => {
                if Rc::ptr_eq(a, b) {
                    return true;
                }
                a.borrow().eq(&*b.borrow())
            }
            (Value::Dictionary(a), Value::Dictionary(b)) => {
                if Rc::ptr_eq(a, b) {
                    return true;
                }
                a.borrow().eq(&*b.borrow())
            }
            (Value::Tuple(a), Value::Tuple(b)) => a == b, // Deep equality for tuples
            (Value::Function(a), Value::Function(b)) => a.name == b.name,
            (Value::NativeFunction(a, _), Value::NativeFunction(b, _)) => a == b,
            (Value::BoundMethod(r1, n1), Value::BoundMethod(r2, n2)) => r1 == r2 && n1 == n2,
            _ => false,
        }
    }
}

impl Eq for Value {}

#[derive(Debug, Clone)]
pub enum FStringSegment {
    Literal(String),
    Expression(Expr),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Value),
    Identifier(String),
    BinaryOp(Box<Expr>, Token, Box<Expr>),
    UnaryOp(Token, Box<Expr>),
    LogicalOp(Box<Expr>, Token, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    List(Vec<Expr>),
    Tuple(Vec<Expr>), // New
    Dictionary(Vec<(Expr, Expr)>),
    Index(Box<Expr>, Box<Expr>),
    GetAttr(Box<Expr>, String),
    Slice(
        Box<Expr>,
        Option<Box<Expr>>,
        Option<Box<Expr>>,
        Option<Box<Expr>>,
    ),
    FString(Vec<FStringSegment>),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Expr),
    Assignment(String, Expr),
    If(Expr, Vec<Stmt>, Option<Vec<Stmt>>),
    Return(Option<Expr>),
    Def(String, Vec<String>, Vec<Stmt>),
    For(String, Expr, Vec<Stmt>),
    Break,
    Continue,
}
