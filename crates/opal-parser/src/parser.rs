use opal_lexer::{Span, SpannedToken, Token};
use thiserror::Error;

use crate::ast::*;

#[derive(Error, Debug, Clone)]
pub enum ParseError {
    #[error("unexpected token {found:?} at position {span:?}, expected {expected}")]
    UnexpectedToken {
        found: Token,
        expected: String,
        span: Span,
    },
    #[error("unexpected end of input, expected {expected}")]
    UnexpectedEof { expected: String },
    #[error("invalid f-string at position {span:?}: {message}")]
    InvalidFString { message: String, span: Span },
    #[error("lex error: {message}")]
    LexError { message: String, span: Span },
}

pub struct Parser<'src> {
    source: &'src str,
    tokens: Vec<SpannedToken>,
    pos: usize,
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str, tokens: Vec<SpannedToken>) -> Self {
        Self {
            source,
            tokens,
            pos: 0,
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut statements = Vec::new();
        self.skip_newlines();

        while !self.is_at_end() {
            statements.push(self.parse_statement()?);
            self.skip_newlines();
        }

        Ok(Program { statements })
    }

    // --- Statements ---

    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        let start = self.current_span();

        // Let binding
        if self.check(&Token::Let) {
            return self.parse_let_statement(start);
        }

        // Function definition
        if self.check(&Token::Def) {
            return self.parse_function_def();
        }

        // Return statement
        if self.check(&Token::Return) {
            return self.parse_return_statement(start);
        }

        // For loop
        if self.check(&Token::For) {
            return self.parse_for_loop(start);
        }

        // While loop
        if self.check(&Token::While) {
            return self.parse_while_loop(start);
        }

        // Class definition
        if self.check(&Token::Class) {
            return self.parse_class_def(start);
        }

        // Module definition
        if self.check(&Token::Module) {
            return self.parse_module_def(start);
        }

        // From import
        if self.check(&Token::From) {
            return self.parse_from_import(start);
        }

        // Requires precondition
        if self.check(&Token::Requires) {
            return self.parse_requires(start);
        }

        // Try/catch
        if self.check(&Token::Try) {
            return self.parse_try_catch(start);
        }

        // Raise
        if self.check(&Token::Raise) {
            return self.parse_raise(start);
        }

        // Needs declaration (inside class body)
        if self.check(&Token::Needs) {
            return self.parse_needs_decl(start);
        }

        // Try to parse an expression — could be expression statement or assignment
        let expr = self.parse_expression(0)?;

        if self.check(&Token::Eq) {
            // This is an assignment
            if let ExprKind::Identifier(name) = expr.kind {
                self.advance(); // consume =
                let value = self.parse_expression(0)?;
                self.expect_statement_end()?;
                let span = Span {
                    start: start.start,
                    end: value.span.end,
                };
                return Ok(Stmt {
                    kind: StmtKind::Assign { name, value },
                    span,
                });
            } else {
                return Err(ParseError::UnexpectedToken {
                    found: Token::Eq,
                    expected: "assignment target must be an identifier".into(),
                    span: self.current_span(),
                });
            }
        }

        // Expression statement
        self.expect_statement_end()?;
        let span = expr.span;
        Ok(Stmt {
            kind: StmtKind::Expr(expr),
            span,
        })
    }

    fn parse_let_statement(&mut self, start: Span) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'let'
        let name = self.expect_identifier()?;
        self.expect_token(&Token::Eq, "=")?;
        let value = self.parse_expression(0)?;
        self.expect_statement_end()?;
        let span = Span {
            start: start.start,
            end: value.span.end,
        };
        Ok(Stmt {
            kind: StmtKind::Let { name, value },
            span,
        })
    }

    fn parse_function_def(&mut self) -> Result<Stmt, ParseError> {
        let start = self.current_span();
        self.advance(); // consume 'def'

        let name = self.expect_identifier()?;
        self.expect_token(&Token::LParen, "(")?;
        let params = self.parse_params()?;
        self.expect_token(&Token::RParen, ")")?;

        // Optional return type: -> Type
        let return_type = if self.check(&Token::Arrow) {
            self.advance();
            Some(self.expect_identifier()?)
        } else {
            None
        };

        self.expect_newline()?;
        let body = self.parse_block()?;
        self.expect_token(&Token::End, "end")?;
        let end = self.previous_span().end;

        Ok(Stmt {
            kind: StmtKind::FuncDef {
                name,
                params,
                return_type,
                body,
            },
            span: Span {
                start: start.start,
                end,
            },
        })
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();

        if self.check(&Token::RParen) {
            return Ok(params);
        }

        loop {
            let name = self.expect_identifier()?;

            // Optional type annotation: : Type
            let type_annotation = if self.check(&Token::Colon) {
                self.advance();
                Some(self.expect_identifier()?)
            } else {
                None
            };

            // Optional default value: = expr
            let default = if self.check(&Token::Eq) {
                self.advance();
                Some(self.parse_expression(0)?)
            } else {
                None
            };

            params.push(Param {
                name,
                type_annotation,
                default,
            });

            if !self.check(&Token::Comma) {
                break;
            }
            self.advance(); // consume comma
        }

        Ok(params)
    }

    fn parse_return_statement(&mut self, start: Span) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'return'
        let value = if self.is_at_end() || self.check(&Token::Newline) || self.check(&Token::End) {
            None
        } else {
            Some(self.parse_expression(0)?)
        };
        self.expect_statement_end()?;
        let end = value.as_ref().map_or(start.end, |e| e.span.end);
        Ok(Stmt {
            kind: StmtKind::Return(value),
            span: Span {
                start: start.start,
                end,
            },
        })
    }

    fn parse_for_loop(&mut self, start: Span) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'for'
        let var = self.expect_identifier()?;
        self.expect_token(&Token::In, "in")?;
        let iterable = self.parse_expression(0)?;
        self.expect_newline()?;
        let body = self.parse_block()?;
        self.expect_token(&Token::End, "end")?;
        let end = self.previous_span().end;
        Ok(Stmt {
            kind: StmtKind::For {
                var,
                iterable,
                body,
            },
            span: Span {
                start: start.start,
                end,
            },
        })
    }

    fn parse_while_loop(&mut self, start: Span) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'while'
        let condition = self.parse_expression(0)?;
        self.expect_newline()?;
        let body = self.parse_block()?;
        self.expect_token(&Token::End, "end")?;
        let end = self.previous_span().end;
        Ok(Stmt {
            kind: StmtKind::While { condition, body },
            span: Span {
                start: start.start,
                end,
            },
        })
    }

    fn parse_class_def(&mut self, start: Span) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'class'
        let name = self.expect_identifier()?;
        self.expect_newline()?;
        self.skip_newlines();

        let mut needs = Vec::new();
        let mut methods = Vec::new();

        while !self.check(&Token::End) && !self.is_at_end() {
            if self.check(&Token::Needs) {
                let stmt = self.parse_needs_decl(self.current_span())?;
                if let StmtKind::NeedsDecl(decl) = stmt.kind {
                    needs.push(decl);
                }
            } else if self.check(&Token::Def) {
                methods.push(self.parse_function_def()?);
            } else {
                self.skip_newlines();
                if self.check(&Token::End) {
                    break;
                }
                return Err(ParseError::UnexpectedToken {
                    found: self.peek().unwrap().clone(),
                    expected: "needs, def, or end in class body".into(),
                    span: self.current_span(),
                });
            }
            self.skip_newlines();
        }

        self.expect_token(&Token::End, "end")?;
        let end = self.previous_span().end;
        Ok(Stmt {
            kind: StmtKind::ClassDef {
                name,
                needs,
                methods,
            },
            span: Span {
                start: start.start,
                end,
            },
        })
    }

    fn parse_needs_decl(&mut self, start: Span) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'needs'
        let name = self.expect_identifier()?;
        let type_annotation = if self.check(&Token::Colon) {
            self.advance();
            Some(self.expect_identifier()?)
        } else {
            None
        };
        self.expect_statement_end()?;
        let end = self.previous_span().end;
        Ok(Stmt {
            kind: StmtKind::NeedsDecl(NeedsDecl {
                name,
                type_annotation,
            }),
            span: Span {
                start: start.start,
                end,
            },
        })
    }

    fn parse_module_def(&mut self, start: Span) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'module'
        let name = self.expect_identifier()?;
        self.expect_newline()?;

        let mut body = Vec::new();
        self.skip_newlines();
        while !self.check(&Token::End) && !self.is_at_end() {
            body.push(self.parse_statement()?);
            self.skip_newlines();
        }
        self.expect_token(&Token::End, "end")?;
        let end = self.previous_span().end;
        Ok(Stmt {
            kind: StmtKind::ModuleDef { name, body },
            span: Span {
                start: start.start,
                end,
            },
        })
    }

    fn parse_from_import(&mut self, start: Span) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'from'
        let module_path = self.expect_identifier()?;
        self.expect_token(&Token::Import, "import")?;
        let mut names = Vec::new();
        loop {
            names.push(self.expect_identifier()?);
            if !self.check(&Token::Comma) {
                break;
            }
            self.advance();
        }
        self.expect_statement_end()?;
        let end = self.previous_span().end;
        Ok(Stmt {
            kind: StmtKind::FromImport { module_path, names },
            span: Span {
                start: start.start,
                end,
            },
        })
    }

    fn parse_requires(&mut self, start: Span) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'requires'
        let condition = self.parse_expression(0)?;
        let message = if self.check(&Token::Comma) {
            self.advance();
            Some(self.parse_expression(0)?)
        } else {
            None
        };
        self.expect_statement_end()?;
        let end = message
            .as_ref()
            .map_or(condition.span.end, |m| m.span.end);
        Ok(Stmt {
            kind: StmtKind::Requires { condition, message },
            span: Span {
                start: start.start,
                end,
            },
        })
    }

    fn parse_try_catch(&mut self, start: Span) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'try'
        self.expect_newline()?;
        let body = self.parse_block()?;

        let mut catches = Vec::new();
        while self.check(&Token::Catch) {
            self.advance();
            // Optional: catch Type as var
            let (error_type, var_name) = if self.check(&Token::Newline) || self.check(&Token::As) {
                let var = if self.check(&Token::As) {
                    self.advance();
                    Some(self.expect_identifier()?)
                } else {
                    None
                };
                (None, var)
            } else {
                let etype = self.expect_identifier()?;
                let var = if self.check(&Token::As) {
                    self.advance();
                    Some(self.expect_identifier()?)
                } else {
                    None
                };
                (Some(etype), var)
            };
            self.expect_newline()?;
            let catch_body = self.parse_block()?;
            catches.push(CatchClause {
                error_type,
                var_name,
                body: catch_body,
            });
        }

        let ensure = if self.check(&Token::Ensure) {
            self.advance();
            self.expect_newline()?;
            Some(self.parse_block()?)
        } else {
            None
        };

        self.expect_token(&Token::End, "end")?;
        let end = self.previous_span().end;
        Ok(Stmt {
            kind: StmtKind::TryCatch {
                body,
                catches,
                ensure,
            },
            span: Span {
                start: start.start,
                end,
            },
        })
    }

    fn parse_raise(&mut self, start: Span) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'raise'
        let expr = self.parse_expression(0)?;
        self.expect_statement_end()?;
        let end = expr.span.end;
        Ok(Stmt {
            kind: StmtKind::Raise(expr),
            span: Span {
                start: start.start,
                end,
            },
        })
    }

    fn parse_match_expression(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        self.advance(); // consume 'match'
        let subject = self.parse_expression(0)?;
        self.expect_newline()?;
        self.skip_newlines();

        let mut cases = Vec::new();
        while self.check(&Token::Case) {
            self.advance();
            let pattern = self.parse_pattern()?;
            self.expect_newline()?;
            let body = self.parse_block()?;
            cases.push(MatchCase { pattern, body });
            self.skip_newlines();
        }

        self.expect_token(&Token::End, "end")?;
        let end = self.previous_span().end;
        Ok(Expr {
            kind: ExprKind::Match {
                subject: Box::new(subject),
                cases,
            },
            span: Span {
                start: start.start,
                end,
            },
        })
    }

    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        // Wildcard: _
        if self.peek_is_identifier() {
            let name = self.extract_text(&self.current_span());
            if name == "_" {
                self.advance();
                return Ok(Pattern::Wildcard);
            }
        }

        // Constructor or identifier
        if self.peek_is_identifier() {
            let name = self.extract_text(&self.current_span());
            self.advance();

            // Constructor: Name(patterns)
            if self.check(&Token::LParen) {
                self.advance();
                let mut patterns = Vec::new();
                if !self.check(&Token::RParen) {
                    loop {
                        patterns.push(self.parse_pattern()?);
                        if !self.check(&Token::Comma) {
                            break;
                        }
                        self.advance();
                    }
                }
                self.expect_token(&Token::RParen, ")")?;
                return Ok(Pattern::Constructor(name, patterns));
            }

            // Plain identifier — could be a binding or a literal-like name
            return Ok(Pattern::Identifier(name));
        }

        // Literal patterns (integers, strings, bools, null)
        let expr = self.parse_primary()?;
        Ok(Pattern::Literal(expr))
    }

    // --- Expressions (Pratt parser) ---

    pub fn parse_expression(&mut self, min_precedence: u8) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;

        while let Some(op) = self.peek_binary_op() {
            let (prec, assoc) = op_precedence(op);
            if prec < min_precedence {
                break;
            }
            self.advance(); // consume operator
            let next_prec = if assoc == Assoc::Left { prec + 1 } else { prec };
            let right = self.parse_expression(next_prec)?;
            let span = Span {
                start: left.span.start,
                end: right.span.end,
            };
            left = Expr {
                kind: ExprKind::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
                span,
            };
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();

        if self.check(&Token::Minus) {
            self.advance();
            let operand = self.parse_unary()?;
            let span = Span {
                start: start.start,
                end: operand.span.end,
            };
            return Ok(Expr {
                kind: ExprKind::UnaryOp {
                    op: UnOp::Neg,
                    operand: Box::new(operand),
                },
                span,
            });
        }
        if self.check(&Token::Not) {
            self.advance();
            let operand = self.parse_unary()?;
            let span = Span {
                start: start.start,
                end: operand.span.end,
            };
            return Ok(Expr {
                kind: ExprKind::UnaryOp {
                    op: UnOp::Not,
                    operand: Box::new(operand),
                },
                span,
            });
        }

        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.check(&Token::LParen) {
                // Function call: expr(args)
                self.advance();
                let mut args = self.parse_args()?;
                self.expect_token(&Token::RParen, ")")?;

                // Trailing block: expr(args) do |params| ... end
                if self.check(&Token::Do) {
                    let closure = self.parse_block_closure()?;
                    args.push(Arg {
                        name: None,
                        value: closure,
                    });
                }

                let span = Span {
                    start: expr.span.start,
                    end: self.previous_span().end,
                };
                expr = Expr {
                    kind: ExprKind::Call {
                        function: Box::new(expr),
                        args,
                    },
                    span,
                };
            } else if self.check(&Token::Dot) {
                // Member access: expr.field
                self.advance();
                let field = self.expect_identifier()?;
                let span = Span {
                    start: expr.span.start,
                    end: self.previous_span().end,
                };
                expr = Expr {
                    kind: ExprKind::MemberAccess {
                        object: Box::new(expr),
                        field,
                    },
                    span,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let span = self.current_span();

        match self.peek() {
            Some(Token::Integer(n)) => {
                let n = *n;
                self.advance();
                Ok(Expr {
                    kind: ExprKind::Integer(n),
                    span,
                })
            }
            Some(Token::Float(n)) => {
                let n = *n;
                self.advance();
                Ok(Expr {
                    kind: ExprKind::Float(n),
                    span,
                })
            }
            Some(Token::DoubleString | Token::SingleString) => {
                let text = self.extract_string_content(&span);
                self.advance();
                Ok(Expr {
                    kind: ExprKind::String(text),
                    span,
                })
            }
            Some(Token::FString | Token::FSingleString) => self.parse_fstring(span),
            Some(Token::True) => {
                self.advance();
                Ok(Expr {
                    kind: ExprKind::Bool(true),
                    span,
                })
            }
            Some(Token::False) => {
                self.advance();
                Ok(Expr {
                    kind: ExprKind::Bool(false),
                    span,
                })
            }
            Some(Token::Null) => {
                self.advance();
                Ok(Expr {
                    kind: ExprKind::Null,
                    span,
                })
            }
            Some(Token::Identifier) => {
                let name = self.extract_text(&span);
                self.advance();
                Ok(Expr {
                    kind: ExprKind::Identifier(name),
                    span,
                })
            }
            Some(Token::If) => self.parse_if_expression(),
            Some(Token::Match) => self.parse_match_expression(),
            Some(Token::LParen) => {
                self.advance();
                let expr = self.parse_expression(0)?;
                self.expect_token(&Token::RParen, ")")?;
                let end = self.previous_span().end;
                Ok(Expr {
                    kind: ExprKind::Grouped(Box::new(expr)),
                    span: Span {
                        start: span.start,
                        end,
                    },
                })
            }
            // List literal: [expr, expr, ...]
            Some(Token::LBracket) => {
                self.advance();
                let mut elements = Vec::new();
                if !self.check(&Token::RBracket) {
                    loop {
                        elements.push(self.parse_expression(0)?);
                        if !self.check(&Token::Comma) {
                            break;
                        }
                        self.advance();
                    }
                }
                self.expect_token(&Token::RBracket, "]")?;
                let end = self.previous_span().end;
                Ok(Expr {
                    kind: ExprKind::List(elements),
                    span: Span {
                        start: span.start,
                        end,
                    },
                })
            }
            // Instance variable: .field (at start of expression, not after another expr)
            Some(Token::Dot) => {
                self.advance();
                let field = self.expect_identifier()?;
                let end = self.previous_span().end;
                Ok(Expr {
                    kind: ExprKind::InstanceVar(field),
                    span: Span {
                        start: span.start,
                        end,
                    },
                })
            }
            // Inline closure: |params| expr
            Some(Token::Bar) => self.parse_inline_closure(),
            // Block closure: do |params| ... end
            Some(Token::Do) => self.parse_block_closure(),
            Some(tok) => Err(ParseError::UnexpectedToken {
                found: tok.clone(),
                expected: "expression".into(),
                span,
            }),
            None => Err(ParseError::UnexpectedEof {
                expected: "expression".into(),
            }),
        }
    }

    // --- Specific parsers ---

    fn parse_if_expression(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        self.advance(); // consume 'if'

        let condition = self.parse_expression(0)?;

        // Opal supports both `then` (inline) and newline (block) after condition
        let then_branch = if self.check(&Token::Then) {
            self.advance();
            let expr = self.parse_expression(0)?;
            vec![Stmt {
                span: expr.span,
                kind: StmtKind::Expr(expr),
            }]
        } else {
            self.expect_newline()?;
            self.parse_block()?
        };

        // elsif branches
        let mut elsif_branches = Vec::new();
        while self.check(&Token::Elsif) {
            self.advance();
            let cond = self.parse_expression(0)?;
            let body = if self.check(&Token::Then) {
                self.advance();
                let expr = self.parse_expression(0)?;
                vec![Stmt {
                    span: expr.span,
                    kind: StmtKind::Expr(expr),
                }]
            } else {
                self.expect_newline()?;
                self.parse_block()?
            };
            elsif_branches.push((cond, body));
        }

        // else branch
        let else_branch = if self.check(&Token::Else) {
            self.advance();
            if !self.check(&Token::Newline) && !self.is_at_end() {
                let expr = self.parse_expression(0)?;
                Some(vec![Stmt {
                    span: expr.span,
                    kind: StmtKind::Expr(expr),
                }])
            } else {
                self.skip_newlines();
                Some(self.parse_block()?)
            }
        } else {
            None
        };

        self.expect_token(&Token::End, "end")?;
        let end = self.previous_span().end;

        Ok(Expr {
            kind: ExprKind::If {
                condition: Box::new(condition),
                then_branch,
                elsif_branches,
                else_branch,
            },
            span: Span {
                start: start.start,
                end,
            },
        })
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();
        self.skip_newlines();

        while !self.check(&Token::End)
            && !self.check(&Token::Else)
            && !self.check(&Token::Elsif)
            && !self.check(&Token::Case)
            && !self.check(&Token::Catch)
            && !self.check(&Token::Ensure)
            && !self.is_at_end()
        {
            stmts.push(self.parse_statement()?);
            self.skip_newlines();
        }

        Ok(stmts)
    }

    /// Parse inline closure: `|params| expr`
    fn parse_inline_closure(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        self.advance(); // consume '|'
        let mut params = Vec::new();
        if !self.check(&Token::Bar) {
            loop {
                params.push(self.expect_identifier()?);
                if !self.check(&Token::Comma) {
                    break;
                }
                self.advance();
            }
        }
        self.expect_token(&Token::Bar, "|")?;
        let body_expr = self.parse_expression(0)?;
        let end = body_expr.span.end;
        let body = vec![Stmt {
            span: body_expr.span,
            kind: StmtKind::Expr(body_expr),
        }];
        Ok(Expr {
            kind: ExprKind::Closure { params, body },
            span: Span {
                start: start.start,
                end,
            },
        })
    }

    /// Parse block closure: `do |params| ... end` or `do ... end`
    fn parse_block_closure(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        self.advance(); // consume 'do'

        // Optional params: |params|
        let params = if self.check(&Token::Bar) {
            self.advance();
            let mut p = Vec::new();
            if !self.check(&Token::Bar) {
                loop {
                    p.push(self.expect_identifier()?);
                    if !self.check(&Token::Comma) {
                        break;
                    }
                    self.advance();
                }
            }
            self.expect_token(&Token::Bar, "|")?;
            p
        } else {
            Vec::new()
        };

        self.skip_newlines();
        let body = self.parse_block()?;
        self.expect_token(&Token::End, "end")?;
        let end = self.previous_span().end;

        Ok(Expr {
            kind: ExprKind::Closure { params, body },
            span: Span {
                start: start.start,
                end,
            },
        })
    }

    fn parse_args(&mut self) -> Result<Vec<Arg>, ParseError> {
        let mut args = Vec::new();

        if self.check(&Token::RParen) {
            return Ok(args);
        }

        loop {
            // Check for named argument: `name: expr`
            if self.peek_is_identifier() && self.peek_ahead(1).is_some_and(|t| *t == Token::Colon) {
                let name = self.extract_text(&self.current_span());
                self.advance(); // identifier
                self.advance(); // colon
                let value = self.parse_expression(0)?;
                args.push(Arg {
                    name: Some(name),
                    value,
                });
            } else {
                let value = self.parse_expression(0)?;
                args.push(Arg { name: None, value });
            }

            if !self.check(&Token::Comma) {
                break;
            }
            self.advance(); // consume comma
        }

        Ok(args)
    }

    fn parse_fstring(&mut self, span: Span) -> Result<Expr, ParseError> {
        // The lexer gives us the entire f-string as one token.
        // We need to re-parse its contents to extract {expr} interpolations.
        let raw = self.extract_text(&span);
        self.advance();

        // Strip the f"..." or f'...' wrapper
        let quote_char = raw.chars().nth(1).unwrap_or('"');
        let inner = &raw[2..raw.len() - 1]; // skip f" and trailing "

        let mut parts = Vec::new();
        let mut current_literal = String::new();
        let mut chars = inner.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                // Start of interpolation
                if !current_literal.is_empty() {
                    parts.push(FStringPart::Literal(current_literal.clone()));
                    current_literal.clear();
                }
                // Collect the expression text until matching '}'
                let mut expr_text = String::new();
                let mut depth = 1;
                for c in chars.by_ref() {
                    if c == '{' {
                        depth += 1;
                    } else if c == '}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    expr_text.push(c);
                }
                if depth != 0 {
                    return Err(ParseError::InvalidFString {
                        message: "unmatched '{' in f-string".into(),
                        span,
                    });
                }
                // Parse the expression inside {}
                let inner_tokens =
                    opal_lexer::lex(&expr_text).map_err(|e| ParseError::InvalidFString {
                        message: format!("lex error in f-string expression: {}", e),
                        span,
                    })?;
                let mut inner_parser = Parser::new(&expr_text, inner_tokens);
                let expr = inner_parser.parse_expression(0)?;
                parts.push(FStringPart::Expr(expr));
            } else if ch == '\\' {
                // Escape sequence
                if let Some(next) = chars.next() {
                    match next {
                        'n' => current_literal.push('\n'),
                        't' => current_literal.push('\t'),
                        'r' => current_literal.push('\r'),
                        '\\' => current_literal.push('\\'),
                        c if c == quote_char => current_literal.push(c),
                        '{' => current_literal.push('{'),
                        '}' => current_literal.push('}'),
                        other => {
                            current_literal.push('\\');
                            current_literal.push(other);
                        }
                    }
                }
            } else {
                current_literal.push(ch);
            }
        }

        if !current_literal.is_empty() {
            parts.push(FStringPart::Literal(current_literal));
        }

        Ok(Expr {
            kind: ExprKind::FString(parts),
            span,
        })
    }

    // --- Token helpers ---

    fn extract_text(&self, span: &Span) -> String {
        self.source[span.start..span.end].to_string()
    }

    fn extract_string_content(&self, span: &Span) -> String {
        let raw = &self.source[span.start..span.end];
        // Strip quotes
        let inner = &raw[1..raw.len() - 1];
        // Process escape sequences
        let mut result = String::new();
        let mut chars = inner.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\\' {
                if let Some(next) = chars.next() {
                    match next {
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        'r' => result.push('\r'),
                        '\\' => result.push('\\'),
                        '"' => result.push('"'),
                        '\'' => result.push('\''),
                        '0' => result.push('\0'),
                        other => {
                            result.push('\\');
                            result.push(other);
                        }
                    }
                }
            } else {
                result.push(ch);
            }
        }
        result
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|t| &t.token)
    }

    fn peek_ahead(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.pos + offset).map(|t| &t.token)
    }

    fn peek_is_identifier(&self) -> bool {
        matches!(self.peek(), Some(Token::Identifier))
    }

    fn check(&self, expected: &Token) -> bool {
        self.peek()
            .is_some_and(|t| std::mem::discriminant(t) == std::mem::discriminant(expected))
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    fn current_span(&self) -> Span {
        self.tokens
            .get(self.pos)
            .map(|t| t.span)
            .unwrap_or(Span { start: 0, end: 0 })
    }

    fn previous_span(&self) -> Span {
        if self.pos > 0 {
            self.tokens[self.pos - 1].span
        } else {
            Span { start: 0, end: 0 }
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn expect_token(&mut self, expected: &Token, name: &str) -> Result<(), ParseError> {
        if self.check(expected) {
            self.advance();
            Ok(())
        } else {
            match self.peek() {
                Some(tok) => Err(ParseError::UnexpectedToken {
                    found: tok.clone(),
                    expected: name.to_string(),
                    span: self.current_span(),
                }),
                None => Err(ParseError::UnexpectedEof {
                    expected: name.to_string(),
                }),
            }
        }
    }

    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        if let Some(Token::Identifier) = self.peek() {
            let text = self.extract_text(&self.current_span());
            self.advance();
            Ok(text)
        } else {
            match self.peek() {
                Some(tok) => Err(ParseError::UnexpectedToken {
                    found: tok.clone(),
                    expected: "identifier".to_string(),
                    span: self.current_span(),
                }),
                None => Err(ParseError::UnexpectedEof {
                    expected: "identifier".to_string(),
                }),
            }
        }
    }

    fn expect_newline(&mut self) -> Result<(), ParseError> {
        self.expect_token(&Token::Newline, "newline")
    }

    fn expect_statement_end(&mut self) -> Result<(), ParseError> {
        if self.is_at_end()
            || self.check(&Token::Newline)
            || self.check(&Token::End)
            || self.check(&Token::Else)
            || self.check(&Token::Elsif)
            || self.check(&Token::Case)
            || self.check(&Token::Catch)
            || self.check(&Token::Ensure)
        {
            if self.check(&Token::Newline) {
                self.advance();
            }
            Ok(())
        } else {
            match self.peek() {
                Some(tok) => Err(ParseError::UnexpectedToken {
                    found: tok.clone(),
                    expected: "newline or end of input".to_string(),
                    span: self.current_span(),
                }),
                None => Ok(()),
            }
        }
    }

    fn skip_newlines(&mut self) {
        while self.check(&Token::Newline) {
            self.advance();
        }
    }

    fn peek_binary_op(&self) -> Option<BinOp> {
        match self.peek() {
            Some(Token::Plus) => Some(BinOp::Add),
            Some(Token::Minus) => Some(BinOp::Sub),
            Some(Token::Star) => Some(BinOp::Mul),
            Some(Token::Slash) => Some(BinOp::Div),
            Some(Token::Percent) => Some(BinOp::Mod),
            Some(Token::DoubleStar) => Some(BinOp::Pow),
            Some(Token::EqEq) => Some(BinOp::Eq),
            Some(Token::BangEq) => Some(BinOp::NotEq),
            Some(Token::Lt) => Some(BinOp::Lt),
            Some(Token::Gt) => Some(BinOp::Gt),
            Some(Token::LtEq) => Some(BinOp::LtEq),
            Some(Token::GtEq) => Some(BinOp::GtEq),
            Some(Token::And) => Some(BinOp::And),
            Some(Token::Or) => Some(BinOp::Or),
            Some(Token::Pipe) => Some(BinOp::Pipe),
            _ => None,
        }
    }
}

// --- Operator precedence ---

#[derive(PartialEq)]
enum Assoc {
    Left,
    Right,
}

fn op_precedence(op: BinOp) -> (u8, Assoc) {
    match op {
        BinOp::Or => (1, Assoc::Left),
        BinOp::And => (2, Assoc::Left),
        BinOp::Eq | BinOp::NotEq => (3, Assoc::Left),
        BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq => (4, Assoc::Left),
        BinOp::Pipe => (5, Assoc::Left),
        BinOp::Add | BinOp::Sub => (6, Assoc::Left),
        BinOp::Mul | BinOp::Div | BinOp::Mod => (7, Assoc::Left),
        BinOp::Pow => (8, Assoc::Right),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(source: &str) -> Program {
        let tokens = opal_lexer::lex(source).unwrap();
        let mut parser = Parser::new(source, tokens);
        parser.parse_program().unwrap()
    }

    #[test]
    fn parse_string_literal() {
        let prog = parse(r#"print("hello")"#);
        assert_eq!(prog.statements.len(), 1);
    }

    #[test]
    fn parse_assignment() {
        let prog = parse(r#"name = "Opal""#);
        assert_eq!(prog.statements.len(), 1);
        match &prog.statements[0].kind {
            StmtKind::Assign { name, .. } => assert_eq!(name, "name"),
            _ => panic!("expected assignment"),
        }
    }

    #[test]
    fn parse_let_binding() {
        let prog = parse("let x = 42");
        match &prog.statements[0].kind {
            StmtKind::Let { name, .. } => assert_eq!(name, "x"),
            _ => panic!("expected let binding"),
        }
    }

    #[test]
    fn parse_fstring() {
        let prog = parse(r#"print(f"Hello, {name}!")"#);
        assert_eq!(prog.statements.len(), 1);
    }

    #[test]
    fn parse_binary_ops_precedence() {
        let prog = parse("1 + 2 * 3");
        // Should parse as 1 + (2 * 3) due to precedence
        assert_eq!(prog.statements.len(), 1);
        match &prog.statements[0].kind {
            StmtKind::Expr(expr) => match &expr.kind {
                ExprKind::BinaryOp { op, .. } => assert_eq!(*op, BinOp::Add),
                _ => panic!("expected binary op"),
            },
            _ => panic!("expected expression"),
        }
    }

    #[test]
    fn parse_if_inline() {
        let prog = parse("if true then 1 else 2 end");
        assert_eq!(prog.statements.len(), 1);
        match &prog.statements[0].kind {
            StmtKind::Expr(expr) => {
                assert!(matches!(expr.kind, ExprKind::If { .. }));
            }
            _ => panic!("expected expression"),
        }
    }

    #[test]
    fn parse_function_call_with_args() {
        let prog = parse(r#"print("a", "b")"#);
        assert_eq!(prog.statements.len(), 1);
        match &prog.statements[0].kind {
            StmtKind::Expr(expr) => match &expr.kind {
                ExprKind::Call { args, .. } => assert_eq!(args.len(), 2),
                _ => panic!("expected call"),
            },
            _ => panic!("expected expression"),
        }
    }

    #[test]
    fn parse_multiple_statements() {
        let prog = parse("name = \"Opal\"\nprint(name)");
        assert_eq!(prog.statements.len(), 2);
    }

    #[test]
    fn parse_function_def() {
        let prog = parse("def add(a: Int, b: Int) -> Int\n  a + b\nend");
        assert_eq!(prog.statements.len(), 1);
        match &prog.statements[0].kind {
            StmtKind::FuncDef {
                name,
                params,
                return_type,
                ..
            } => {
                assert_eq!(name, "add");
                assert_eq!(params.len(), 2);
                assert_eq!(params[0].name, "a");
                assert_eq!(params[0].type_annotation.as_deref(), Some("Int"));
                assert_eq!(return_type.as_deref(), Some("Int"));
            }
            _ => panic!("expected function definition"),
        }
    }

    #[test]
    fn parse_function_no_types() {
        let prog = parse("def greet(name)\n  print(name)\nend");
        match &prog.statements[0].kind {
            StmtKind::FuncDef {
                params,
                return_type,
                ..
            } => {
                assert_eq!(params[0].type_annotation, None);
                assert_eq!(*return_type, None);
            }
            _ => panic!("expected function definition"),
        }
    }

    #[test]
    fn parse_return_statement() {
        let prog = parse("def foo()\n  return 42\nend");
        match &prog.statements[0].kind {
            StmtKind::FuncDef { body, .. } => {
                assert!(matches!(body[0].kind, StmtKind::Return(Some(_))));
            }
            _ => panic!("expected function definition"),
        }
    }

    #[test]
    fn parse_list_literal() {
        let prog = parse("[1, 2, 3]");
        match &prog.statements[0].kind {
            StmtKind::Expr(expr) => match &expr.kind {
                ExprKind::List(elements) => assert_eq!(elements.len(), 3),
                _ => panic!("expected list"),
            },
            _ => panic!("expected expression"),
        }
    }

    #[test]
    fn parse_empty_list() {
        let prog = parse("[]");
        match &prog.statements[0].kind {
            StmtKind::Expr(expr) => match &expr.kind {
                ExprKind::List(elements) => assert_eq!(elements.len(), 0),
                _ => panic!("expected list"),
            },
            _ => panic!("expected expression"),
        }
    }

    #[test]
    fn parse_inline_closure() {
        let prog = parse("numbers.filter(|n| n > 0)");
        assert_eq!(prog.statements.len(), 1);
    }

    #[test]
    fn parse_block_closure_trailing() {
        let prog = parse("list.reduce(0) do |acc, n|\n  acc + n\nend");
        assert_eq!(prog.statements.len(), 1);
    }

    #[test]
    fn parse_for_loop() {
        let prog = parse("for x in items\n  print(x)\nend");
        assert!(matches!(prog.statements[0].kind, StmtKind::For { .. }));
    }

    #[test]
    fn parse_while_loop() {
        let prog = parse("while x > 0\n  x = x - 1\nend");
        assert!(matches!(prog.statements[0].kind, StmtKind::While { .. }));
    }

    #[test]
    fn parse_class_def() {
        let prog =
            parse("class Circle\n  needs radius: Float\n\n  def area()\n    .radius\n  end\nend");
        match &prog.statements[0].kind {
            StmtKind::ClassDef {
                name,
                needs,
                methods,
            } => {
                assert_eq!(name, "Circle");
                assert_eq!(needs.len(), 1);
                assert_eq!(needs[0].name, "radius");
                assert_eq!(methods.len(), 1);
            }
            _ => panic!("expected class def"),
        }
    }

    #[test]
    fn parse_module_def() {
        let prog = parse("module Shapes\n  class Circle\n    needs radius: Float\n  end\nend");
        assert!(matches!(
            prog.statements[0].kind,
            StmtKind::ModuleDef { .. }
        ));
    }

    #[test]
    fn parse_from_import() {
        let prog = parse("from Shapes import Circle, Rectangle");
        match &prog.statements[0].kind {
            StmtKind::FromImport { module_path, names } => {
                assert_eq!(module_path, "Shapes");
                assert_eq!(names, &["Circle", "Rectangle"]);
            }
            _ => panic!("expected from import"),
        }
    }

    #[test]
    fn parse_instance_var() {
        let prog = parse("def area()\n  .radius * .radius\nend");
        match &prog.statements[0].kind {
            StmtKind::FuncDef { body, .. } => match &body[0].kind {
                StmtKind::Expr(expr) => match &expr.kind {
                    ExprKind::BinaryOp { left, .. } => {
                        assert!(matches!(left.kind, ExprKind::InstanceVar(_)));
                    }
                    _ => panic!("expected binary op"),
                },
                _ => panic!("expected expr"),
            },
            _ => panic!("expected func def"),
        }
    }

    #[test]
    fn parse_match_expression() {
        let prog = parse("match x\n  case Ok(v)\n    print(v)\n  case Error(e)\n    print(e)\nend");
        assert_eq!(prog.statements.len(), 1);
    }

    #[test]
    fn parse_requires() {
        let prog = parse("requires x > 0, \"must be positive\"");
        assert!(matches!(
            prog.statements[0].kind,
            StmtKind::Requires { .. }
        ));
    }

    #[test]
    fn parse_raise() {
        let prog = parse("raise \"error\"");
        assert!(matches!(prog.statements[0].kind, StmtKind::Raise(_)));
    }

    #[test]
    fn parse_try_catch() {
        let prog = parse("try\n  print(1)\ncatch as e\n  print(e)\nend");
        assert!(matches!(
            prog.statements[0].kind,
            StmtKind::TryCatch { .. }
        ));
    }
}
