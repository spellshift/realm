use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Def {
        name: String,
        parameters: Vec<Parameter>,
        body: Suite,
    },
    If {
        condition: Expression,
        consequence: Suite,
        alternatives: Vec<(Expression, Suite)>,
        default: Option<Suite>,
    },
    For {
        loop_vars: Vec<Expression>,
        iterable: Expression,
        body: Suite,
    },
    Simple(Vec<SmallStmt>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Parameter {
    Identifier(String),
    IdentifierWithValue(String, Expression),
    Star,
    StarIdentifier(String),
    StarStarIdentifier(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Suite {
    pub statements: Vec<Statement>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SmallStmt {
    Return(Option<Vec<Expression>>),
    Break,
    Continue,
    Pass,
    Assign {
        lhs: Vec<Expression>,
        op: AssignOp,
        rhs: Vec<Expression>,
    },
    Expr(Vec<Expression>),
    Load {
        module: String,
        symbols: Vec<(String, Option<String>)>,
    },
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AssignOp {
    Assign,
    Add,
    Subtract,
    Multiply,
    Divide,
    FloorDivide,
    Modulo,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LeftShift,
    RightShift,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    If {
        condition: Box<Expression>,
        consequence: Box<Expression>,
        alternative: Box<Expression>,
    },
    Primary(PrimaryExpr),
    Unary {
        op: UnaryOp,
        expr: Box<Expression>,
    },
    Binary {
        left: Box<Expression>,
        op: BinaryOp,
        right: Box<Expression>,
    },
    Lambda {
        parameters: Vec<Parameter>,
        body: Box<Expression>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum PrimaryExpr {
    Operand(Operand),
    Dot {
        expr: Box<PrimaryExpr>,
        ident: String,
    },
    Call {
        expr: Box<PrimaryExpr>,
        arguments: Vec<Argument>,
    },
    Subscript {
        expr: Box<PrimaryExpr>,
        subscript: Subscript,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Subscript {
    Index(Box<Expression>),
    Slice(
        Option<Box<Expression>>,
        Option<Box<Expression>>,
        Option<Box<Expression>>,
    ),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operand {
    Identifier(String),
    Int(i64),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    List(Vec<Expression>),
    ListComp {
        element: Box<Expression>,
        clauses: Vec<CompClause>,
    },
    Dict(Vec<(Expression, Expression)>),
    DictComp {
        key: Box<Expression>,
        value: Box<Expression>,
        clauses: Vec<CompClause>,
    },
    Tuple(Vec<Expression>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Argument {
    Positional(Expression),
    Named(String, Expression),
    Star(Expression),
    StarStar(Expression),
}

#[derive(Debug, PartialEq, Clone)]
pub enum CompClause {
    For {
        loop_vars: Vec<Expression>,
        iterable: Expression,
    },
    If(Expression),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum UnaryOp {
    Add,
    Subtract,
    Invert,
    Not,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BinaryOp {
    Or,
    And,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    In,
    NotIn,
    BitwiseOr,
    BitwiseXor,
    BitwiseAnd,
    LeftShift,
    RightShift,
    Add,
    Subtract,
    Multiply,
    Modulo,
    Divide,
    FloorDivide,
}
