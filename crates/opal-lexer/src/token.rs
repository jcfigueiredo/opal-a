use logos::{FilterResult, Logos};

/// Callback to lex multiline comments: `### ... ###`
fn lex_multiline_comment(lex: &mut logos::Lexer<Token>) -> FilterResult<(), ()> {
    let remainder = lex.remainder();
    match remainder.find("###") {
        Some(end) => {
            lex.bump(end + 3); // consume up to and including closing ###
            FilterResult::Skip
        }
        None => FilterResult::Error(()),
    }
}

/// Callback to lex triple-double-quoted strings: `""" ... """`
fn lex_triple_double_string(lex: &mut logos::Lexer<Token>) -> Result<(), ()> {
    let remainder = lex.remainder();
    match remainder.find(r#"""""#) {
        Some(end) => {
            lex.bump(end + 3);
            Ok(())
        }
        None => Err(()),
    }
}

/// Callback to lex triple-single-quoted strings: `''' ... '''`
fn lex_triple_single_string(lex: &mut logos::Lexer<Token>) -> Result<(), ()> {
    let remainder = lex.remainder();
    match remainder.find("'''") {
        Some(end) => {
            lex.bump(end + 3);
            Ok(())
        }
        None => Err(()),
    }
}

/// Callback to lex f-strings with double quotes: `f"... {expr} ..."`
/// Tracks brace depth so quotes inside interpolations don't end the string.
fn lex_fstring_double(lex: &mut logos::Lexer<Token>) -> Result<(), ()> {
    let remainder = lex.remainder();
    let bytes = remainder.as_bytes();
    let mut i = 0;
    let mut depth = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' if depth == 0 => {
                i += 2; // skip escaped char
            }
            b'"' if depth == 0 => {
                lex.bump(i + 1);
                return Ok(());
            }
            b'{' => {
                depth += 1;
                i += 1;
            }
            b'}' => {
                depth -= 1;
                i += 1;
            }
            b'"' if depth > 0 => {
                // quote inside interpolation — skip string literal
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
                i += 1; // skip closing quote
            }
            b'\'' if depth > 0 => {
                // single-quoted string inside interpolation
                i += 1;
                while i < bytes.len() && bytes[i] != b'\'' {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    Err(())
}

/// Callback to lex f-strings with single quotes: `f'... {expr} ...'`
fn lex_fstring_single(lex: &mut logos::Lexer<Token>) -> Result<(), ()> {
    let remainder = lex.remainder();
    let bytes = remainder.as_bytes();
    let mut i = 0;
    let mut depth = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' if depth == 0 => {
                i += 2;
            }
            b'\'' if depth == 0 => {
                lex.bump(i + 1);
                return Ok(());
            }
            b'{' => {
                depth += 1;
                i += 1;
            }
            b'}' => {
                depth -= 1;
                i += 1;
            }
            b'\'' if depth > 0 => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'\'' {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
                i += 1;
            }
            b'"' if depth > 0 => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    Err(())
}

/// Byte offset span in source code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// A token with its span
#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r]+")]
pub enum Token {
    // === Newlines (significant in Opal) ===
    #[regex(r"\n")]
    Newline,

    // === Comments ===
    #[regex(r"#[^\n]*")]
    Comment,

    #[token("###", lex_multiline_comment)]
    #[allow(dead_code)]
    MultilineComment,

    // === Keywords ===
    #[token("let")]
    Let,
    #[token("def")]
    Def,
    #[token("end")]
    End,
    #[token("if")]
    If,
    #[token("elsif")]
    Elsif,
    #[token("else")]
    Else,
    #[token("then")]
    Then,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("in")]
    In,
    #[token("do")]
    Do,
    #[token("match")]
    Match,
    #[token("case")]
    Case,
    #[token("class")]
    Class,
    #[token("module")]
    Module,
    #[token("import")]
    Import,
    #[token("from")]
    From,
    #[token("export")]
    Export,
    #[token("as")]
    As,
    #[token("return")]
    Return,
    #[token("try")]
    Try,
    #[token("catch")]
    Catch,
    #[token("ensure")]
    Ensure,
    #[token("raise")]
    Raise,
    #[token("actor")]
    Actor,
    #[token("receive")]
    Receive,
    #[token("supervisor")]
    Supervisor,
    #[token("parallel")]
    Parallel,
    #[token("async")]
    Async,
    #[token("await")]
    Await,
    #[token("needs")]
    Needs,
    // event, emit, on — reserved for macro-based self-hosting, not parser keywords
    #[token("macro")]
    Macro,
    #[token("ast")]
    Ast,
    #[token("type")]
    Type,
    #[token("enum")]
    Enum,
    #[token("model")]
    Model,
    #[token("settings")]
    Settings,
    #[token("protocol")]
    Protocol,
    #[token("implements")]
    Implements,
    #[token("private")]
    Private,
    #[token("public")]
    Public,
    #[token("requires")]
    Requires,
    #[token("extern")]
    Extern,
    #[token("with")]
    With,
    #[token("where")]
    Where,
    #[token("defaults")]
    Defaults,
    #[token("self")]
    SelfKw,
    #[token("receives")]
    Receives,
    #[token("reply")]
    Reply,
    #[token("send")]
    Send,
    #[token("break")]
    Break,
    #[token("next")]
    Next,

    // === Boolean & null literals ===
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("null")]
    Null,

    // === Logical operators (keywords) ===
    #[token("and")]
    And,
    #[token("or")]
    Or,
    #[token("not")]
    Not,
    #[token("is")]
    Is,

    // === Number literals ===
    #[regex(r"[0-9][0-9_]*\.[0-9][0-9_]*", |lex| lex.slice().replace('_', "").parse::<f64>().ok())]
    Float(f64),

    #[regex(r"[0-9][0-9_]*", priority = 2, callback = |lex| lex.slice().replace('_', "").parse::<i64>().ok())]
    Integer(i64),

    // === String literals ===
    // Triple-quoted strings (use callbacks since logos doesn't support non-greedy)
    #[token(r#"""""#, lex_triple_double_string)]
    TripleDoubleString,

    #[token("'''", lex_triple_single_string)]
    TripleSingleString,

    // Regular strings
    #[regex(r#""([^"\\]|\\.)*""#)]
    DoubleString,

    #[regex(r"'([^'\\]|\\.)*'")]
    SingleString,

    // === F-strings ===
    // Uses callbacks to track brace depth, allowing quotes inside {expr}.
    #[token(r#"f""#, lex_fstring_double)]
    FString,

    #[token("f'", lex_fstring_single)]
    FSingleString,

    // === R-strings ===
    #[regex(r#"r"[^"]*""#)]
    RString,

    #[regex(r"r'[^']*'")]
    RSingleString,

    // === T-strings ===
    #[regex(r#"t"([^"\\]|\\.)*""#)]
    TString,

    #[regex(r"t'([^'\\]|\\.)*'")]
    TSingleString,

    // === Symbols ===
    #[regex(r":[a-zA-Z_][a-zA-Z0-9_]*")]
    Symbol,

    // === Identifiers (must come after keywords) ===
    // Note: `?` is not included in identifier suffix to avoid conflicting with
    // `?.` (null-safe access) and `??` (null coalesce) operators.
    // `!` suffix is still supported for methods like `save!`.
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*!?")]
    Identifier,

    // === Operators ===
    #[token("**")]
    DoubleStar,
    #[token("+=")]
    PlusEq,
    #[token("-=")]
    MinusEq,
    #[token("*=")]
    StarEq,
    #[token("/=")]
    SlashEq,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("==")]
    EqEq,
    #[token("!=")]
    BangEq,
    #[token("<=")]
    LtEq,
    #[token(">=")]
    GtEq,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("|>")]
    Pipe,
    #[token("->")]
    Arrow,
    #[token("...")]
    DotDotDot,
    #[token("..")]
    DotDot,
    #[token("?.")]
    QuestionDot,
    #[token("??")]
    QuestionQuestion,

    // === Delimiters ===
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token(".")]
    Dot,
    #[token("=")]
    Eq,
    #[token("!")]
    Bang,
    #[token("|")]
    Bar,
    #[token("@")]
    At,
    #[token("$")]
    Dollar,

    // === Annotations ===
    #[token("@[")]
    AtBracket,
}
