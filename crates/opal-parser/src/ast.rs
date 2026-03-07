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
    /// Compound assignment: `name += expr`, `name -= expr`, etc.
    CompoundAssign { name: String, op: BinOp, value: Expr },
    /// Let binding: `let name = expr`
    Let { name: String, value: Expr },
    /// Function definition: `def name(params) -> ReturnType ... end`
    FuncDef {
        name: String,
        params: Vec<Param>,
        return_type: Option<String>,
        body: Vec<Stmt>,
        visibility: Option<String>,
        is_static: bool,
    },
    /// Return statement: `return expr`
    Return(Option<Expr>),
    /// For loop: `for x in expr ... end` or `for [a, b] in expr ... end`
    For {
        var: String,
        pattern: Option<Pattern>,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    /// While loop: `while expr ... end`
    While { condition: Expr, body: Vec<Stmt> },
    /// Class definition
    ClassDef {
        name: String,
        parent: Option<String>,
        needs: Vec<NeedsDecl>,
        methods: Vec<Stmt>,
        implements: Vec<String>,
    },
    /// Protocol definition
    ProtocolDef {
        name: String,
        methods: Vec<ProtocolMethod>,
    },
    /// Module definition
    ModuleDef { name: String, needs: Vec<NeedsDecl>, body: Vec<Stmt> },
    // Deprecated: use Import instead
    /// Import: `from X import Y, Z`
    FromImport {
        module_path: String,
        names: Vec<String>,
    },
    /// Import statement: `import Math.{abs, max}`
    Import(ImportStmt),
    /// Export block: `export {name1, name2}`
    ExportBlock(Vec<String>),
    /// Needs declaration (inside class)
    NeedsDecl(NeedsDecl),
    /// Requires precondition: `requires expr, "message"`
    Requires {
        condition: Expr,
        message: Option<Expr>,
    },
    /// Raise an error: `raise expr`
    Raise(Expr),
    /// Actor definition
    ActorDef {
        name: String,
        needs: Vec<NeedsDecl>,
        init: Option<Vec<Stmt>>,
        receive_cases: Vec<MatchCase>,
        methods: Vec<Stmt>,
    },
    /// Reply from actor receive handler
    Reply(Expr),
    /// Instance variable assignment: `.field = expr`
    InstanceAssign { field: String, value: Expr },
    /// Macro definition: `macro name(params) ... end`
    MacroDef {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    /// Macro invocation as statement: `@name expr NEWLINE block end`
    MacroInvoke {
        name: String,
        args: Vec<Expr>,
        block: Option<Vec<Stmt>>,
    },
    /// Annotated statement: @[...] followed by a def/class/etc
    Annotated {
        annotations: Vec<Annotation>,
        statement: Box<Stmt>,
    },
    /// Index assignment: `expr[expr] = expr`
    IndexAssign {
        object: Expr,
        index: Expr,
        value: Expr,
    },
    /// Type alias: `type Name = TypeExpr`
    TypeAlias { name: String, definition: TypeExpr },
    /// Enum definition
    EnumDef {
        name: String,
        variants: Vec<EnumVariantDef>,
        methods: Vec<Stmt>,
        implements: Vec<String>,
    },
    /// Extern FFI declaration: `extern "lib" ... end`
    ExternDef {
        lib_name: String,
        declarations: Vec<ExternDecl>,
    },
    /// Break out of a loop
    Break,
    /// Skip to next iteration of a loop
    Next,
    /// Parallel assignment: `a, b = 1, 2`
    ParallelAssign {
        names: Vec<String>,
        values: Vec<Expr>,
    },
    /// Destructure assignment: `[a, b] = list` or `[head | tail] = list`
    DestructureAssign { pattern: Pattern, value: Expr },
    /// Model definition (immutable validated data class)
    ModelDef {
        name: String,
        needs: Vec<ModelNeedsDecl>,
        methods: Vec<Stmt>,
    },
    /// Retroactive conformance: `implements Protocol for Type ... end`
    RetroactiveImpl {
        protocol_name: String,
        type_name: String,
        methods: Vec<Stmt>,
    },
    /// Event definition: `event Name(field: Type, ...)`
    EventDef {
        name: String,
        fields: Vec<NeedsDecl>,
    },
    /// Emit statement: `emit expr`
    Emit(Expr),
    /// On handler: `on EventType do |param| ... end`
    OnHandler {
        event_name: String,
        param: String,
        body: Vec<Stmt>,
    },
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
    /// Symbol literal: `:name`
    Symbol(String),
    /// Await expression: `await expr`
    Await(Box<Expr>),
    /// AST quasi-quote block: `ast ... end` — produces AST as a value
    AstBlock(Vec<Stmt>),
    /// Splice: `$var` inside ast block — substitutes an AST value
    Splice(String),
    /// Match expression
    Match {
        subject: Box<Expr>,
        cases: Vec<MatchCase>,
    },
    /// Try/catch expression: `try ... catch as e ... end`
    TryCatch {
        body: Vec<Stmt>,
        catches: Vec<CatchClause>,
        ensure: Option<Vec<Stmt>>,
    },
    /// Dict literal: `{key: value, ...}` or `{:}` for empty
    Dict(Vec<(Expr, Expr)>),
    /// Range: `start..end` (exclusive) or `start...end` (inclusive)
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
    },
    /// Index access: `expr[expr]`
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    /// Null-safe member access: `obj?.field`
    NullSafeMemberAccess {
        object: Box<Expr>,
        field: String,
    },
    /// List comprehension: [expr for var in iter] or [expr for var in iter if cond]
    ListComprehension {
        expr: Box<Expr>,
        var: String,
        iterable: Box<Expr>,
        condition: Option<Box<Expr>>,
    },
    /// Type cast: `expr as Type`
    Cast {
        expr: Box<Expr>,
        type_name: String,
    },
    /// super() call to parent method
    Super(Vec<Expr>),
}

/// A case in a match expression
#[derive(Debug, Clone)]
pub struct MatchCase {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Vec<Stmt>,
}

/// A pattern for matching
#[derive(Debug, Clone)]
pub enum Pattern {
    /// Binds a variable: `x`
    Identifier(String),
    /// Matches a literal value
    Literal(Expr),
    /// Constructor pattern: `Ok(x)`, `Error(msg)`, `Some(v)`
    Constructor(String, Vec<Pattern>),
    /// List pattern: `[a, b, c]` or `[head | tail]`
    List(Vec<Pattern>, Option<Box<Pattern>>),
    /// Wildcard: `_`
    Wildcard,
    /// Enum variant pattern: `Shape.Circle(r)`
    EnumVariant(String, String, Vec<Pattern>),
    /// Or-pattern: matches if any sub-pattern matches
    Or(Vec<Pattern>),
    /// Range pattern: matches integers within range
    Range { start: i64, end: i64, inclusive: bool },
    /// As-binding: destructure AND bind whole value
    As(Box<Pattern>, String),
}

/// A catch clause in try/catch
#[derive(Debug, Clone)]
pub struct CatchClause {
    pub var_name: String,           // variable name is required
    pub error_type: Option<String>, // optional type filter: catch e as ErrorType
    pub body: Vec<Stmt>,
}

/// A method in a protocol: required (no body) or default (has body)
#[derive(Debug, Clone)]
pub struct ProtocolMethod {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<String>,
    pub body: Option<Vec<Stmt>>,
}

/// A function declaration inside an extern block
#[derive(Debug, Clone)]
pub struct ExternDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<String>,
}

/// Import statement (Gleam-style)
#[derive(Debug, Clone)]
pub struct ImportStmt {
    pub path: Vec<String>,
    pub kind: ImportKind,
}

#[derive(Debug, Clone)]
pub enum ImportKind {
    Module,
    ModuleAlias(String),
    Selective(Vec<ImportItem>),
}

#[derive(Debug, Clone)]
pub struct ImportItem {
    pub name: String,
    pub alias: Option<String>,
}

/// A `needs` declaration in a class
#[derive(Debug, Clone)]
pub struct NeedsDecl {
    pub name: String,
    pub type_annotation: Option<String>,
    pub default: Option<Expr>,
}

/// A `needs` declaration in a model (with optional where validator)
#[derive(Debug, Clone)]
pub struct ModelNeedsDecl {
    pub name: String,
    pub type_annotation: Option<String>,
    pub validator: Option<Expr>,  // closure expression from `where |v| ...`
}

#[derive(Debug, Clone)]
pub enum FStringPart {
    Literal(String),
    Expr(Expr),
    /// Expression with format specifier: `{expr:.2}`, `{expr:>10}`, `{expr:<10}`
    FormattedExpr { expr: Expr, spec: String },
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

/// An annotation entry: key or key: value
#[derive(Debug, Clone)]
pub struct AnnotationEntry {
    pub key: String,
    pub value: Option<Expr>,
}

/// An annotation: @[entries]
#[derive(Debug, Clone)]
pub struct Annotation {
    pub entries: Vec<AnnotationEntry>,
}

/// An enum variant definition
#[derive(Debug, Clone)]
pub struct EnumVariantDef {
    pub name: String,
    pub fields: Vec<NeedsDecl>,
}

/// A type expression used in type aliases
#[derive(Debug, Clone)]
pub enum TypeExpr {
    Named(String),
    Union(Vec<TypeExpr>),
    SymbolSet(Vec<String>),
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
    Is,
    IsNot,
    In,
    NotIn,
    NullCoalesce,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
}
