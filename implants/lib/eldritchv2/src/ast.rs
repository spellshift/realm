use super::token::{Span, TokenKind};
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::fmt;

#[derive(Debug)]
pub struct Environment {
    pub parent: Option<Rc<RefCell<Environment>>>,
    pub values: BTreeMap<String, Value>,
}

#[derive(Debug, Clone)]
pub enum RuntimeParam {
    Normal(String),
    WithDefault(String, Value),
    Star(String),
    StarStar(String),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<RuntimeParam>,
    pub body: Vec<Stmt>,
    pub closure: Rc<RefCell<Environment>>,
}

#[derive(Debug, Clone)]
pub enum Param {
    Normal(String),
    WithDefault(String, Expr),
    Star(String),
    StarStar(String),
}

#[derive(Debug, Clone)]
pub enum Argument {
    Positional(Expr),
    Keyword(String, Expr),
    StarArgs(Expr),
    KwArgs(Expr),
}

pub type BuiltinFn = fn(&[Value]) -> Result<Value, String>;

#[derive(Debug, Clone)]
pub enum Value {
    None,
    Bool(bool),
    Int(i64),
    String(String),
    Bytes(Vec<u8>),
    List(Rc<RefCell<Vec<Value>>>),
    Tuple(Vec<Value>),
    Dictionary(Rc<RefCell<BTreeMap<String, Value>>>),
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
            (Value::Bytes(a), Value::Bytes(b)) => a == b,
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
            (Value::Tuple(a), Value::Tuple(b)) => a == b,
            (Value::Function(a), Value::Function(b)) => a.name == b.name,
            (Value::NativeFunction(a, _), Value::NativeFunction(b, _)) => a == b,
            (Value::BoundMethod(r1, n1), Value::BoundMethod(r2, n2)) => r1 == r2 && n1 == n2,
            _ => false,
        }
    }
}

impl Eq for Value {}

// Moved Display implementation here so it is available globally
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::None => write!(f, "None"),
            Value::Bool(b) => write!(f, "{}", if *b { "True" } else { "False" }),
            Value::Int(i) => write!(f, "{}", i),
            Value::String(s) => write!(f, "{}", s),
            Value::Bytes(b) => write!(f, "{:?}", b),
            Value::List(l) => {
                write!(f, "[")?;
                let list = l.borrow();
                for (i, v) in list.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Tuple(t) => {
                write!(f, "(")?;
                for (i, v) in t.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                if t.len() == 1 {
                    write!(f, ",")?;
                }
                write!(f, ")")
            }
            Value::Dictionary(d) => {
                write!(f, "{{")?;
                let dict = d.borrow();
                for (i, (k, v)) in dict.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
            // For functions, just print name to avoid circular dep with get_type_name
            Value::Function(func) => write!(f, "<function {}>", func.name),
            Value::NativeFunction(name, _) => write!(f, "<native function {}>", name),
            Value::BoundMethod(_, name) => write!(f, "<bound method {}>", name),
        }
    }
}

#[derive(Debug, Clone)]
pub enum FStringSegment {
    Literal(String),
    Expression(Expr),
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Literal(Value),
    Identifier(String),
    BinaryOp(Box<Expr>, TokenKind, Box<Expr>),
    UnaryOp(TokenKind, Box<Expr>),
    LogicalOp(Box<Expr>, TokenKind, Box<Expr>),
    Call(Box<Expr>, Vec<Argument>),
    List(Vec<Expr>),
    Tuple(Vec<Expr>),
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
    ListComp {
        body: Box<Expr>,
        var: String,
        iterable: Box<Expr>,
        cond: Option<Box<Expr>>,
    },
    DictComp {
        key: Box<Expr>,
        value: Box<Expr>,
        var: String,
        iterable: Box<Expr>,
        cond: Option<Box<Expr>>,
    },
    Lambda {
        params: Vec<Param>,
        body: Box<Expr>,
    },
    If {
        cond: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
}

#[derive(Debug, Clone)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum StmtKind {
    Expression(Expr),
    Assignment(Expr, Expr), // Changed from String to Expr for unpacking
    AugmentedAssignment(Expr, TokenKind, Expr), // Target, Op, Value
    If(Expr, Vec<Stmt>, Option<Vec<Stmt>>),
    Return(Option<Expr>),
    Def(String, Vec<Param>, Vec<Stmt>),
    For(Vec<String>, Expr, Vec<Stmt>),
    Break,
    Continue,
    Pass,
}
