// AST: defines the abstract syntax tree nodes

#![allow(dead_code)]

/// A complete elang program: an ordered list of top-level statements.
pub type Program = Vec<Statement>;

/// Visibility modifier for class members (pub / pri).
#[derive(Debug, Clone)]
pub enum Visibility {
    Pub,
    Pri,
    Default,
}

/// Every executable piece of elang code is a Statement.
#[derive(Debug, Clone)]
pub enum Statement {
    LetDecl { name: String, value: Expr, line: usize },
    ConstDecl { name: String, value: Expr, line: usize },
    VarDecl { name: String, value: Expr, line: usize },
    Assign { name: String, value: Expr, line: usize },
    FnDef { name: String, params: Vec<String>, body: Vec<Statement>, is_async: bool, is_pure: bool, visibility: Visibility, line: usize },
    ClassDef { name: String, parent: Option<String>, body: Vec<Statement>, line: usize },
    Return { value: Expr, line: usize },
    If { condition: Expr, then_block: Vec<Statement>, else_block: Option<Vec<Statement>>, line: usize },
    Loop { kind: LoopKind, body: Vec<Statement>, line: usize },
    ForIn { var: String, iterable: Expr, body: Vec<Statement>, line: usize },
    Match { value: Expr, arms: Vec<MatchArm>, line: usize },
    Try { body: Vec<Statement>, catches: Vec<CatchClause>, line: usize },
    Import { module: String, line: usize },
    Export { stmt: Box<Statement>, line: usize },
    Break { line: usize },
    Continue { line: usize },
    ExprStmt { expr: Expr, line: usize },
    Print { value: Expr, line: usize },
    Field { name: String, value: Expr, visibility: Visibility, line: usize },
    FieldAssign { object: String, field: String, value: Expr, line: usize },
}

/// Any expression that evaluates to a value.
#[derive(Debug, Clone)]
pub enum Expr {
    Int { value: i64, line: usize },
    Float { value: f64, line: usize },
    Str { value: String, line: usize },
    Bool { value: bool, line: usize },
    Nothing { line: usize },
    Ident { name: String, line: usize },
    StrInterp { value: String, line: usize },
    BinOp { left: Box<Expr>, op: BinOpKind, right: Box<Expr>, line: usize },
    UnaryOp { op: UnaryOpKind, expr: Box<Expr>, line: usize },
    Call { callee: Box<Expr>, args: Vec<Expr>, line: usize },
    Index { object: Box<Expr>, index: Box<Expr>, line: usize },
    Field { object: Box<Expr>, field: String, line: usize },
    List { items: Vec<Expr>, line: usize },
    Map { pairs: Vec<(String, Expr)>, line: usize },
    Lambda { params: Vec<String>, body: Box<Expr>, line: usize },
    Pipe { left: Box<Expr>, right: Box<Expr>, line: usize },
    Await { expr: Box<Expr>, line: usize },
}

/// Binary arithmetic and comparison operators.
#[derive(Debug, Clone)]
pub enum BinOpKind {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
}

/// Unary prefix operators.
#[derive(Debug, Clone)]
pub enum UnaryOpKind {
    Neg,
    Not,
}

/// The kind of a loop statement.
#[derive(Debug, Clone)]
pub enum LoopKind {
    RepeatN(Expr),
    RepeatRange { var: String, from: Expr, to: Expr },
    While(Expr),
    Forever,
}

/// A single arm inside a match expression.
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: MatchPattern,
    pub body: Vec<Statement>,
}

/// Patterns that can appear on the left side of a match arm.
#[derive(Debug, Clone)]
pub enum MatchPattern {
    Literal(Expr),
    Wildcard,
    IsType(String),
}

/// A catch clause attached to a try statement.
#[derive(Debug, Clone)]
pub struct CatchClause {
    pub error_type: Option<String>,
    pub var: String,
    pub body: Vec<Statement>,
}
