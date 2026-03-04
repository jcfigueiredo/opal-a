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

        // Protocol definition
        if self.check(&Token::Protocol) {
            return self.parse_protocol_def(start);
        }

        // Module definition
        if self.check(&Token::Module) {
            return self.parse_module_def(start);
        }

        // Import statement (new syntax)
        if self.check(&Token::Import) {
            return self.parse_import(start);
        }

        // Export block
        if self.check(&Token::Export) {
            return self.parse_export_block(start);
        }

        // From import
        if self.check(&Token::From) {
            return self.parse_from_import(start);
        }

        // Extern FFI block
        if self.check(&Token::Extern) {
            return self.parse_extern_def(start);
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

        // Macro definition
        if self.check(&Token::Macro) {
            return self.parse_macro_def(start);
        }

        // Annotation: @[key: val, ...]
        if self.check(&Token::AtBracket) {
            return self.parse_annotated(start);
        }

        // Macro invocation: @name
        if self.check(&Token::At) {
            return self.parse_macro_invoke(start);
        }

        // Type alias: type Name = ...
        if self.check(&Token::Type) {
            return self.parse_type_alias(start);
        }

        // Enum definition
        if self.check(&Token::Enum) {
            return self.parse_enum_def(start);
        }

        // Actor definition
        if self.check(&Token::Actor) {
            return self.parse_actor_def(start);
        }

        // Reply
        if self.check(&Token::Reply) {
            return self.parse_reply(start);
        }

        // Break
        if self.check(&Token::Break) {
            self.advance(); // consume 'break'
            self.expect_statement_end()?;
            return Ok(Stmt {
                kind: StmtKind::Break,
                span: start,
            });
        }

        // Next
        if self.check(&Token::Next) {
            self.advance(); // consume 'next'
            self.expect_statement_end()?;
            return Ok(Stmt {
                kind: StmtKind::Next,
                span: start,
            });
        }

        // Instance variable assignment: .field = expr
        if self.check(&Token::Dot)
            && self
                .peek_ahead(1)
                .is_some_and(|t| matches!(t, Token::Identifier))
            && self.peek_ahead(2).is_some_and(|t| matches!(t, Token::Eq))
        {
            return self.parse_instance_assign(start);
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

        // Optional return type: -> Type or -> Type[T, E]
        let return_type = if self.check(&Token::Arrow) {
            self.advance();
            let name = self.expect_identifier()?;
            // Skip generic params like [Float, String] — not enforced yet
            if self.check(&Token::LBracket) {
                self.advance();
                let mut depth = 1;
                while depth > 0 && !self.is_at_end() {
                    if self.check(&Token::LBracket) {
                        depth += 1;
                    } else if self.check(&Token::RBracket) {
                        depth -= 1;
                    }
                    self.advance();
                }
            }
            Some(name)
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

    fn parse_annotated(&mut self, start: Span) -> Result<Stmt, ParseError> {
        let mut annotations = Vec::new();
        while self.check(&Token::AtBracket) {
            self.advance(); // consume '@['
            let mut entries = Vec::new();
            if !self.check(&Token::RBracket) {
                loop {
                    let key = self.expect_identifier()?;
                    let value = if self.check(&Token::Colon) {
                        self.advance();
                        Some(self.parse_expression(0)?)
                    } else {
                        None
                    };
                    entries.push(AnnotationEntry { key, value });
                    if !self.check(&Token::Comma) {
                        break;
                    }
                    self.advance();
                }
            }
            self.expect_token(&Token::RBracket, "]")?;
            annotations.push(Annotation { entries });
            self.expect_newline()?;
            self.skip_newlines();
        }
        let statement = self.parse_statement()?;
        let end = statement.span.end;
        Ok(Stmt {
            kind: StmtKind::Annotated {
                annotations,
                statement: Box::new(statement),
            },
            span: Span { start: start.start, end },
        })
    }

    fn parse_enum_def(&mut self, start: Span) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'enum'
        let name = self.expect_identifier()?;

        // Parse optional: implements Protocol1, Protocol2
        let mut implements = Vec::new();
        if self.check(&Token::Implements) {
            self.advance();
            loop {
                implements.push(self.expect_identifier()?);
                if !self.check(&Token::Comma) {
                    break;
                }
                self.advance();
            }
        }

        self.expect_newline()?;
        self.skip_newlines();

        let mut variants = Vec::new();
        let mut methods = Vec::new();

        while !self.check(&Token::End) {
            // Method definition
            if self.check(&Token::Def) {
                methods.push(self.parse_function_def()?);
                self.skip_newlines();
                continue;
            }

            // Variant: Name or Name(field: Type, ...)
            if self.peek_is_identifier() {
                let variant_name = self.extract_text(&self.current_span());
                self.advance();

                let mut fields = Vec::new();
                if self.check(&Token::LParen) {
                    self.advance();
                    if !self.check(&Token::RParen) {
                        loop {
                            let field_name = self.expect_identifier()?;
                            let type_ann = if self.check(&Token::Colon) {
                                self.advance();
                                Some(self.expect_identifier()?)
                            } else {
                                None
                            };
                            fields.push(NeedsDecl {
                                name: field_name,
                                type_annotation: type_ann,
                            });
                            if !self.check(&Token::Comma) {
                                break;
                            }
                            self.advance();
                        }
                    }
                    self.expect_token(&Token::RParen, ")")?;
                }

                variants.push(EnumVariantDef {
                    name: variant_name,
                    fields,
                });
                self.expect_newline()?;
                self.skip_newlines();
                continue;
            }

            break;
        }

        let end = self.current_span().end;
        self.expect_token(&Token::End, "end")?;
        self.expect_newline()?;

        Ok(Stmt {
            kind: StmtKind::EnumDef {
                name,
                variants,
                methods,
                implements,
            },
            span: Span { start: start.start, end },
        })
    }

    fn parse_type_alias(&mut self, start: Span) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'type'
        let name = self.expect_identifier()?;
        self.expect_token(&Token::Eq, "=")?;
        let definition = self.parse_type_expr()?;
        let end = self.previous_span().end;
        self.expect_newline()?;
        Ok(Stmt {
            kind: StmtKind::TypeAlias { name, definition },
            span: Span { start: start.start, end },
        })
    }

    fn parse_type_expr(&mut self) -> Result<TypeExpr, ParseError> {
        // Parse the first element — either a symbol (:name) or a named type (Name)
        let first = self.parse_type_atom()?;

        // Check for union: ... | ...
        if self.check(&Token::Bar) {
            let mut parts = vec![first];
            while self.check(&Token::Bar) {
                self.advance(); // consume '|'
                parts.push(self.parse_type_atom()?);
            }
            // Check if all parts are symbols — if so, it's a SymbolSet
            let all_symbols: Vec<String> = parts.iter().filter_map(|p| {
                if let TypeExpr::SymbolSet(syms) = p {
                    if syms.len() == 1 { return Some(syms[0].clone()); }
                }
                None
            }).collect();
            if all_symbols.len() == parts.len() {
                return Ok(TypeExpr::SymbolSet(all_symbols));
            }
            // Otherwise it's a Union of types
            return Ok(TypeExpr::Union(parts));
        }

        Ok(first)
    }

    fn parse_type_atom(&mut self) -> Result<TypeExpr, ParseError> {
        // Symbol: :name
        if self.check(&Token::Symbol) {
            let text = self.extract_text(&self.current_span());
            let name = text[1..].to_string();
            self.advance();
            return Ok(TypeExpr::SymbolSet(vec![name]));
        }
        // Named type: Identifier
        if self.peek_is_identifier() {
            let name = self.extract_text(&self.current_span());
            self.advance();
            return Ok(TypeExpr::Named(name));
        }
        let span = self.current_span();
        Err(ParseError::UnexpectedToken {
            found: self.peek().cloned().unwrap_or(Token::Newline),
            expected: "type name or symbol".to_string(),
            span,
        })
    }

    fn parse_class_def(&mut self, start: Span) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'class'
        let name = self.expect_identifier()?;

        // Parse optional: implements Protocol1, Protocol2
        let mut implements = Vec::new();
        if self.check(&Token::Implements) {
            self.advance();
            loop {
                implements.push(self.expect_identifier()?);
                if !self.check(&Token::Comma) {
                    break;
                }
                self.advance();
            }
        }

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
                implements,
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

    fn parse_protocol_def(&mut self, start: Span) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'protocol'
        let name = self.expect_identifier()?;
        self.expect_newline()?;
        self.skip_newlines();

        let mut methods = Vec::new();

        while !self.check(&Token::End) && !self.is_at_end() {
            if self.check(&Token::Def) {
                self.advance(); // consume 'def'
                let method_name = self.expect_identifier()?;

                // Parse params
                self.expect_token(&Token::LParen, "(")?;
                let mut params = Vec::new();
                if !self.check(&Token::RParen) {
                    loop {
                        let param_name = self.expect_identifier()?;
                        let type_ann = if self.check(&Token::Colon) {
                            self.advance();
                            Some(self.expect_identifier()?)
                        } else {
                            None
                        };
                        params.push(Param {
                            name: param_name,
                            type_annotation: type_ann,
                            default: None,
                        });
                        if !self.check(&Token::Comma) {
                            break;
                        }
                        self.advance();
                    }
                }
                self.expect_token(&Token::RParen, ")")?;

                // Optional return type
                let return_type = if self.check(&Token::Arrow) {
                    self.advance();
                    Some(self.expect_identifier()?)
                } else {
                    None
                };

                // Check for body (default method) or newline (required method)
                let body = if self.check(&Token::Newline) || self.is_at_end() {
                    self.skip_newlines();
                    // Check if next line starts a body (not another def or end)
                    if !self.check(&Token::Def)
                        && !self.check(&Token::End)
                        && !self.is_at_end()
                    {
                        // Has a body — parse until we hit end/def
                        let body = self.parse_block()?;
                        self.expect_token(&Token::End, "end")?;
                        self.skip_newlines();
                        Some(body)
                    } else {
                        None // Required method (no body)
                    }
                } else {
                    None
                };

                methods.push(ProtocolMethod {
                    name: method_name,
                    params,
                    return_type,
                    body,
                });
            } else {
                self.skip_newlines();
                if self.check(&Token::End) {
                    break;
                }
                return Err(ParseError::UnexpectedToken {
                    found: self.peek().unwrap().clone(),
                    expected: "def or end in protocol body".into(),
                    span: self.current_span(),
                });
            }
        }

        self.expect_token(&Token::End, "end")?;
        let end = self.previous_span().end;
        Ok(Stmt {
            kind: StmtKind::ProtocolDef { name, methods },
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

    fn parse_import(&mut self, start: Span) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'import'
        let first = self.expect_identifier()?;
        let mut path = vec![first];

        // Parse dotted module path: Math.Vector.{abs, max}
        // Stop if we see `.{` (selective import)
        while self.check(&Token::Dot) {
            // Peek ahead: if after `.` we see `{`, stop path parsing
            if self
                .peek_ahead(1)
                .is_some_and(|t| matches!(t, Token::LBrace))
            {
                self.advance(); // consume '.'
                break;
            }
            self.advance(); // consume '.'
            let segment = self.expect_identifier()?;
            path.push(segment);
        }

        // Determine import kind
        let kind = if self.check(&Token::LBrace) {
            // Selective: import Math.{abs, max as maximum}
            self.advance(); // consume '{'
            let mut items = Vec::new();
            if !self.check(&Token::RBrace) {
                loop {
                    let name = self.expect_identifier()?;
                    let alias = if self.check(&Token::As) {
                        self.advance();
                        Some(self.expect_identifier()?)
                    } else {
                        None
                    };
                    items.push(ImportItem { name, alias });
                    if !self.check(&Token::Comma) {
                        break;
                    }
                    self.advance();
                }
            }
            self.expect_token(&Token::RBrace, "}")?;
            ImportKind::Selective(items)
        } else if self.check(&Token::As) {
            // Alias: import Math.Vector as Vec
            self.advance();
            let alias = self.expect_identifier()?;
            ImportKind::ModuleAlias(alias)
        } else {
            // Whole module: import Math
            ImportKind::Module
        };

        self.expect_statement_end()?;
        let end = self.previous_span().end;
        Ok(Stmt {
            kind: StmtKind::Import(ImportStmt { path, kind }),
            span: Span {
                start: start.start,
                end,
            },
        })
    }

    fn parse_export_block(&mut self, start: Span) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'export'
        self.expect_token(&Token::LBrace, "{")?;
        let mut names = Vec::new();
        if !self.check(&Token::RBrace) {
            loop {
                names.push(self.expect_identifier()?);
                if !self.check(&Token::Comma) {
                    break;
                }
                self.advance();
            }
        }
        self.expect_token(&Token::RBrace, "}")?;
        self.expect_statement_end()?;
        let end = self.previous_span().end;
        Ok(Stmt {
            kind: StmtKind::ExportBlock(names),
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
        let mut items = Vec::new();
        loop {
            let name = self.expect_identifier()?;
            let alias = if self.check(&Token::As) {
                self.advance();
                Some(self.expect_identifier()?)
            } else {
                None
            };
            items.push(ImportItem { name, alias });
            if !self.check(&Token::Comma) {
                break;
            }
            self.advance();
        }
        self.expect_statement_end()?;
        let end = self.previous_span().end;
        Ok(Stmt {
            kind: StmtKind::Import(ImportStmt {
                path: vec![module_path],
                kind: ImportKind::Selective(items),
            }),
            span: Span {
                start: start.start,
                end,
            },
        })
    }

    fn parse_extern_def(&mut self, start: Span) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'extern'

        // Parse library name as a string literal
        let lib_span = self.current_span();
        let lib_name = match self.peek() {
            Some(Token::DoubleString | Token::SingleString) => {
                let name = self.extract_string_content(&lib_span);
                self.advance();
                name
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    found: self.peek().cloned().unwrap_or(Token::Newline),
                    expected: "string literal for extern library name".into(),
                    span: lib_span,
                });
            }
        };

        self.expect_newline()?;
        self.skip_newlines();

        let mut declarations = Vec::new();

        while !self.check(&Token::End) && !self.is_at_end() {
            // Each declaration is: def name(params) -> ReturnType
            self.expect_token(&Token::Def, "def")?;
            let name = self.expect_identifier()?;
            self.expect_token(&Token::LParen, "(")?;
            let params = self.parse_params()?;
            self.expect_token(&Token::RParen, ")")?;

            // Optional return type: -> Type or -> Type[T, E]
            let return_type = if self.check(&Token::Arrow) {
                self.advance();
                let type_name = self.expect_identifier()?;
                // Skip generic params like [Float, String]
                if self.check(&Token::LBracket) {
                    self.advance();
                    let mut depth = 1;
                    while depth > 0 && !self.is_at_end() {
                        if self.check(&Token::LBracket) {
                            depth += 1;
                        } else if self.check(&Token::RBracket) {
                            depth -= 1;
                        }
                        self.advance();
                    }
                }
                Some(type_name)
            } else {
                None
            };

            declarations.push(ExternDecl {
                name,
                params,
                return_type,
            });

            self.skip_newlines();
        }

        self.expect_token(&Token::End, "end")?;
        let end = self.previous_span().end;

        Ok(Stmt {
            kind: StmtKind::ExternDef {
                lib_name,
                declarations,
            },
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
        let end = message.as_ref().map_or(condition.span.end, |m| m.span.end);
        Ok(Stmt {
            kind: StmtKind::Requires { condition, message },
            span: Span {
                start: start.start,
                end,
            },
        })
    }

    fn parse_try_catch(&mut self, start: Span) -> Result<Stmt, ParseError> {
        let expr = self.parse_try_catch_expr(start)?;
        Ok(Stmt {
            span: expr.span,
            kind: StmtKind::Expr(expr),
        })
    }

    fn parse_try_catch_expr(&mut self, start: Span) -> Result<Expr, ParseError> {
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
        Ok(Expr {
            kind: ExprKind::TryCatch {
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

    fn parse_actor_def(&mut self, start: Span) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'actor'
        let name = self.expect_identifier()?;
        self.expect_newline()?;
        self.skip_newlines();

        let mut init = None;
        let mut receive_cases = Vec::new();
        let mut methods = Vec::new();

        while !self.check(&Token::End) && !self.is_at_end() {
            if self.check(&Token::Def) {
                let func = self.parse_function_def()?;
                // Check if it's init
                if let StmtKind::FuncDef {
                    ref name, ref body, ..
                } = func.kind
                {
                    if name == "init" {
                        init = Some(body.clone());
                    } else {
                        methods.push(func);
                    }
                }
            } else if self.check(&Token::Receive) {
                self.advance(); // consume 'receive'
                self.expect_newline()?;
                self.skip_newlines();
                while self.check(&Token::Case) {
                    self.advance();
                    let pattern = self.parse_pattern()?;
                    self.expect_newline()?;
                    let body = self.parse_block()?;
                    receive_cases.push(MatchCase { pattern, body });
                    self.skip_newlines();
                }
                self.expect_token(&Token::End, "end")?; // end of receive
            } else {
                self.skip_newlines();
                if self.check(&Token::End) {
                    break;
                }
            }
            self.skip_newlines();
        }

        self.expect_token(&Token::End, "end")?;
        let end = self.previous_span().end;
        Ok(Stmt {
            kind: StmtKind::ActorDef {
                name,
                init,
                receive_cases,
                methods,
            },
            span: Span {
                start: start.start,
                end,
            },
        })
    }

    fn parse_reply(&mut self, start: Span) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'reply'
        let expr = self.parse_expression(0)?;
        self.expect_statement_end()?;
        let end = expr.span.end;
        Ok(Stmt {
            kind: StmtKind::Reply(expr),
            span: Span {
                start: start.start,
                end,
            },
        })
    }

    fn parse_instance_assign(&mut self, start: Span) -> Result<Stmt, ParseError> {
        self.advance(); // consume '.'
        let field = self.expect_identifier()?;
        self.expect_token(&Token::Eq, "=")?;
        let value = self.parse_expression(0)?;
        self.expect_statement_end()?;
        let end = value.span.end;
        Ok(Stmt {
            kind: StmtKind::InstanceAssign { field, value },
            span: Span {
                start: start.start,
                end,
            },
        })
    }

    fn parse_macro_def(&mut self, start: Span) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'macro'
        let name = self.expect_identifier()?;
        self.expect_token(&Token::LParen, "(")?;
        let mut params = Vec::new();
        if !self.check(&Token::RParen) {
            loop {
                params.push(self.expect_identifier()?);
                if !self.check(&Token::Comma) {
                    break;
                }
                self.advance();
            }
        }
        self.expect_token(&Token::RParen, ")")?;
        self.expect_newline()?;
        let body = self.parse_block()?;
        self.expect_token(&Token::End, "end")?;
        let end = self.previous_span().end;
        Ok(Stmt {
            kind: StmtKind::MacroDef { name, params, body },
            span: Span {
                start: start.start,
                end,
            },
        })
    }

    fn parse_macro_invoke(&mut self, start: Span) -> Result<Stmt, ParseError> {
        self.advance(); // consume '@'
        let name = self.expect_identifier()?;

        // Collect arguments: expressions until newline
        let mut args = Vec::new();
        let mut had_comma = false;
        while !self.is_at_end() && !self.check(&Token::Newline) && !self.check(&Token::End) {
            args.push(self.parse_expression(0)?);
            if self.check(&Token::Comma) {
                self.advance();
                had_comma = true;
            } else {
                break;
            }
        }

        // Only parse trailing block if no comma-separated args were given.
        // `@name expr NEWLINE block end` = trailing block form
        // `@name arg1, arg2` = inline form (no trailing block)
        let block = if !had_comma && self.check(&Token::Newline) {
            self.advance();
            self.skip_newlines();
            if self.check(&Token::End) || self.is_at_end() {
                None
            } else {
                let body = self.parse_block()?;
                self.expect_token(&Token::End, "end")?;
                Some(body)
            }
        } else {
            self.expect_statement_end()?;
            None
        };

        let end = self.previous_span().end;
        Ok(Stmt {
            kind: StmtKind::MacroInvoke { name, args, block },
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
        // List pattern: [a, b, c] or [head | tail]
        if self.check(&Token::LBracket) {
            self.advance();
            self.skip_newlines();
            let mut elements = Vec::new();
            let mut rest = None;
            if !self.check(&Token::RBracket) {
                loop {
                    self.skip_newlines();
                    elements.push(self.parse_pattern()?);
                    self.skip_newlines();
                    // Check for | rest syntax: [head | tail]
                    if self.check(&Token::Bar) {
                        self.advance();
                        self.skip_newlines();
                        rest = Some(Box::new(self.parse_pattern()?));
                        self.skip_newlines();
                        break;
                    }
                    if !self.check(&Token::Comma) {
                        break;
                    }
                    self.advance();
                }
            }
            self.skip_newlines();
            self.expect_token(&Token::RBracket, "]")?;
            return Ok(Pattern::List(elements, rest));
        }

        // Symbol pattern: :name
        if self.check(&Token::Symbol) {
            let text = self.extract_text(&self.current_span());
            let name = text[1..].to_string();
            let span = self.current_span();
            self.advance();
            return Ok(Pattern::Literal(Expr {
                kind: ExprKind::Symbol(name),
                span,
            }));
        }

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

            // Enum variant pattern: Name.Variant or Name.Variant(patterns)
            if self.check(&Token::Dot) {
                self.advance(); // consume '.'
                let variant_name = self.expect_identifier()?;
                let mut sub_patterns = Vec::new();
                if self.check(&Token::LParen) {
                    self.advance();
                    if !self.check(&Token::RParen) {
                        loop {
                            sub_patterns.push(self.parse_pattern()?);
                            if !self.check(&Token::Comma) {
                                break;
                            }
                            self.advance();
                        }
                    }
                    self.expect_token(&Token::RParen, ")")?;
                }
                return Ok(Pattern::EnumVariant(name, variant_name, sub_patterns));
            }

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
            // Handle `is not` as two-token operator
            let op = if op == BinOp::Is && self.check(&Token::Not) {
                self.advance(); // consume `not`
                BinOp::IsNot
            } else {
                op
            };
            let (prec, _) = op_precedence(op);
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

        // Range operators: .. (exclusive) and ... (inclusive)
        if self.check(&Token::DotDot) || self.check(&Token::DotDotDot) {
            let inclusive = self.check(&Token::DotDotDot);
            self.advance();
            let right = self.parse_unary()?;
            let span = Span {
                start: left.span.start,
                end: right.span.end,
            };
            left = Expr {
                kind: ExprKind::Range {
                    start: Box::new(left),
                    end: Box::new(right),
                    inclusive,
                },
                span,
            };
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();

        if self.check(&Token::Await) {
            self.advance();
            let operand = self.parse_unary()?;
            let span = Span {
                start: start.start,
                end: operand.span.end,
            };
            return Ok(Expr {
                kind: ExprKind::Await(Box::new(operand)),
                span,
            });
        }
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
                // Member access: expr.field (allows keywords like 'send' as method names)
                self.advance();
                let field = self.expect_method_name()?;
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
            Some(Token::SelfKw) => {
                self.advance();
                Ok(Expr {
                    kind: ExprKind::Identifier("self".to_string()),
                    span,
                })
            }
            Some(Token::If) => self.parse_if_expression(),
            Some(Token::Match) => self.parse_match_expression(),
            Some(Token::Try) => self.parse_try_catch_expr(span),
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
                self.skip_newlines();
                let mut elements = Vec::new();
                if !self.check(&Token::RBracket) {
                    loop {
                        self.skip_newlines();
                        elements.push(self.parse_expression(0)?);
                        self.skip_newlines();
                        if !self.check(&Token::Comma) {
                            break;
                        }
                        self.advance();
                    }
                }
                self.skip_newlines();
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
            // Symbol literal: :name
            // AST quasi-quote: ast ... end
            Some(Token::Ast) => {
                self.advance();
                self.skip_newlines();
                let body = self.parse_block()?;
                self.expect_token(&Token::End, "end")?;
                let end = self.previous_span().end;
                Ok(Expr {
                    kind: ExprKind::AstBlock(body),
                    span: Span {
                        start: span.start,
                        end,
                    },
                })
            }
            // Splice: $var
            Some(Token::Dollar) => {
                self.advance();
                let name = self.expect_identifier()?;
                let end = self.previous_span().end;
                Ok(Expr {
                    kind: ExprKind::Splice(name),
                    span: Span {
                        start: span.start,
                        end,
                    },
                })
            }
            Some(Token::Symbol) => {
                let text = self.extract_text(&span);
                let name = text[1..].to_string(); // strip leading ':'
                self.advance();
                Ok(Expr {
                    kind: ExprKind::Symbol(name),
                    span,
                })
            }
            // Dict literal: {key: value, ...} or {:} for empty
            Some(Token::LBrace) => {
                self.advance();
                self.skip_newlines();
                // Empty dict: {:}
                if self.check(&Token::Colon) {
                    self.advance();
                    self.expect_token(&Token::RBrace, "}")?;
                    let end = self.previous_span().end;
                    return Ok(Expr {
                        kind: ExprKind::Dict(vec![]),
                        span: Span {
                            start: span.start,
                            end,
                        },
                    });
                }
                // Dict entries
                let mut entries = Vec::new();
                if !self.check(&Token::RBrace) {
                    loop {
                        self.skip_newlines();
                        let key = self.parse_expression(0)?;
                        self.expect_token(&Token::Colon, ":")?;
                        let value = self.parse_expression(0)?;
                        entries.push((key, value));
                        self.skip_newlines();
                        if !self.check(&Token::Comma) {
                            break;
                        }
                        self.advance();
                    }
                }
                self.skip_newlines();
                self.expect_token(&Token::RBrace, "}")?;
                let end = self.previous_span().end;
                Ok(Expr {
                    kind: ExprKind::Dict(entries),
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
        self.skip_newlines();

        if self.check(&Token::RParen) {
            return Ok(args);
        }

        loop {
            self.skip_newlines();
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

            self.skip_newlines();
            if !self.check(&Token::Comma) {
                break;
            }
            self.advance(); // consume comma
        }
        self.skip_newlines();

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
                // Track string literals so braces inside strings aren't counted.
                let mut expr_text = String::new();
                let mut depth = 1;
                while let Some(&c) = chars.peek() {
                    if c == '"' || c == '\'' {
                        // Skip string literal inside interpolation
                        let q = c;
                        expr_text.push(chars.next().unwrap());
                        while let Some(&sc) = chars.peek() {
                            expr_text.push(chars.next().unwrap());
                            if sc == '\\' {
                                // skip escaped char
                                if let Some(&esc) = chars.peek() {
                                    expr_text.push(chars.next().unwrap());
                                    let _ = esc;
                                }
                            } else if sc == q {
                                break;
                            }
                        }
                    } else if c == '{' {
                        depth += 1;
                        expr_text.push(chars.next().unwrap());
                    } else if c == '}' {
                        depth -= 1;
                        if depth == 0 {
                            chars.next(); // consume closing '}'
                            break;
                        }
                        expr_text.push(chars.next().unwrap());
                    } else {
                        expr_text.push(chars.next().unwrap());
                    }
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

    /// Like expect_identifier but also accepts keyword tokens as method names
    fn expect_method_name(&mut self) -> Result<String, ParseError> {
        // Accept identifiers and keywords that might be used as method names
        let text = self.extract_text(&self.current_span());
        match self.peek() {
            Some(
                Token::Identifier
                | Token::Send
                | Token::Receive
                | Token::Type
                | Token::Match
                | Token::Is,
            ) => {
                self.advance();
                Ok(text)
            }
            Some(tok) => Err(ParseError::UnexpectedToken {
                found: tok.clone(),
                expected: "method name".to_string(),
                span: self.current_span(),
            }),
            None => Err(ParseError::UnexpectedEof {
                expected: "method name".to_string(),
            }),
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
            Some(Token::Is) => Some(BinOp::Is),
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
        BinOp::Is | BinOp::IsNot => (4, Assoc::Left),
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
    fn parse_fstring_nested_quotes() {
        let prog = parse(r#"print(f"val: {d.get("key")}")"#);
        assert_eq!(prog.statements.len(), 1);
        // Should parse as a call with one f-string arg
        match &prog.statements[0].kind {
            StmtKind::Expr(expr) => match &expr.kind {
                ExprKind::Call { args, .. } => {
                    assert_eq!(args.len(), 1);
                    assert!(matches!(args[0].value.kind, ExprKind::FString(_)));
                }
                _ => panic!("expected call"),
            },
            _ => panic!("expected expression"),
        }
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
    fn parse_multiline_list() {
        let prog = parse("[\n  1,\n  2,\n  3\n]");
        match &prog.statements[0].kind {
            StmtKind::Expr(expr) => match &expr.kind {
                ExprKind::List(elements) => assert_eq!(elements.len(), 3),
                _ => panic!("expected list"),
            },
            _ => panic!("expected expression"),
        }
    }

    #[test]
    fn parse_multiline_dict() {
        let prog = parse("{\n  a: 1,\n  b: 2\n}");
        match &prog.statements[0].kind {
            StmtKind::Expr(expr) => match &expr.kind {
                ExprKind::Dict(entries) => assert_eq!(entries.len(), 2),
                _ => panic!("expected dict"),
            },
            _ => panic!("expected expression"),
        }
    }

    #[test]
    fn parse_multiline_call() {
        let prog = parse("foo(\n  1,\n  2,\n  3\n)");
        assert_eq!(prog.statements.len(), 1);
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
                implements,
            } => {
                assert_eq!(name, "Circle");
                assert_eq!(needs.len(), 1);
                assert_eq!(needs[0].name, "radius");
                assert_eq!(methods.len(), 1);
                assert!(implements.is_empty());
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
            StmtKind::Import(imp) => {
                assert_eq!(imp.path, vec!["Shapes"]);
                if let ImportKind::Selective(items) = &imp.kind {
                    assert_eq!(items.len(), 2);
                    assert_eq!(items[0].name, "Circle");
                    assert_eq!(items[1].name, "Rectangle");
                } else {
                    panic!("expected selective");
                }
            }
            _ => panic!("expected import"),
        }
    }

    #[test]
    fn parse_import_module() {
        let prog = parse("import Math");
        match &prog.statements[0].kind {
            StmtKind::Import(imp) => {
                assert_eq!(imp.path, vec!["Math"]);
                assert!(matches!(imp.kind, ImportKind::Module));
            }
            _ => panic!("expected import"),
        }
    }

    #[test]
    fn parse_import_selective() {
        let prog = parse("import Math.{abs, max}");
        match &prog.statements[0].kind {
            StmtKind::Import(imp) => {
                assert_eq!(imp.path, vec!["Math"]);
                if let ImportKind::Selective(items) = &imp.kind {
                    assert_eq!(items.len(), 2);
                    assert_eq!(items[0].name, "abs");
                    assert_eq!(items[1].name, "max");
                } else {
                    panic!("expected selective");
                }
            }
            _ => panic!("expected import"),
        }
    }

    #[test]
    fn parse_import_alias() {
        let prog = parse("import Math.Vector as Vec");
        match &prog.statements[0].kind {
            StmtKind::Import(imp) => {
                assert_eq!(imp.path, vec!["Math", "Vector"]);
                assert!(matches!(imp.kind, ImportKind::ModuleAlias(ref s) if s == "Vec"));
            }
            _ => panic!("expected import"),
        }
    }

    #[test]
    fn parse_export_block() {
        let prog = parse("export {abs, Vector}");
        match &prog.statements[0].kind {
            StmtKind::ExportBlock(names) => {
                assert_eq!(names, &["abs", "Vector"]);
            }
            _ => panic!("expected export block"),
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
        assert!(matches!(prog.statements[0].kind, StmtKind::Requires { .. }));
    }

    #[test]
    fn parse_raise() {
        let prog = parse("raise \"error\"");
        assert!(matches!(prog.statements[0].kind, StmtKind::Raise(_)));
    }

    #[test]
    fn parse_try_catch() {
        let prog = parse("try\n  print(1)\ncatch as e\n  print(e)\nend");
        // try/catch is now an expression (wrapped in StmtKind::Expr)
        match &prog.statements[0].kind {
            StmtKind::Expr(expr) => {
                assert!(matches!(expr.kind, ExprKind::TryCatch { .. }));
            }
            _ => panic!("expected try/catch expression"),
        }
    }

    #[test]
    fn parse_try_as_expression() {
        let prog = parse("x = try\n  42\ncatch as e\n  0\nend");
        assert!(matches!(prog.statements[0].kind, StmtKind::Assign { .. }));
    }

    #[test]
    fn parse_extern_def() {
        let prog = parse(
            "extern \"http\"\n  def listen(port: Int) -> Null\n  def serve(app: App) -> Null\nend",
        );
        match &prog.statements[0].kind {
            StmtKind::ExternDef {
                lib_name,
                declarations,
            } => {
                assert_eq!(lib_name, "http");
                assert_eq!(declarations.len(), 2);
                assert_eq!(declarations[0].name, "listen");
            }
            _ => panic!("expected extern def"),
        }
    }
}
