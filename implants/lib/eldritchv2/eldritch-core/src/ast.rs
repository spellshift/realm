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

// Resolve circular reference for ForeignValue signature
use crate::interpreter::Interpreter;

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
        interp: &mut Interpreter,
        name: &str,
        args: &[Value],
        kwargs: &BTreeMap<String, Value>,
    ) -> Result<Value, String>;
    fn get_attr(&self, _name: &str) -> Option<Value> {
        None
    }
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
        let mut visited = BTreeSet::new();
        self.fmt_helper(f, &mut visited)
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        let mut visited = BTreeSet::new();
        self.eq_helper(other, &mut visited)
    }
}

impl Value {
    fn eq_helper(&self, other: &Self, visited: &mut BTreeSet<(usize, usize)>) -> bool {
        let p1 = match self {
            Value::List(l) => Arc::as_ptr(l) as usize,
            Value::Dictionary(d) => Arc::as_ptr(d) as usize,
            Value::Set(s) => Arc::as_ptr(s) as usize,
            _ => 0,
        };
        let p2 = match other {
            Value::List(l) => Arc::as_ptr(l) as usize,
            Value::Dictionary(d) => Arc::as_ptr(d) as usize,
            Value::Set(s) => Arc::as_ptr(s) as usize,
            _ => 0,
        };

        if p1 != 0 && p2 != 0 {
            let pair = if p1 < p2 { (p1, p2) } else { (p2, p1) };
            if visited.contains(&pair) {
                return true;
            }
            visited.insert(pair);
        }

        let result = match (self, other) {
            (Value::None, Value::None) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bytes(a), Value::Bytes(b)) => a == b,
            (Value::List(a), Value::List(b)) => {
                if Arc::ptr_eq(a, b) {
                    true
                } else {
                    let la = a.read();
                    let lb = b.read();
                    if la.len() != lb.len() {
                        false
                    } else {
                        la.iter()
                            .zip(lb.iter())
                            .all(|(va, vb)| va.eq_helper(vb, visited))
                    }
                }
            }
            (Value::Dictionary(a), Value::Dictionary(b)) => {
                if Arc::ptr_eq(a, b) {
                    true
                } else {
                    let da = a.read();
                    let db = b.read();
                    if da.len() != db.len() {
                        false
                    } else {
                        da.iter().zip(db.iter()).all(|((ka, va), (kb, vb))| {
                            ka.eq_helper(kb, visited) && va.eq_helper(vb, visited)
                        })
                    }
                }
            }
            (Value::Set(a), Value::Set(b)) => {
                if Arc::ptr_eq(a, b) {
                    true
                } else {
                    let sa = a.read();
                    let sb = b.read();
                    if sa.len() != sb.len() {
                        false
                    } else {
                        sa.iter()
                            .zip(sb.iter())
                            .all(|(va, vb)| va.eq_helper(vb, visited))
                    }
                }
            }
            (Value::Tuple(a), Value::Tuple(b)) => {
                if a.len() != b.len() {
                    false
                } else {
                    a.iter()
                        .zip(b.iter())
                        .all(|(va, vb)| va.eq_helper(vb, visited))
                }
            }
            (Value::Function(a), Value::Function(b)) => a.name == b.name,
            (Value::NativeFunction(a, _), Value::NativeFunction(b, _)) => a == b,
            (Value::NativeFunctionWithKwargs(a, _), Value::NativeFunctionWithKwargs(b, _)) => {
                a == b
            }
            (Value::BoundMethod(r1, n1), Value::BoundMethod(r2, n2)) => r1 == r2 && n1 == n2,
            (Value::Foreign(a), Value::Foreign(b)) => Arc::ptr_eq(a, b),
            _ => false,
        };

        if p1 != 0 && p2 != 0 {
            let pair = if p1 < p2 { (p1, p2) } else { (p2, p1) };
            visited.remove(&pair);
        }

        result
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
        let mut visited = BTreeSet::new();
        self.cmp_helper(other, &mut visited)
    }
}

impl Value {
    fn cmp_helper(&self, other: &Self, visited: &mut BTreeSet<(usize, usize)>) -> Ordering {
        let p1 = match self {
            Value::List(l) => Arc::as_ptr(l) as usize,
            Value::Dictionary(d) => Arc::as_ptr(d) as usize,
            Value::Set(s) => Arc::as_ptr(s) as usize,
            _ => 0,
        };
        let p2 = match other {
            Value::List(l) => Arc::as_ptr(l) as usize,
            Value::Dictionary(d) => Arc::as_ptr(d) as usize,
            Value::Set(s) => Arc::as_ptr(s) as usize,
            _ => 0,
        };

        if p1 != 0 && p2 != 0 {
            let pair = (p1, p2);
            if visited.contains(&pair) {
                return Ordering::Equal;
            }
            visited.insert(pair);
        }

        // Special case for Int vs Float comparison to make them behave numerically
        // This must be done before discriminant check because they have different discriminants
        // but we want them to compare by value.
        match (self, other) {
            (Value::Int(i), Value::Float(f)) => {
                if p1 != 0 && p2 != 0 {
                    let pair = (p1, p2);
                    visited.remove(&pair);
                }
                return (*i as f64).total_cmp(f);
            }
            (Value::Float(f), Value::Int(i)) => {
                if p1 != 0 && p2 != 0 {
                    let pair = (p1, p2);
                    visited.remove(&pair);
                }
                return f.total_cmp(&(*i as f64));
            }
            _ => {}
        }

        // Define an ordering between types:
        // None < Bool < Int < Float < String < Bytes < List < Tuple < Dict < Set < Function < Native < Bound < Foreign
        let self_discriminant = self.discriminant_value();
        let other_discriminant = other.discriminant_value();

        if self_discriminant != other_discriminant {
            if p1 != 0 && p2 != 0 {
                let pair = (p1, p2);
                visited.remove(&pair);
            }
            return self_discriminant.cmp(&other_discriminant);
        }

        let result = match (self, other) {
            (Value::None, Value::None) => Ordering::Equal,
            (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
            (Value::Int(a), Value::Int(b)) => a.cmp(b),
            (Value::Float(a), Value::Float(b)) => a.total_cmp(b),
            (Value::String(a), Value::String(b)) => a.cmp(b),
            (Value::Bytes(a), Value::Bytes(b)) => a.cmp(b),
            (Value::List(a), Value::List(b)) => {
                if Arc::ptr_eq(a, b) {
                    Ordering::Equal
                } else {
                    let la = a.read();
                    let lb = b.read();
                    // Lexicographical comparison with recursion
                    let len = la.len().min(lb.len());
                    let mut ord = Ordering::Equal;
                    for i in 0..len {
                        ord = la[i].cmp_helper(&lb[i], visited);
                        if ord != Ordering::Equal {
                            break;
                        }
                    }
                    if ord == Ordering::Equal {
                        la.len().cmp(&lb.len())
                    } else {
                        ord
                    }
                }
            }
            (Value::Tuple(a), Value::Tuple(b)) => {
                let len = a.len().min(b.len());
                let mut ord = Ordering::Equal;
                for i in 0..len {
                    ord = a[i].cmp_helper(&b[i], visited);
                    if ord != Ordering::Equal {
                        break;
                    }
                }
                if ord == Ordering::Equal {
                    a.len().cmp(&b.len())
                } else {
                    ord
                }
            }
            (Value::Dictionary(a), Value::Dictionary(b)) => {
                if Arc::ptr_eq(a, b) {
                    Ordering::Equal
                } else {
                    let da = a.read();
                    let db = b.read();
                    // Iterate and compare (key, value) pairs
                    let mut it1 = da.iter();
                    let mut it2 = db.iter();
                    loop {
                        match (it1.next(), it2.next()) {
                            (Some((k1, v1)), Some((k2, v2))) => {
                                let mut ord = k1.cmp_helper(k2, visited);
                                if ord == Ordering::Equal {
                                    ord = v1.cmp_helper(v2, visited);
                                }
                                if ord != Ordering::Equal {
                                    break ord;
                                }
                            }
                            (Some(_), None) => {
                                break Ordering::Greater;
                            }
                            (None, Some(_)) => {
                                break Ordering::Less;
                            }
                            (None, None) => {
                                break Ordering::Equal;
                            }
                        }
                    }
                }
            }
            (Value::Set(a), Value::Set(b)) => {
                if Arc::ptr_eq(a, b) {
                    Ordering::Equal
                } else {
                    let sa = a.read();
                    let sb = b.read();
                    let mut it1 = sa.iter();
                    let mut it2 = sb.iter();
                    loop {
                        match (it1.next(), it2.next()) {
                            (Some(v1), Some(v2)) => {
                                let ord = v1.cmp_helper(v2, visited);
                                if ord != Ordering::Equal {
                                    break ord;
                                }
                            }
                            (Some(_), None) => {
                                break Ordering::Greater;
                            }
                            (None, Some(_)) => {
                                break Ordering::Less;
                            }
                            (None, None) => {
                                break Ordering::Equal;
                            }
                        }
                    }
                }
            }
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
        };

        if p1 != 0 && p2 != 0 {
            let pair = (p1, p2);
            visited.remove(&pair);
        }

        result
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

    fn fmt_helper(&self, f: &mut fmt::Formatter<'_>, visited: &mut BTreeSet<usize>) -> fmt::Result {
        match self {
            Value::None => write!(f, "None"),
            Value::Bool(b) => write!(f, "{}", if *b { "True" } else { "False" }),
            Value::Int(i) => write!(f, "{i}"),
            Value::Float(fl) => write!(f, "{fl:?}"),
            Value::String(s) => write!(f, "{s:?}"),
            Value::Bytes(b) => {
                write!(f, "b\"")?;
                for byte in b {
                    match byte {
                        b'\n' => write!(f, "\\n")?,
                        b'\r' => write!(f, "\\r")?,
                        b'\t' => write!(f, "\\t")?,
                        b'\\' => write!(f, "\\\\")?,
                        b'"' => write!(f, "\\\"")?,
                        0x20..=0x7E => write!(f, "{}", *byte as char)?,
                        _ => write!(f, "\\x{byte:02x}")?,
                    }
                }
                write!(f, "\"")
            }
            Value::List(l) => {
                let ptr = Arc::as_ptr(l) as usize;
                if visited.contains(&ptr) {
                    return write!(f, "[...]");
                }
                visited.insert(ptr);

                write!(f, "[")?;
                let list = l.read();
                for (i, v) in list.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    v.fmt_helper(f, visited)?;
                }
                write!(f, "]")?;

                visited.remove(&ptr);
                Ok(())
            }
            Value::Tuple(t) => {
                write!(f, "(")?;
                for (i, v) in t.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    v.fmt_helper(f, visited)?;
                }
                if t.len() == 1 {
                    write!(f, ",")?;
                }
                write!(f, ")")
            }
            Value::Dictionary(d) => {
                let ptr = Arc::as_ptr(d) as usize;
                if visited.contains(&ptr) {
                    return write!(f, "{{...}}");
                }
                visited.insert(ptr);

                write!(f, "{{")?;
                let dict = d.read();
                for (i, (k, v)) in dict.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    k.fmt_helper(f, visited)?;
                    write!(f, ": ")?;
                    v.fmt_helper(f, visited)?;
                }
                write!(f, "}}")?;

                visited.remove(&ptr);
                Ok(())
            }
            Value::Set(s) => {
                let ptr = Arc::as_ptr(s) as usize;
                if visited.contains(&ptr) {
                    // Similar to python set(...)
                    // But if we want consistent {...} style:
                    return write!(f, "{{...}}");
                }
                visited.insert(ptr);

                let set = s.read();
                if set.is_empty() {
                    write!(f, "set()")?;
                } else {
                    write!(f, "{{")?;
                    for (i, v) in set.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        v.fmt_helper(f, visited)?;
                    }
                    write!(f, "}}")?;
                }

                visited.remove(&ptr);
                Ok(())
            }
            Value::Function(func) => write!(f, "<function {}>", func.name),
            Value::NativeFunction(name, _) => write!(f, "<native function {name}>"),
            Value::NativeFunctionWithKwargs(name, _) => write!(f, "<native function {name}>"),
            Value::BoundMethod(_, name) => write!(f, "<bound method {name}>"),
            Value::Foreign(obj) => write!(f, "<{}>", obj.type_name()),
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
            Value::String(s) => write!(f, "{s}"),    // Strings print without quotes in str()
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
    Error(String),
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
    Error(String),
}
