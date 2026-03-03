use opal_lexer::Span;

/// A complete Opal program
#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

/// A statement
#[derive(Debug, Clone)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum StmtKind {
    /// An expression used as a statement
    Expr(Expr),
    /// Variable assignment: `name = expr`
    Assign { name: String, value: Expr },
    /// Let binding: `let name = expr`
    Let { name: String, value: Expr },
    /// Function definition: `def name(params) -> ReturnType ... end`
    FuncDef {
        name: String,
        params: Vec<Param>,
        return_type: Option<String>,
        body: Vec<Stmt>,
    },
    /// Return statement: `return expr`
    Return(Option<Expr>),
    /// For loop: `for x in expr ... end`
    For {
        var: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    /// While loop: `while expr ... end`
    While { condition: Expr, body: Vec<Stmt> },
    /// Class definition
    ClassDef {
        name: String,
        needs: Vec<NeedsDecl>,
        methods: Vec<Stmt>,
    },
    /// Module definition
    ModuleDef { name: String, body: Vec<Stmt> },
    /// Import: `from X import Y, Z`
    FromImport {
        module_path: String,
        names: Vec<String>,
    },
    /// Needs declaration (inside class)
    NeedsDecl(NeedsDecl),
}

/// An expression
#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    /// String literal: `"hello"` or `'hello'`
    String(String),
    /// F-string: `f"Hello, {name}!"`
    FString(Vec<FStringPart>),
    /// Integer literal: `42`
    Integer(i64),
    /// Float literal: `3.14`
    Float(f64),
    /// Boolean literal: `true` or `false`
    Bool(bool),
    /// Null literal
    Null,
    /// Variable reference: `name`
    Identifier(String),
    /// Function call: `print("hello")` or `name.method(args)`
    Call { function: Box<Expr>, args: Vec<Arg> },
    /// Binary operation: `a + b`
    BinaryOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    /// Unary operation: `-x`, `not x`
    UnaryOp { op: UnOp, operand: Box<Expr> },
    /// Member access: `obj.field`
    MemberAccess { object: Box<Expr>, field: String },
    /// If expression: `if cond then expr else expr end`
    If {
        condition: Box<Expr>,
        then_branch: Vec<Stmt>,
        elsif_branches: Vec<(Expr, Vec<Stmt>)>,
        else_branch: Option<Vec<Stmt>>,
    },
    /// Grouped expression: `(expr)`
    Grouped(Box<Expr>),
    /// List literal: `[1, 2, 3]`
    List(Vec<Expr>),
    /// Closure: `|params| expr` or `do |params| ... end`
    Closure {
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    /// Instance variable access: `.field`
    InstanceVar(String),
}

/// A `needs` declaration in a class
#[derive(Debug, Clone)]
pub struct NeedsDecl {
    pub name: String,
    pub type_annotation: Option<String>,
}

#[derive(Debug, Clone)]
pub enum FStringPart {
    Literal(String),
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub struct Arg {
    pub name: Option<String>,
    pub value: Expr,
}

/// A function parameter
#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub type_annotation: Option<String>,
    pub default: Option<Expr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
    Pipe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
}
