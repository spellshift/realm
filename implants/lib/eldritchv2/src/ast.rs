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

// Type alias for native/built-in function implementations
pub type BuiltinFn = fn(&[Value]) -> Result<Value, String>;

#[derive(Debug, Clone)]
pub enum Value {
    None,
    Bool(bool),
    Int(i64),
    String(String),
    // Use Rc<RefCell<...>> for mutable, shareable data structures
    List(Rc<RefCell<Vec<Value>>>),
    Dictionary(Rc<RefCell<HashMap<String, Value>>>),
    Function(Function), // Holds the full function definition and closure environment
    NativeFunction(String, BuiltinFn), // (name, function pointer)
}

// Manual implementation of PartialEq because Rc<RefCell<...>> does not implement it,
// but we need comparison for basic types like Int and Bool.
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::None, Value::None) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,

            // For Lists and Dictionaries, we rely on reference equality for simplicity
            // in the core language comparison operations (==, !=). Deep equality
            // is usually implemented via a custom native function if needed.
            (Value::List(a), Value::List(b)) => Rc::ptr_eq(a, b),
            (Value::Dictionary(a), Value::Dictionary(b)) => Rc::ptr_eq(a, b),

            // Functions are equal if their memory references are the same
            (Value::Function(a), Value::Function(b)) => a.name == b.name,
            (Value::NativeFunction(a, _), Value::NativeFunction(b, _)) => a == b,

            _ => false,
        }
    }
}

// Required if PartialEq is implemented manually
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
    Call(Box<Expr>, Vec<Expr>),
    List(Vec<Expr>),
    Dictionary(Vec<(Expr, Expr)>),
    Index(Box<Expr>, Box<Expr>), // New: Index/Subscript expression: obj[index]
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
}
