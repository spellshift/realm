use super::interpreter::Printer;
use super::token::{Span, TokenKind};
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt;
use spin::RwLock;

#[derive(Debug)]
pub struct Environment {
    pub parent: Option<Arc<RwLock<Environment>>>,
    pub values: BTreeMap<String, Value>,
    pub printer: Arc<dyn Printer + Send + Sync>,
    pub libraries: BTreeSet<String>,
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
    pub closure: Arc<RwLock<Environment>>,
}

#[derive(Debug, Clone)]
pub enum Param {
    Normal(String, Option<Box<Expr>>),
    WithDefault(String, Option<Box<Expr>>, Expr),
    Star(String, Option<Box<Expr>>),
    StarStar(String, Option<Box<Expr>>),
}

#[derive(Debug, Clone)]
pub enum Argument {
    Positional(Expr),
    Keyword(String, Expr),
    StarArgs(Expr),
    KwArgs(Expr),
}

pub type BuiltinFn = fn(&Arc<RwLock<Environment>>, &[Value]) -> Result<Value, String>;
pub type BuiltinFnWithKwargs =
    fn(&Arc<RwLock<Environment>>, &[Value], &BTreeMap<String, Value>) -> Result<Value, String>;

pub trait ForeignValue: fmt::Debug + Send + Sync {
    fn type_name(&self) -> &str;
    fn method_names(&self) -> Vec<String>;
    fn call_method(
        &self,
        name: &str,
        args: &[Value],
        kwargs: &BTreeMap<String, Value>,
    ) -> Result<Value, String>;
}

#[derive(Clone)]
pub enum Value {
    None,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    List(Arc<RwLock<Vec<Value>>>),
    Tuple(Vec<Value>),
    Dictionary(Arc<RwLock<BTreeMap<Value, Value>>>),
    Set(Arc<RwLock<BTreeSet<Value>>>),
    Function(Function),
    NativeFunction(String, BuiltinFn),
    NativeFunctionWithKwargs(String, BuiltinFnWithKwargs),
    BoundMethod(Box<Value>, String),
    Foreign(Arc<dyn ForeignValue>),
}

// Implement repr-like behavior for Debug
impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::None => write!(f, "None"),
            Value::Bool(b) => write!(f, "{}", if *b { "True" } else { "False" }),
            Value::Int(i) => write!(f, "{i}"),
            Value::Float(fl) => write!(f, "{fl:?}"),
            Value::String(s) => write!(f, "{s:?}"), // Quoted string
            Value::Bytes(b) => {
                // Heuristic: if all bytes are printable ASCII, print as b"...", else [...]
                // Or just stick to python's repr(bytes) which uses hex escapes for non-printable.
                // For simplicity and correctness with existing types, let's use the Rust debug for byte literal if possible,
                // or just `b` prefix.
                // `write!(f, "b{:?}", b)` prints `b[1, 2]`. We want `b"..."`.
                // We can convert to string with escaping.
                // Simple version:
                write!(f, "b\"")?;
                for byte in b {
                    match byte {
                        b'\n' => write!(f, "\\n")?,
                        b'\r' => write!(f, "\\r")?,
                        b'\t' => write!(f, "\\t")?,
                        b'\\' => write!(f, "\\\\")?,
                        b'"' => write!(f, "\\\"")?,
                        0x20..=0x7E => write!(f, "{}", *byte as char)?,
                        _ => write!(f, "\\x{:02x}", byte)?,
                    }
                }
                write!(f, "\"")
            }
            Value::List(l) => {
                write!(f, "[")?;
                let list = l.read();
                for (i, v) in list.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{v:?}")?;
                }
                write!(f, "]")
            }
            Value::Tuple(t) => {
                write!(f, "(")?;
                for (i, v) in t.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{v:?}")?;
                }
                if t.len() == 1 {
                    write!(f, ",")?;
                }
                write!(f, ")")
            }
            Value::Dictionary(d) => {
                write!(f, "{{")?;
                let dict = d.read();
                for (i, (k, v)) in dict.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{k:?}: {v:?}")?;
                }
                write!(f, "}}")
            }
            Value::Set(s) => {
                let set = s.read();
                if set.is_empty() {
                    write!(f, "set()")
                } else {
                    write!(f, "{{")?;
                    for (i, v) in set.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{v:?}")?;
                    }
                    write!(f, "}}")
                }
            }
            Value::Function(func) => write!(f, "<function {}>", func.name),
            Value::NativeFunction(name, _) => write!(f, "<native function {name}>"),
            Value::NativeFunctionWithKwargs(name, _) => write!(f, "<native function {name}>"),
            Value::BoundMethod(_, name) => write!(f, "<bound method {name}>"),
            Value::Foreign(obj) => write!(f, "<{}>", obj.type_name()),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::None, Value::None) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b, // Note: NaN != NaN usually, but handled by PartialOrd? No, PartialEq
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bytes(a), Value::Bytes(b)) => a == b,
            (Value::List(a), Value::List(b)) => {
                if Arc::ptr_eq(a, b) {
                    return true;
                }
                a.read().eq(&*b.read())
            }
            (Value::Dictionary(a), Value::Dictionary(b)) => {
                if Arc::ptr_eq(a, b) {
                    return true;
                }
                a.read().eq(&*b.read())
            }
            (Value::Set(a), Value::Set(b)) => {
                if Arc::ptr_eq(a, b) {
                    return true;
                }
                a.read().eq(&*b.read())
            }
            (Value::Tuple(a), Value::Tuple(b)) => a == b,
            (Value::Function(a), Value::Function(b)) => a.name == b.name,
            (Value::NativeFunction(a, _), Value::NativeFunction(b, _)) => a == b,
            (Value::NativeFunctionWithKwargs(a, _), Value::NativeFunctionWithKwargs(b, _)) => {
                a == b
            }
            (Value::BoundMethod(r1, n1), Value::BoundMethod(r2, n2)) => r1 == r2 && n1 == n2,
            (Value::Foreign(a), Value::Foreign(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl Eq for Value {}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        // Define an ordering between types:
        // None < Bool < Int < Float < String < Bytes < List < Tuple < Dict < Set < Function < Native < Bound < Foreign
        let self_discriminant = self.discriminant_value();
        let other_discriminant = other.discriminant_value();

        if self_discriminant != other_discriminant {
            return self_discriminant.cmp(&other_discriminant);
        }

        match (self, other) {
            (Value::None, Value::None) => Ordering::Equal,
            (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
            (Value::Int(a), Value::Int(b)) => a.cmp(b),
            (Value::Float(a), Value::Float(b)) => a.total_cmp(b),
            (Value::String(a), Value::String(b)) => a.cmp(b),
            (Value::Bytes(a), Value::Bytes(b)) => a.cmp(b),
            (Value::List(a), Value::List(b)) => {
                if Arc::ptr_eq(a, b) {
                    return Ordering::Equal;
                }
                a.read().cmp(&*b.read())
            }
            (Value::Tuple(a), Value::Tuple(b)) => a.cmp(b),
            (Value::Dictionary(a), Value::Dictionary(b)) => {
                if Arc::ptr_eq(a, b) {
                    return Ordering::Equal;
                }
                // BTreeMap implements Ord
                a.read().cmp(&*b.read())
            }
            (Value::Set(a), Value::Set(b)) => {
                if Arc::ptr_eq(a, b) {
                    return Ordering::Equal;
                }
                // BTreeSet implements Ord
                a.read().cmp(&*b.read())
            }
            // For functions and others, we just compare pointers or names as best effort
            // This is primarily to satisfy BTreeSet requirement, not for user-facing logical ordering necessarily.
            (Value::Function(a), Value::Function(b)) => a.name.cmp(&b.name),
            (Value::NativeFunction(a, _), Value::NativeFunction(b, _)) => a.cmp(b),
            (Value::NativeFunctionWithKwargs(a, _), Value::NativeFunctionWithKwargs(b, _)) => {
                a.cmp(b)
            }
            (Value::BoundMethod(r1, n1), Value::BoundMethod(r2, n2)) => match r1.cmp(r2) {
                Ordering::Equal => n1.cmp(n2),
                ord => ord,
            },
            (Value::Foreign(a), Value::Foreign(b)) => {
                let p1 = Arc::as_ptr(a) as *const ();
                let p2 = Arc::as_ptr(b) as *const ();
                p1.cmp(&p2)
            }
            _ => Ordering::Equal, // Should be covered by discriminant check
        }
    }
}

impl Value {
    fn discriminant_value(&self) -> u8 {
        match self {
            Value::None => 0,
            Value::Bool(_) => 1,
            Value::Int(_) => 2,
            Value::Float(_) => 3,
            Value::String(_) => 4,
            Value::Bytes(_) => 5,
            Value::List(_) => 6,
            Value::Tuple(_) => 7,
            Value::Dictionary(_) => 8,
            Value::Set(_) => 9,
            Value::Function(_) => 10,
            Value::NativeFunction(_, _) => 11,
            Value::NativeFunctionWithKwargs(_, _) => 12,
            Value::BoundMethod(_, _) => 13,
            Value::Foreign(_) => 14,
        }
    }
}

// Display implementation (equivalent to Python str())
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::None => write!(f, "None"),
            Value::Bool(b) => write!(f, "{}", if *b { "True" } else { "False" }),
            Value::Int(i) => write!(f, "{i}"),
            Value::Float(fl) => write!(f, "{fl:?}"), // Use Debug for floats to get decent formatting (1.0 etc)
            Value::String(s) => write!(f, "{s}"), // Strings print without quotes in str()
            Value::Bytes(b) => write!(f, "{:?}", Value::Bytes(b.clone())), // Delegate to Debug for bytes representation
            Value::List(l) => {
                // Containers use repr (Debug) for their elements
                write!(f, "{:?}", Value::List(l.clone()))
            }
            Value::Tuple(t) => {
                write!(f, "{:?}", Value::Tuple(t.clone()))
            }
            Value::Dictionary(d) => {
                write!(f, "{:?}", Value::Dictionary(d.clone()))
            }
            Value::Set(s) => {
                write!(f, "{:?}", Value::Set(s.clone()))
            }
            Value::Function(func) => write!(f, "<function {}>", func.name),
            Value::NativeFunction(name, _) => write!(f, "<native function {name}>"),
            Value::NativeFunctionWithKwargs(name, _) => write!(f, "<native function {name}>"),
            Value::BoundMethod(_, name) => write!(f, "<bound method {name}>"),
            Value::Foreign(obj) => write!(f, "<{}>", obj.type_name()),
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
    Set(Vec<Expr>),
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
    SetComp {
        body: Box<Expr>,
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
    Assignment(Expr, Option<Box<Expr>>, Expr),
    AugmentedAssignment(Expr, TokenKind, Expr),
    If(Expr, Vec<Stmt>, Option<Vec<Stmt>>),
    Return(Option<Expr>),
    Def(String, Vec<Param>, Option<Box<Expr>>, Vec<Stmt>),
    For(Vec<String>, Expr, Vec<Stmt>),
    Break,
    Continue,
    Pass,
}
