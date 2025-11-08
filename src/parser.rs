//! # Parser Module
//!
//! Builds an Abstract Syntax Tree from tokens produced by the lexer.
//!
//! This is a recursive descent parser that handles Glimmer-Weave's
//! natural language-inspired syntax.

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::ast::*;
use crate::token::{Token, PositionedToken};
use crate::source_location::SourceSpan;

/// Parser for Glimmer-Weave source code
pub struct Parser {
    tokens: Vec<PositionedToken>,
    position: usize,
}

/// Parser error
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub position: usize,
}

pub type ParseResult<T> = Result<T, ParseError>;

impl Parser {
    /// Create a new parser from a vector of positioned tokens
    pub fn new(tokens: Vec<PositionedToken>) -> Self {
        Parser { tokens, position: 0 }
    }

    /// Get current token
    fn current(&self) -> &Token {
        self.tokens.get(self.position).map(|pt| &pt.token).unwrap_or(&Token::Eof)
    }

    /// Get current token's span
    fn current_span(&self) -> SourceSpan {
        self.tokens.get(self.position)
            .map(|pt| pt.span.to_source_span())
            .unwrap_or_else(SourceSpan::unknown)
    }

    /// Peek at next token
    ///
    /// FUTURE: Will be needed for lookahead parsing of complex expressions
    /// and disambiguation of ambiguous syntax (e.g., generics vs comparison).
    #[allow(dead_code)]
    fn peek(&self) -> &Token {
        self.tokens.get(self.position + 1).map(|pt| &pt.token).unwrap_or(&Token::Eof)
    }

    /// Advance to next token
    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    /// Skip newlines
    fn skip_newlines(&mut self) {
        while matches!(self.current(), Token::Newline) {
            self.advance();
        }
    }

    /// Check if current token matches expected
    fn check(&self, expected: &Token) -> bool {
        core::mem::discriminant(self.current()) == core::mem::discriminant(expected)
    }

    /// Consume token if it matches
    fn match_token(&mut self, expected: Token) -> bool {
        if self.check(&expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Expect a specific token
    fn expect(&mut self, expected: Token) -> ParseResult<()> {
        if self.check(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError {
                message: alloc::format!(
                    "Expected {:?}, found {:?}",
                    expected,
                    self.current()
                ),
                position: self.position,
            })
        }
    }

    /// Parse a complete program
    pub fn parse(&mut self) -> ParseResult<Vec<AstNode>> {
        let mut statements = Vec::new();

        self.skip_newlines();

        while !matches!(self.current(), Token::Eof) {
            statements.push(self.parse_statement()?);
            self.skip_newlines();
        }

        Ok(statements)
    }

    /// Parse a statement
    fn parse_statement(&mut self) -> ParseResult<AstNode> {
        self.skip_newlines();

        match self.current() {
            Token::Bind => self.parse_bind(),
            Token::Weave => self.parse_weave(),
            Token::Set => self.parse_set(),
            Token::Should => self.parse_if(),
            Token::For => self.parse_for(),
            Token::Whilst => self.parse_while(),
            Token::Chant => self.parse_chant_def(),
            Token::Form => self.parse_form_def(),
            Token::Variant => self.parse_variant_def(),
            Token::Aspect => self.parse_aspect_def(),
            Token::Embody => self.parse_embody_stmt(),
            Token::Yield => self.parse_yield(),
            Token::Break => self.parse_break(),
            Token::Continue => self.parse_continue(),
            Token::Match => self.parse_match(),
            Token::Attempt => self.parse_attempt(),
            Token::Request => self.parse_request(),
            // === Module System ===
            Token::Grove => self.parse_module_decl(),
            Token::Summon => self.parse_import(),
            Token::Gather => self.parse_import(), // gather is also handled by parse_import
            Token::Offer => self.parse_export(),
            _ => {
                // Try expression statement
                let expr = self.parse_expression()?;
                Ok(AstNode::ExprStmt {
                    expr: Box::new(expr),
                    span: self.current_span(),
                })
            }
        }
    }

    /// Parse: bind x to 42  OR  bind x: Number to 42
    fn parse_bind(&mut self) -> ParseResult<AstNode> {
        let span = self.current_span();
        self.expect(Token::Bind)?;

        let name = match self.current() {
            Token::Ident(n) => n.clone(),
            _ => {
                return Err(ParseError {
                    message: "Expected identifier after 'bind'".to_string(),
                    position: self.position,
                })
            }
        };
        self.advance();

        // Check for optional type annotation: ': Type'
        let typ = if self.match_token(Token::Colon) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        self.expect(Token::To)?;

        let value = Box::new(self.parse_expression()?);

        Ok(AstNode::BindStmt { name, typ, value, span })
    }

    /// Parse: weave counter as 0  OR  weave counter: Number as 0
    fn parse_weave(&mut self) -> ParseResult<AstNode> {
        let span = self.current_span();
        self.expect(Token::Weave)?;

        let name = match self.current() {
            Token::Ident(n) => n.clone(),
            _ => {
                return Err(ParseError {
                    message: "Expected identifier after 'weave'".to_string(),
                    position: self.position,
                })
            }
        };
        self.advance();

        // Check for optional type annotation: ': Type'
        let typ = if self.match_token(Token::Colon) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        self.expect(Token::As)?;

        let value = Box::new(self.parse_expression()?);

        Ok(AstNode::WeaveStmt { name, typ, value, span })
    }

    /// Parse: set counter to 10, set list[i] to 5, set obj.field to "value"
    fn parse_set(&mut self) -> ParseResult<AstNode> {
        let span = self.current_span();
        self.expect(Token::Set)?;

        // Parse target expression (identifier, index access, or field access)
        let target = Box::new(self.parse_postfix()?);

        self.expect(Token::To)?;

        let value = Box::new(self.parse_expression()?);

        Ok(AstNode::SetStmt { target, value, span })
    }

    /// Parse: should x > 5 then ... otherwise ... end
    fn parse_if(&mut self) -> ParseResult<AstNode> {
        let span = self.current_span();
        self.expect(Token::Should)?;

        let condition = Box::new(self.parse_expression()?);

        self.expect(Token::Then)?;
        self.skip_newlines();

        let mut then_branch = Vec::new();
        while !matches!(self.current(), Token::Otherwise | Token::End | Token::Eof) {
            then_branch.push(self.parse_statement()?);
            self.skip_newlines();
        }

        let else_branch = if self.match_token(Token::Otherwise) {
            self.skip_newlines();
            let mut else_stmts = Vec::new();
            while !matches!(self.current(), Token::End | Token::Eof) {
                else_stmts.push(self.parse_statement()?);
                self.skip_newlines();
            }
            Some(else_stmts)
        } else {
            None
        };

        self.expect(Token::End)?;

        Ok(AstNode::IfStmt {
            condition,
            then_branch,
            else_branch,
            span })
    }

    /// Parse: for each x in list then ... end
    fn parse_for(&mut self) -> ParseResult<AstNode> {
        let span = self.current_span();
        self.expect(Token::For)?;
        self.expect(Token::Each)?;

        let variable = match self.current() {
            Token::Ident(n) => n.clone(),
            _ => {
                return Err(ParseError {
                    message: "Expected identifier after 'for each'".to_string(),
                    position: self.position,
                })
            }
        };
        self.advance();

        self.expect(Token::In)?;

        let iterable = Box::new(self.parse_expression()?);

        self.expect(Token::Then)?;
        self.skip_newlines();

        let mut body = Vec::new();
        while !matches!(self.current(), Token::End | Token::Eof) {
            body.push(self.parse_statement()?);
            self.skip_newlines();
        }

        self.expect(Token::End)?;

        Ok(AstNode::ForStmt {
            variable,
            iterable,
            body,
            span,
        })
    }

    /// Parse: whilst condition then ... end
    fn parse_while(&mut self) -> ParseResult<AstNode> {
        let span = self.current_span();
        self.expect(Token::Whilst)?;

        let condition = Box::new(self.parse_expression()?);

        self.expect(Token::Then)?;
        self.skip_newlines();

        let mut body = Vec::new();
        while !matches!(self.current(), Token::End | Token::Eof) {
            body.push(self.parse_statement()?);
            self.skip_newlines();
        }

        self.expect(Token::End)?;

        Ok(AstNode::WhileStmt {
            condition,
            body,
            span })
    }

    /// Parse: chant greet(name) then ... end
    fn parse_chant_def(&mut self) -> ParseResult<AstNode> {
        self.expect(Token::Chant)?;

        let name = match self.current() {
            Token::Ident(n) => n.clone(),
            _ => {
                return Err(ParseError {
                    message: "Expected identifier after 'chant'".to_string(),
                    position: self.position,
                })
            }
        };
        self.advance();

        // Parse optional generic type parameters: <T, U>
        let type_params = if matches!(self.current(), Token::LeftAngle) {
            self.advance(); // consume <
            let mut params = Vec::new();

            loop {
                match self.current() {
                    Token::Ident(param_name) => {
                        params.push(param_name.clone());
                        self.advance();

                        if matches!(self.current(), Token::Comma) {
                            self.advance(); // consume comma
                        } else {
                            break;
                        }
                    }
                    _ => {
                        return Err(ParseError {
                            message: "Expected type parameter name".to_string(),
                            position: self.position,
                        })
                    }
                }
            }

            self.expect(Token::RightAngle)?;
            params
        } else {
            Vec::new() // No generic type parameters
        };

        // Parse parameters with optional type annotations
        self.expect(Token::LeftParen)?;
        self.skip_newlines();  // Skip newlines after opening paren

        let mut params = Vec::new();
        if !matches!(self.current(), Token::RightParen) {
            loop {
                // Check for variadic parameter: ...name
                let is_variadic = self.match_token(Token::Ellipsis);

                // Check for borrow mode: borrow [mut]
                let (borrow_mode, lifetime) = if self.match_token(Token::Borrow) {
                    // Check if it's mutable borrow
                    let is_mut = self.match_token(Token::Mut);

                    // TODO: Parse lifetime annotations like 'a, 'b
                    // For now, no lifetime support in parameters
                    let lifetime = None;

                    if is_mut {
                        (BorrowMode::BorrowedMut, lifetime)
                    } else {
                        (BorrowMode::Borrowed, lifetime)
                    }
                } else {
                    (BorrowMode::Owned, None)
                };

                let param_name = match self.current() {
                    Token::Ident(p) => p.clone(),
                    _ => {
                        return Err(ParseError {
                            message: "Expected parameter name".to_string(),
                            position: self.position,
                        })
                    }
                };
                self.advance();

                // Check for optional type annotation: 'as Type'
                let param_type = if self.match_token(Token::As) {
                    Some(self.parse_type_annotation()?)
                } else {
                    None
                };

                params.push(Parameter {
                    name: param_name,
                    typ: param_type,
                    is_variadic,
                    borrow_mode,
                    lifetime,
                });

                // If this is a variadic parameter, it must be the last one
                if is_variadic {
                    if self.match_token(Token::Comma) {
                        return Err(ParseError {
                            message: "Variadic parameter must be the last parameter".to_string(),
                            position: self.position,
                        });
                    }
                    break;
                }

                if !self.match_token(Token::Comma) {
                    break;
                }
                self.skip_newlines();  // Skip newlines after comma
            }
        }

        self.skip_newlines();  // Skip newlines before closing paren
        self.expect(Token::RightParen)?;

        // Check for optional return type: '-> Type'
        let return_type = if self.match_token(Token::Arrow) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        self.expect(Token::Then)?;
        self.skip_newlines();

        let mut body = Vec::new();
        while !matches!(self.current(), Token::End | Token::Eof) {
            body.push(self.parse_statement()?);
            self.skip_newlines();
        }

        self.expect(Token::End)?;

        Ok(AstNode::ChantDef {
            name,
            type_params,
            params,
            return_type,
            body,
            lifetime_params: Vec::new(),
            span: self.current_span(),
        })
    }

    /// Parse: form Person with name as Text age as Number end
    /// or: form Box<T> with value as T end
    fn parse_form_def(&mut self) -> ParseResult<AstNode> {
        self.expect(Token::Form)?;

        let name = match self.current() {
            Token::Ident(n) => n.clone(),
            _ => {
                return Err(ParseError {
                    message: "Expected identifier after 'form'".to_string(),
                    position: self.position,
                })
            }
        };
        self.advance();

        // Parse optional generic type parameters: <T, U>
        let type_params = if matches!(self.current(), Token::LeftAngle) {
            self.advance(); // consume <
            let mut params = Vec::new();

            loop {
                match self.current() {
                    Token::Ident(param_name) => {
                        params.push(param_name.clone());
                        self.advance();

                        if matches!(self.current(), Token::Comma) {
                            self.advance(); // consume comma
                        } else {
                            break;
                        }
                    }
                    _ => {
                        return Err(ParseError {
                            message: "Expected type parameter name".to_string(),
                            position: self.position,
                        })
                    }
                }
            }

            self.expect(Token::RightAngle)?;
            params
        } else {
            Vec::new() // No generic type parameters
        };

        self.expect(Token::With)?;
        self.skip_newlines();

        let mut fields = Vec::new();
        while !matches!(self.current(), Token::End | Token::Eof) {
            // Parse field: name as Type
            let field_name = match self.current() {
                Token::Ident(n) => n.clone(),
                _ => {
                    return Err(ParseError {
                        message: "Expected field name in struct definition".to_string(),
                        position: self.position,
                    })
                }
            };
            self.advance();

            self.expect(Token::As)?;

            let field_type = self.parse_type_annotation()?;

            fields.push(StructField {
                name: field_name,
                typ: field_type,
            });

            self.skip_newlines();
        }

        self.expect(Token::End)?;

        Ok(AstNode::FormDef {
            name,
            type_params,
            fields,
            span: self.current_span(),
        })
    }

    /// Parse: variant Color then Red, Green, Blue end
    /// or with data: variant Message then Quit, Move(x: Number, y: Number) end
    /// or with generics: variant Option<T> then Some(value: T), None end
    fn parse_variant_def(&mut self) -> ParseResult<AstNode> {
        self.expect(Token::Variant)?;

        let name = match self.current() {
            Token::Ident(n) => n.clone(),
            _ => {
                return Err(ParseError {
                    message: "Expected identifier after 'variant'".to_string(),
                    position: self.position,
                })
            }
        };
        self.advance();

        // Parse optional generic type parameters: <T, U>
        let type_params = if matches!(self.current(), Token::LeftAngle) {
            self.advance(); // consume <
            let mut params = Vec::new();

            loop {
                match self.current() {
                    Token::Ident(param_name) => {
                        params.push(param_name.clone());
                        self.advance();

                        if matches!(self.current(), Token::Comma) {
                            self.advance(); // consume comma
                        } else {
                            break;
                        }
                    }
                    _ => {
                        return Err(ParseError {
                            message: "Expected type parameter name".to_string(),
                            position: self.position,
                        })
                    }
                }
            }

            self.expect(Token::RightAngle)?;
            params
        } else {
            Vec::new() // No generic type parameters
        };

        self.expect(Token::Then)?;
        self.skip_newlines();

        let mut variants = Vec::new();
        while !matches!(self.current(), Token::End | Token::Eof) {
            // Parse variant case: Name or Name(field1: Type1, field2: Type2)
            let variant_name = match self.current() {
                Token::Ident(n) => n.clone(),
                _ => {
                    return Err(ParseError {
                        message: "Expected variant name in enum definition".to_string(),
                        position: self.position,
                    })
                }
            };
            self.advance();

            // Check for fields in parentheses
            let fields = if matches!(self.current(), Token::LeftParen) {
                self.advance(); // consume (
                let mut variant_fields = Vec::new();

                while !matches!(self.current(), Token::RightParen | Token::Eof) {
                    // Parse field: name: Type
                    let field_name = match self.current() {
                        Token::Ident(n) => n.clone(),
                        _ => {
                            return Err(ParseError {
                                message: "Expected field name in variant".to_string(),
                                position: self.position,
                            })
                        }
                    };
                    self.advance();

                    self.expect(Token::Colon)?;

                    let field_type = self.parse_type_annotation()?;

                    variant_fields.push(Parameter {
                        name: field_name,
                        typ: Some(field_type),
                        is_variadic: false,
                        borrow_mode: BorrowMode::Owned,
                        lifetime: None,
                    });

                    // Handle comma separator
                    if matches!(self.current(), Token::Comma) {
                        self.advance();
                    } else if !matches!(self.current(), Token::RightParen) {
                        return Err(ParseError {
                            message: "Expected ',' or ')' in variant field list".to_string(),
                            position: self.position,
                        });
                    }
                }

                self.expect(Token::RightParen)?;
                variant_fields
            } else {
                Vec::new() // Unit variant (no fields)
            };

            variants.push(VariantCase {
                name: variant_name,
                fields,
            });

            // Handle comma separator between variants (optional)
            if matches!(self.current(), Token::Comma) {
                self.advance();
            }

            self.skip_newlines();
        }

        self.expect(Token::End)?;

        Ok(AstNode::VariantDef {
            name,
            type_params,
            variants,
            span: self.current_span(),
        })
    }

    /// Parse trait definition: aspect Display then chant show(self) -> Text end
    /// or with generics: aspect Container<T> then chant add(self, item: T) end
    fn parse_aspect_def(&mut self) -> ParseResult<AstNode> {
        self.expect(Token::Aspect)?;

        let name = match self.current() {
            Token::Ident(n) => n.clone(),
            _ => {
                return Err(ParseError {
                    message: "Expected identifier after 'aspect'".to_string(),
                    position: self.position,
                })
            }
        };
        self.advance();

        // Parse optional generic type parameters: <T, U>
        let type_params = if matches!(self.current(), Token::LeftAngle) {
            self.advance(); // consume <
            let mut params = Vec::new();

            loop {
                match self.current() {
                    Token::Ident(param_name) => {
                        params.push(param_name.clone());
                        self.advance();

                        if matches!(self.current(), Token::Comma) {
                            self.advance(); // consume comma
                        } else {
                            break;
                        }
                    }
                    _ => {
                        return Err(ParseError {
                            message: "Expected type parameter name".to_string(),
                            position: self.position,
                        })
                    }
                }
            }

            self.expect(Token::RightAngle)?;
            params
        } else {
            Vec::new()
        };

        self.expect(Token::Then)?;
        self.skip_newlines();

        // Parse trait method signatures
        let mut methods = Vec::new();
        while !matches!(self.current(), Token::End | Token::Eof) {
            // Parse method signature: chant method_name(self, param: Type) -> ReturnType
            self.expect(Token::Chant)?;

            let method_name = match self.current() {
                Token::Ident(n) => n.clone(),
                _ => {
                    return Err(ParseError {
                        message: "Expected method name in aspect".to_string(),
                        position: self.position,
                    })
                }
            };
            self.advance();

            self.expect(Token::LeftParen)?;

            // Parse parameters (first must be 'self')
            let mut params = Vec::new();

            // Parse first parameter (must be 'self')
            match self.current() {
                Token::Ident(name) if name == "self" => {
                    params.push(Parameter {
                        name: "self".to_string(),
                        typ: None, // self type is inferred
                        is_variadic: false,
                        borrow_mode: BorrowMode::Owned,
                        lifetime: None,
                    });
                    self.advance();
                }
                _ => {
                    return Err(ParseError {
                        message: "Trait methods must have 'self' as first parameter".to_string(),
                        position: self.position,
                    })
                }
            }

            // Parse remaining parameters
            while matches!(self.current(), Token::Comma) {
                self.advance(); // consume comma

                if matches!(self.current(), Token::RightParen) {
                    break; // Trailing comma
                }

                let param_name = match self.current() {
                    Token::Ident(n) => n.clone(),
                    _ => {
                        return Err(ParseError {
                            message: "Expected parameter name".to_string(),
                            position: self.position,
                        })
                    }
                };
                self.advance();

                // Optional type annotation
                let typ = if matches!(self.current(), Token::Colon) {
                    self.advance();
                    Some(self.parse_type_annotation()?)
                } else {
                    None
                };

                params.push(Parameter {
                    name: param_name,
                    typ,
                    is_variadic: false,
                    borrow_mode: BorrowMode::Owned,
                    lifetime: None,
                });
            }

            self.expect(Token::RightParen)?;

            // Parse optional return type
            let return_type = if matches!(self.current(), Token::Arrow) {
                self.advance();
                Some(self.parse_type_annotation()?)
            } else {
                None
            };

            methods.push(TraitMethod {
                name: method_name,
                params,
                return_type,
            });

            self.skip_newlines();
        }

        self.expect(Token::End)?;

        Ok(AstNode::AspectDef {
            name,
            type_params,
            methods,
            span: self.current_span(),
        })
    }

    /// Parse trait implementation: embody Display for Number then chant show(self) -> Text then ... end end
    /// or with generic trait: embody Container<Number> for NumberList then ... end
    fn parse_embody_stmt(&mut self) -> ParseResult<AstNode> {
        self.expect(Token::Embody)?;

        // Parse aspect name
        let aspect_name = match self.current() {
            Token::Ident(n) => n.clone(),
            _ => {
                return Err(ParseError {
                    message: "Expected aspect name after 'embody'".to_string(),
                    position: self.position,
                })
            }
        };
        self.advance();

        // Parse optional type arguments: <Number, Text>
        let type_args = if matches!(self.current(), Token::LeftAngle) {
            self.advance(); // consume <
            let mut args = Vec::new();

            loop {
                args.push(self.parse_type_annotation()?);

                if matches!(self.current(), Token::Comma) {
                    self.advance(); // consume comma
                } else {
                    break;
                }
            }

            self.expect(Token::RightAngle)?;
            args
        } else {
            Vec::new()
        };

        self.expect(Token::For)?;

        // Parse target type
        let target_type = self.parse_type_annotation()?;

        self.expect(Token::Then)?;
        self.skip_newlines();

        // Parse method implementations (full ChantDef nodes)
        let mut methods = Vec::new();
        while !matches!(self.current(), Token::End | Token::Eof) {
            // Parse method implementation
            let method = self.parse_chant_def()?;
            methods.push(method);
            self.skip_newlines();
        }

        self.expect(Token::End)?;

        Ok(AstNode::EmbodyStmt {
            aspect_name,
            type_args,
            target_type,
            methods,
            span: self.current_span(),
        })
    }

    /// Parse: yield result
    fn parse_yield(&mut self) -> ParseResult<AstNode> {
        let span = self.current_span();
        self.expect(Token::Yield)?;

        let value = Box::new(self.parse_expression()?);

        Ok(AstNode::YieldStmt { value, span })
    }

    /// Parse: break
    fn parse_break(&mut self) -> ParseResult<AstNode> {
        let span = self.current_span();
        self.expect(Token::Break)?;
        Ok(AstNode::Break { span })
    }

    /// Parse: continue
    fn parse_continue(&mut self) -> ParseResult<AstNode> {
        let span = self.current_span();
        self.expect(Token::Continue)?;
        Ok(AstNode::Continue { span })
    }

    /// Parse: match x with when pattern then ... end
    fn parse_match(&mut self) -> ParseResult<AstNode> {
        self.expect(Token::Match)?;

        let value = Box::new(self.parse_expression()?);

        self.expect(Token::With)?;
        self.skip_newlines();

        let mut arms = Vec::new();
        while matches!(self.current(), Token::When | Token::Otherwise) {
            if self.match_token(Token::When) {
                let pattern = self.parse_pattern()?;
                self.expect(Token::Then)?;
                self.skip_newlines();

                let mut body = Vec::new();
                while !matches!(
                    self.current(),
                    Token::When | Token::Otherwise | Token::End | Token::Eof
                ) {
                    body.push(self.parse_statement()?);
                    self.skip_newlines();
                }

                arms.push(MatchArm { pattern, body });
            } else if self.match_token(Token::Otherwise) {
                self.expect(Token::Then)?;
                self.skip_newlines();

                let mut body = Vec::new();
                while !matches!(self.current(), Token::End | Token::Eof) {
                    body.push(self.parse_statement()?);
                    self.skip_newlines();
                }

                arms.push(MatchArm {
                    pattern: Pattern::Wildcard,
                    body,
                });
                break;
            }
        }

        self.expect(Token::End)?;

        Ok(AstNode::MatchStmt { value, arms, span: self.current_span() })
    }

    /// Parse pattern for match
    fn parse_pattern(&mut self) -> ParseResult<Pattern> {
        match self.current() {
            Token::Number(n) => {
                let val = *n;
                let span = self.current_span();
                self.advance();
                Ok(Pattern::Literal(AstNode::Number { value: val, span }))
            }
            Token::Text(s) => {
                let val = s.clone();
                let span = self.current_span();
                self.advance();
                Ok(Pattern::Literal(AstNode::Text { value: val, span }))
            }
            Token::Truth(b) => {
                let val = *b;
                let span = self.current_span();
                self.advance();
                Ok(Pattern::Literal(AstNode::Truth { value: val, span }))
            }
            Token::Ident(name) => {
                let n = name.clone();
                self.advance();

                // Phase 2: Check if this is a variant pattern with fields: Ident(pattern, ...)
                if self.match_token(Token::LeftParen) {
                    // Parse inner patterns for field extraction
                    let mut inner_patterns = Vec::new();

                    // Parse first pattern
                    if !matches!(self.current(), Token::RightParen) {
                        inner_patterns.push(self.parse_pattern()?);

                        // Parse additional patterns separated by commas
                        while self.match_token(Token::Comma) {
                            inner_patterns.push(self.parse_pattern()?);
                        }
                    }

                    self.expect(Token::RightParen)?;

                    // For multiple fields, wrap in a tuple-like structure
                    // For now, we'll encode multiple patterns as nested Enum patterns
                    // The innermost pattern will be the last field, working backwards
                    if inner_patterns.is_empty() {
                        // No fields - unit variant pattern
                        Ok(Pattern::Enum {
                            variant: n,
                            inner: None,
                        })
                    } else if inner_patterns.len() == 1 {
                        // Single field - simple case
                        Ok(Pattern::Enum {
                            variant: n,
                            inner: Some(Box::new(inner_patterns.into_iter().next().unwrap())),
                        })
                    } else {
                        // Multiple fields - we need a way to represent this
                        // For now, we'll create a special marker using a list literal pattern
                        // This is a workaround until we add a proper Tuple pattern type
                        Ok(Pattern::Enum {
                            variant: n,
                            inner: Some(Box::new(Pattern::Literal(
                                AstNode::List {
                                    elements: inner_patterns.into_iter()
                                        .map(|p| match p {
                                            Pattern::Ident(name) => AstNode::Ident { name, span: SourceSpan::unknown() },
                                            _ => AstNode::Nothing { span: SourceSpan::unknown() }, // Placeholder
                                        })
                                        .collect(),
                                    span: SourceSpan::unknown(),
                                }
                            ))),
                        })
                    }
                } else {
                    // Just an identifier pattern
                    Ok(Pattern::Ident(n))
                }
            }

            // Enum patterns
            Token::Triumph => {
                self.advance();
                self.expect(Token::LeftParen)?;
                let inner = Box::new(self.parse_pattern()?);
                self.expect(Token::RightParen)?;
                Ok(Pattern::Enum {
                    variant: "Triumph".to_string(),
                    inner: Some(inner),
                })
            }
            Token::Mishap => {
                self.advance();
                self.expect(Token::LeftParen)?;
                let inner = Box::new(self.parse_pattern()?);
                self.expect(Token::RightParen)?;
                Ok(Pattern::Enum {
                    variant: "Mishap".to_string(),
                    inner: Some(inner),
                })
            }
            Token::Present => {
                self.advance();
                self.expect(Token::LeftParen)?;
                let inner = Box::new(self.parse_pattern()?);
                self.expect(Token::RightParen)?;
                Ok(Pattern::Enum {
                    variant: "Present".to_string(),
                    inner: Some(inner),
                })
            }
            Token::Absent => {
                self.advance();
                Ok(Pattern::Enum {
                    variant: "Absent".to_string(),
                    inner: None,
                })
            }

            _ => Err(ParseError {
                message: "Expected pattern".to_string(),
                position: self.position,
            }),
        }
    }

    /// Parse: attempt ... harmonize on Error then ... end
    fn parse_attempt(&mut self) -> ParseResult<AstNode> {
        self.expect(Token::Attempt)?;
        self.skip_newlines();

        let mut body = Vec::new();
        while !matches!(self.current(), Token::Harmonize | Token::End | Token::Eof) {
            body.push(self.parse_statement()?);
            self.skip_newlines();
        }

        let mut handlers = Vec::new();
        while self.match_token(Token::Harmonize) {
            self.expect(Token::On)?;

            let error_type = match self.current() {
                Token::Ident(e) => e.clone(),
                _ => {
                    return Err(ParseError {
                        message: "Expected error type after 'on'".to_string(),
                        position: self.position,
                    })
                }
            };
            self.advance();

            self.expect(Token::Then)?;
            self.skip_newlines();

            let mut handler_body = Vec::new();
            while !matches!(
                self.current(),
                Token::Harmonize | Token::End | Token::Eof
            ) {
                handler_body.push(self.parse_statement()?);
                self.skip_newlines();
            }

            handlers.push(ErrorHandler {
                error_type,
                body: handler_body,
            });
        }

        self.expect(Token::End)?;

        Ok(AstNode::AttemptStmt { body, handlers, span: self.current_span() })
    }

    /// Parse: request VGA.write with justification "message"
    fn parse_request(&mut self) -> ParseResult<AstNode> {
        let span = self.current_span();
        self.expect(Token::Request)?;

        let capability = Box::new(self.parse_expression()?);

        self.expect(Token::With)?;
        self.expect(Token::Justification)?;

        let justification = match self.current() {
            Token::Text(s) => s.clone(),
            _ => {
                return Err(ParseError {
                    message: "Expected string after 'justification'".to_string(),
                    position: self.position,
                })
            }
        };
        self.advance();

        Ok(AstNode::RequestStmt {
            capability,
            justification,
            span,
        })
    }

    // === Module System Parsing (Phase 1) ===

    /// Parse: grove Math with body end
    fn parse_module_decl(&mut self) -> ParseResult<AstNode> {
        self.expect(Token::Grove)?;

        let name = match self.current() {
            Token::Ident(n) => n.clone(),
            _ => {
                return Err(ParseError {
                    message: "Expected module name after 'grove'".to_string(),
                    position: self.position,
                })
            }
        };
        self.advance();

        self.expect(Token::With)?;
        self.skip_newlines();

        let mut body = Vec::new();
        let mut exports = Vec::new();

        // Parse module body until 'end'
        while !matches!(self.current(), Token::End | Token::Eof) {
            let stmt = self.parse_statement()?;

            // If it's an Export statement, extract the items
            if let AstNode::Export { items, .. } = &stmt {
                exports.extend(items.clone());
            }

            body.push(stmt);
            self.skip_newlines();
        }

        self.expect(Token::End)?;

        Ok(AstNode::ModuleDecl {
            name,
            body,
            exports,
            span: self.current_span(),
        })
    }

    /// Parse: summon Math from "std/math.gw"
    ///    OR: summon Math from "std/math.gw" as M
    ///    OR: gather sqrt, pow from Math
    fn parse_import(&mut self) -> ParseResult<AstNode> {
        // Determine if this is 'summon' (import all) or 'gather' (import specific)
        let is_gather = matches!(self.current(), Token::Gather);
        self.advance(); // consume 'summon' or 'gather'

        // Parse module name or items depending on keyword
        let (module_name_hint, items) = if is_gather {
            // gather: parse items list first
            let mut item_list = Vec::new();

            loop {
                match self.current() {
                    Token::Ident(name) => {
                        item_list.push(name.clone());
                        self.advance();

                        if matches!(self.current(), Token::Comma) {
                            self.advance(); // consume comma
                        } else {
                            break;
                        }
                    }
                    _ => {
                        return Err(ParseError {
                            message: "Expected identifier in gather list".to_string(),
                            position: self.position,
                        })
                    }
                }
            }

            (None, Some(item_list))
        } else {
            // summon: parse module name (optional, can be inferred from path)
            // Syntax: summon Math from "path" OR summon from "path"
            if !matches!(self.current(), Token::From) {
                let name = match self.current() {
                    Token::Ident(n) => Some(n.clone()),
                    _ => {
                        return Err(ParseError {
                            message: "Expected module name or 'from' after 'summon'".to_string(),
                            position: self.position,
                        })
                    }
                };
                self.advance();
                (name, None)
            } else {
                (None, None)
            }
        };

        // Parse 'from' clause
        self.expect(Token::From)?;

        // Parse module path (can be identifier or string)
        let (module_name, path) = match self.current() {
            Token::Text(p) => {
                // Path is a string: from "std/math.gw"
                // Extract module name from path (e.g., "std/math.gw" -> "math")
                let extracted_name = p.trim_end_matches(".gw")
                    .split('/')
                    .last()
                    .unwrap_or("unknown")
                    .to_string();

                // Use module name hint if provided, otherwise extract from path
                let final_name = module_name_hint.unwrap_or(extracted_name);
                let path = p.clone();
                self.advance();
                (final_name, path)
            }
            Token::Ident(name) => {
                // Module name: from Math (path will be inferred)
                let module_name = module_name_hint.unwrap_or_else(|| name.clone());
                let path = format!("{}.gw", name);
                self.advance();
                (module_name, path)
            }
            _ => {
                return Err(ParseError {
                    message: "Expected module path (string) or name (identifier) after 'from'".to_string(),
                    position: self.position,
                })
            }
        };

        // Parse optional 'as' alias
        let alias = if matches!(self.current(), Token::As) {
            self.advance(); // consume 'as'

            match self.current() {
                Token::Ident(alias_name) => {
                    let a = Some(alias_name.clone());
                    self.advance();
                    a
                }
                _ => {
                    return Err(ParseError {
                        message: "Expected identifier after 'as'".to_string(),
                        position: self.position,
                    })
                }
            }
        } else {
            None
        };

        Ok(AstNode::Import {
            module_name,
            path,
            items,
            alias,
            span: self.current_span(),
        })
    }

    /// Parse: offer sqrt, pow
    fn parse_export(&mut self) -> ParseResult<AstNode> {
        self.expect(Token::Offer)?;

        let mut items = Vec::new();

        loop {
            match self.current() {
                Token::Ident(name) => {
                    items.push(name.clone());
                    self.advance();

                    if matches!(self.current(), Token::Comma) {
                        self.advance(); // consume comma
                    } else {
                        break;
                    }
                }
                _ => {
                    return Err(ParseError {
                        message: "Expected identifier in export list".to_string(),
                        position: self.position,
                    })
                }
            }
        }

        if items.is_empty() {
            return Err(ParseError {
                message: "Expected at least one item to export after 'offer'".to_string(),
                position: self.position,
            });
        }

        Ok(AstNode::Export { items, span: self.current_span() })
    }

    /// Parse an expression
    fn parse_expression(&mut self) -> ParseResult<AstNode> {
        self.parse_pipeline()
    }

    /// Parse pipeline: x | filter | sort
    fn parse_pipeline(&mut self) -> ParseResult<AstNode> {
        let mut expr = self.parse_logical_or()?;

        if matches!(self.current(), Token::Pipe) {
            let span = self.current_span();
            let mut stages = Vec::new();
            stages.push(expr);

            while self.match_token(Token::Pipe) {
                stages.push(self.parse_logical_or()?);
            }

            expr = AstNode::Pipeline { stages, span };
        }

        Ok(expr)
    }

    /// Parse logical OR: a or b
    fn parse_logical_or(&mut self) -> ParseResult<AstNode> {
        let mut left = self.parse_logical_and()?;

        while self.match_token(Token::Or) {
            let span = self.current_span();
            let right = self.parse_logical_and()?;
            left = AstNode::BinaryOp {
                left: Box::new(left),
                op: BinaryOperator::Or,
                right: Box::new(right),
                span };
        }

        Ok(left)
    }

    /// Parse logical AND: a and b
    fn parse_logical_and(&mut self) -> ParseResult<AstNode> {
        let mut left = self.parse_comparison()?;

        while self.match_token(Token::And) {
            let span = self.current_span();
            let right = self.parse_comparison()?;
            left = AstNode::BinaryOp {
                left: Box::new(left),
                op: BinaryOperator::And,
                right: Box::new(right),
                span };
        }

        Ok(left)
    }

    /// Parse comparison: a > b, x is y
    fn parse_comparison(&mut self) -> ParseResult<AstNode> {
        let mut left = self.parse_additive()?;

        loop {
            let op = match self.current() {
                Token::Is => BinaryOperator::Equal,
                Token::IsNot => BinaryOperator::NotEqual,
                Token::GreaterThan => BinaryOperator::Greater,
                Token::LessThan => BinaryOperator::Less,
                Token::AtLeast => BinaryOperator::GreaterEq,
                Token::AtMost => BinaryOperator::LessEq,
                _ => break,
            };

            let span = self.current_span();
            self.advance();
            let right = self.parse_additive()?;
            left = AstNode::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span };
        }

        Ok(left)
    }

    /// Parse addition/subtraction: a + b, x - y
    fn parse_additive(&mut self) -> ParseResult<AstNode> {
        let mut left = self.parse_multiplicative()?;

        loop {
            let op = match self.current() {
                Token::Plus => BinaryOperator::Add,
                Token::Minus => BinaryOperator::Sub,
                _ => break,
            };

            let span = self.current_span();
            self.advance();
            let right = self.parse_multiplicative()?;
            left = AstNode::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span };
        }

        Ok(left)
    }

    /// Parse multiplication/division: a * b, x / y
    fn parse_multiplicative(&mut self) -> ParseResult<AstNode> {
        let mut left = self.parse_unary()?;

        loop {
            let op = match self.current() {
                Token::Star => BinaryOperator::Mul,
                Token::Slash => BinaryOperator::Div,
                Token::Percent => BinaryOperator::Mod,
                _ => break,
            };

            let span = self.current_span();
            self.advance();
            let right = self.parse_unary()?;
            left = AstNode::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span };
        }

        Ok(left)
    }

    /// Parse unary: not x, -y
    fn parse_unary(&mut self) -> ParseResult<AstNode> {
        match self.current() {
            Token::Not => {
                let span = self.current_span();
                self.advance();
                Ok(AstNode::UnaryOp {
                    op: UnaryOperator::Not,
                    operand: Box::new(self.parse_unary()?),
                    span,
                })
            }
            Token::Minus => {
                let span = self.current_span();
                self.advance();
                Ok(AstNode::UnaryOp {
                    op: UnaryOperator::Negate,
                    operand: Box::new(self.parse_unary()?),
                    span,
                })
            }
            Token::Borrow => {
                let span = self.current_span();
                self.advance();
                // Check for 'borrow mut'
                let mutable = self.match_token(Token::Mut);
                Ok(AstNode::BorrowExpr {
                    value: Box::new(self.parse_unary()?),
                    mutable,
                    span,
                })
            }
            _ => self.parse_postfix(),
        }
    }

    /// Parse postfix: call, field access, index
    fn parse_postfix(&mut self) -> ParseResult<AstNode> {
        let mut expr = self.parse_primary()?;

        loop {
            match self.current() {
                Token::Dot => {
                    let span = self.current_span();
                    self.advance();
                    let field = match self.current() {
                        Token::Ident(f) => f.clone(),
                        _ => {
                            return Err(ParseError {
                                message: "Expected field name after '.'".to_string(),
                                position: self.position,
                            })
                        }
                    };
                    self.advance();
                    expr = AstNode::FieldAccess {
                        object: Box::new(expr),
                        field,
                        span,
                    };
                }
                Token::LeftAngle => {
                    // Parse type arguments: identity<Number> or Box<T>
                    // This must be followed by either ( for function call or { for struct literal
                    self.advance(); // consume <

                    let mut type_args = Vec::new();
                    loop {
                        type_args.push(self.parse_type_annotation()?);
                        if matches!(self.current(), Token::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }

                    self.expect(Token::RightAngle)?;

                    // Now we expect either ( for call or { for struct literal
                    match self.current() {
                        Token::LeftParen => {
                            // Generic function call: identity<Number>(42)
                            self.advance();
                            let mut args = Vec::new();

                            if !matches!(self.current(), Token::RightParen) {
                                loop {
                                    args.push(self.parse_expression()?);
                                    if !self.match_token(Token::Comma) {
                                        break;
                                    }
                                }
                            }

                            self.expect(Token::RightParen)?;
                            expr = AstNode::Call {
                                callee: Box::new(expr),
                                type_args,
                                args,
                                span: self.current_span(),
                            };
                        }
                        Token::LeftBrace => {
                            // Generic struct literal: Box<Number> { value: 42 }
                            if let AstNode::Ident { name: struct_name, .. } = expr {
                                let span = self.current_span();
                                self.advance(); // consume {
                                self.skip_newlines();  // Skip newlines after opening brace

                                let mut fields = Vec::new();
                                if !matches!(self.current(), Token::RightBrace) {
                                    loop {
                                        let field_name = match self.current() {
                                            Token::Ident(n) => n.clone(),
                                            _ => {
                                                return Err(ParseError {
                                                    message: "Expected field name in struct literal".to_string(),
                                                    position: self.position,
                                                })
                                            }
                                        };
                                        self.advance();
                                        self.expect(Token::Colon)?;
                                        let field_value = self.parse_expression()?;
                                        fields.push((field_name, field_value));

                                        if !self.match_token(Token::Comma) {
                                            break;
                                        }
                                        self.skip_newlines();  // Skip newlines after comma
                                    }
                                }

                                self.skip_newlines();  // Skip newlines before closing brace
                                self.expect(Token::RightBrace)?;
                                expr = AstNode::StructLiteral {
                                    struct_name,
                                    type_args,
                                    fields,
                                    span,
                                };
                            } else {
                                return Err(ParseError {
                                    message: "Type arguments can only be used with identifiers".to_string(),
                                    position: self.position,
                                });
                            }
                        }
                        _ => {
                            return Err(ParseError {
                                message: "Expected '(' or '{' after type arguments".to_string(),
                                position: self.position,
                            });
                        }
                    }
                }
                Token::LeftParen => {
                    // Non-generic function call
                    let span = self.current_span();
                    self.advance();
                    let mut args = Vec::new();

                    if !matches!(self.current(), Token::RightParen) {
                        loop {
                            args.push(self.parse_expression()?);
                            if !self.match_token(Token::Comma) {
                                break;
                            }
                        }
                    }

                    self.expect(Token::RightParen)?;
                    expr = AstNode::Call {
                        callee: Box::new(expr),
                        type_args: Vec::new(), // No type arguments
                        args,
                        span,
                    };
                }
                Token::LeftBracket => {
                    let span = self.current_span();
                    self.advance();
                    let index = Box::new(self.parse_expression()?);
                    self.expect(Token::RightBracket)?;
                    expr = AstNode::IndexAccess {
                        object: Box::new(expr),
                        index,
                        span,
                    };
                }
                Token::LeftBrace => {
                    // Struct literal: Person { name: "Alice", age: 30 }
                    // Only valid if expr is an identifier
                    if let AstNode::Ident { name: struct_name, .. } = expr {
                        let span = self.current_span();
                        self.advance(); // consume '{'
                        self.skip_newlines();  // Skip newlines after opening brace

                        let mut fields = Vec::new();
                        if !matches!(self.current(), Token::RightBrace) {
                            loop {
                                // Parse field: name: value
                                let field_name = match self.current() {
                                    Token::Ident(n) => n.clone(),
                                    _ => {
                                        return Err(ParseError {
                                            message: "Expected field name in struct literal".to_string(),
                                            position: self.position,
                                        })
                                    }
                                };
                                self.advance();

                                self.expect(Token::Colon)?;

                                let field_value = self.parse_expression()?;
                                fields.push((field_name, field_value));

                                if !self.match_token(Token::Comma) {
                                    break;
                                }
                                self.skip_newlines();  // Skip newlines after comma
                            }
                        }

                        self.skip_newlines();  // Skip newlines before closing brace
                        self.expect(Token::RightBrace)?;
                        expr = AstNode::StructLiteral {
                            struct_name,
                            type_args: Vec::new(), // No type arguments
                            fields,
                            span,
                        };
                    } else {
                        // Not a struct literal, could be a map literal
                        break;
                    }
                }
                Token::Question => {
                    // Try operator: expr?
                    let span = self.current_span();
                    self.advance();
                    expr = AstNode::Try {
                        expr: Box::new(expr),
                        span,
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    /// Parse primary expression
    fn parse_primary(&mut self) -> ParseResult<AstNode> {
        match self.current().clone() {
            Token::Number(n) => {
                let span = self.current_span();
                self.advance();
                Ok(AstNode::Number { value: n, span })
            }
            Token::Text(s) => {
                let span = self.current_span();
                self.advance();
                Ok(AstNode::Text { value: s, span })
            }
            Token::Truth(b) => {
                let span = self.current_span();
                self.advance();
                Ok(AstNode::Truth { value: b, span })
            }
            Token::Nothing => {
                let span = self.current_span();
                self.advance();
                Ok(AstNode::Nothing { span })
            }
            Token::Ident(name) => {
                let span = self.current_span();
                self.advance();
                Ok(AstNode::Ident { name, span })
            }
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(Token::RightParen)?;
                Ok(expr)
            }
            Token::LeftBracket => self.parse_list(),
            Token::LeftBrace => self.parse_map(),
            Token::Seek => self.parse_seek(),
            Token::Range => self.parse_range(),

            // Enum constructors
            Token::Triumph => {
                let span = self.current_span();
                self.advance();
                self.expect(Token::LeftParen)?;
                let value = Box::new(self.parse_expression()?);
                self.expect(Token::RightParen)?;
                Ok(AstNode::Triumph { value, span })
            }
            Token::Mishap => {
                let span = self.current_span();
                self.advance();
                self.expect(Token::LeftParen)?;
                let value = Box::new(self.parse_expression()?);
                self.expect(Token::RightParen)?;
                Ok(AstNode::Mishap { value, span })
            }
            Token::Present => {
                let span = self.current_span();
                self.advance();
                self.expect(Token::LeftParen)?;
                let value = Box::new(self.parse_expression()?);
                self.expect(Token::RightParen)?;
                Ok(AstNode::Present { value, span })
            }
            Token::Absent => {
                let span = self.current_span();
                self.advance();
                Ok(AstNode::Absent { span })
            }

            // Capability request can be used as an expression
            Token::Request => self.parse_request(),

            _ => Err(ParseError {
                message: alloc::format!("Unexpected token: {:?}", self.current()),
                position: self.position,
            }),
        }
    }

    /// Parse list: [1, 2, 3]
    fn parse_list(&mut self) -> ParseResult<AstNode> {
        let span = self.current_span();
        self.expect(Token::LeftBracket)?;
        self.skip_newlines();  // Skip newlines after opening bracket

        let mut elements = Vec::new();
        if !matches!(self.current(), Token::RightBracket) {
            loop {
                elements.push(self.parse_expression()?);
                if !self.match_token(Token::Comma) {
                    break;
                }
                self.skip_newlines();  // Skip newlines after comma
            }
        }

        self.skip_newlines();  // Skip newlines before closing bracket
        self.expect(Token::RightBracket)?;
        Ok(AstNode::List { elements, span })
    }

    /// Parse map: {name: "Elara", age: 42}
    fn parse_map(&mut self) -> ParseResult<AstNode> {
        let span = self.current_span();
        self.expect(Token::LeftBrace)?;
        self.skip_newlines();  // Skip newlines after opening brace

        let mut pairs = Vec::new();
        if !matches!(self.current(), Token::RightBrace) {
            loop {
                let key = match self.current() {
                    Token::Ident(k) => k.clone(),
                    _ => {
                        return Err(ParseError {
                            message: "Expected identifier as map key".to_string(),
                            position: self.position,
                        })
                    }
                };
                self.advance();

                self.expect(Token::Colon)?;

                let value = self.parse_expression()?;
                pairs.push((key, value));

                if !self.match_token(Token::Comma) {
                    break;
                }
                self.skip_newlines();  // Skip newlines after comma
            }
        }

        self.skip_newlines();  // Skip newlines before closing brace
        self.expect(Token::RightBrace)?;
        Ok(AstNode::Map { entries: pairs, span })
    }

    /// Parse seek expression
    fn parse_seek(&mut self) -> ParseResult<AstNode> {
        let span = self.current_span();
        self.expect(Token::Seek)?;
        self.expect(Token::Where)?;

        let mut conditions = Vec::new();

        // Parse conditions
        while let Token::Ident(field) = self.current() {
            let field = field.clone();
            self.advance();

            let operator = match self.current() {
                Token::Is => QueryOperator::Is,
                Token::IsNot => QueryOperator::IsNot,
                Token::GreaterThan => QueryOperator::Greater,
                Token::LessThan => QueryOperator::Less,
                Token::AtLeast => QueryOperator::GreaterEq,
                Token::AtMost => QueryOperator::LessEq,
                Token::After => QueryOperator::After,
                Token::Before => QueryOperator::Before,
                _ => {
                    return Err(ParseError {
                        message: "Expected comparison operator".to_string(),
                        position: self.position,
                    })
                }
            };
            self.advance();

            let value = Box::new(self.parse_additive()?);

            conditions.push(QueryCondition {
                field,
                operator,
                value,
            });

            if !self.match_token(Token::And) {
                break;
            }
        }

        Ok(AstNode::SeekExpr { conditions, span })
    }

    /// Parse range: range(1, 10)
    fn parse_range(&mut self) -> ParseResult<AstNode> {
        let span = self.current_span();
        self.expect(Token::Range)?;
        self.expect(Token::LeftParen)?;

        let start = Box::new(self.parse_expression()?);
        self.expect(Token::Comma)?;
        let end = Box::new(self.parse_expression()?);

        self.expect(Token::RightParen)?;

        Ok(AstNode::Range { start, end, span })
    }

    /// Parse type annotation: Number, Text, List<Number>, Map, etc.
    fn parse_type_annotation(&mut self) -> ParseResult<TypeAnnotation> {
        match self.current() {
            Token::Ident(type_name) => {
                let name = type_name.clone();
                self.advance();

                // Check for parametrized type syntax: Box<T>, Pair<T, U>, List<Number>
                if matches!(self.current(), Token::LeftAngle) {
                    self.advance(); // consume <

                    // Parse type arguments
                    let mut type_args = Vec::new();
                    loop {
                        type_args.push(self.parse_type_annotation()?);

                        if matches!(self.current(), Token::Comma) {
                            self.advance(); // consume comma
                        } else {
                            break;
                        }
                    }

                    self.expect(Token::RightAngle)?;

                    // Special case for List to maintain backward compatibility
                    if name == "List" && type_args.len() == 1 {
                        Ok(TypeAnnotation::List(Box::new(type_args.into_iter().next().unwrap())))
                    } else {
                        Ok(TypeAnnotation::Parametrized {
                            name,
                            type_args,
                        })
                    }
                } else if name == "Map" {
                    Ok(TypeAnnotation::Map)
                } else {
                    // Simple type: could be Named (Number, Text) or Generic (T, U)
                    // For now, treat single uppercase letters as generic type parameters
                    // The semantic analyzer will determine the actual meaning based on scope
                    if name.len() == 1 && name.chars().next().unwrap().is_uppercase() {
                        Ok(TypeAnnotation::Generic(name))
                    } else {
                        Ok(TypeAnnotation::Named(name))
                    }
                }
            }
            _ => Err(ParseError {
                message: "Expected type name".to_string(),
                position: self.position,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_single_statement(source: &str) -> Result<AstNode, ParseError> {
        let mut lexer = crate::lexer::Lexer::new(source);
        let tokens = lexer.tokenize_positioned();
        let mut parser = Parser::new(tokens);
        parser.parse_statement()
    }

    // === Module Declaration Tests ===

    #[test]
    fn test_parse_module_decl_simple() {
        let source = r#"
grove Math with
    chant add(a, b) then
        yield a + b
    end
    offer add
end
        "#;

        let result = parse_single_statement(source);
        assert!(result.is_ok(), "Failed to parse module declaration: {:?}", result);

        if let Ok(AstNode::ModuleDecl { name, body, exports, .. }) = result {
            assert_eq!(name, "Math");
            assert_eq!(body.len(), 2); // chant def + export
            assert_eq!(exports.len(), 1);
            assert_eq!(exports[0], "add");
        } else {
            panic!("Expected ModuleDecl, got: {:?}", result);
        }
    }

    #[test]
    fn test_parse_module_decl_empty() {
        let source = r#"
grove Empty with
end
        "#;

        let result = parse_single_statement(source);
        assert!(result.is_ok(), "Failed to parse empty module: {:?}", result);

        if let Ok(AstNode::ModuleDecl { name, body, exports, .. }) = result {
            assert_eq!(name, "Empty");
            assert_eq!(body.len(), 0);
            assert_eq!(exports.len(), 0);
        } else {
            panic!("Expected ModuleDecl, got: {:?}", result);
        }
    }

    // === Import Tests ===

    #[test]
    fn test_parse_summon_with_path() {
        let source = r#"summon Math from "std/math.gw""#;

        let result = parse_single_statement(source);
        assert!(result.is_ok(), "Failed to parse summon statement: {:?}", result);

        if let Ok(AstNode::Import { module_name, path, items, alias, .. }) = result {
            assert_eq!(module_name, "Math"); // Module name from "summon Math"
            assert_eq!(path, "std/math.gw");
            assert!(items.is_none());
            assert!(alias.is_none());
        } else {
            panic!("Expected Import, got: {:?}", result);
        }
    }

    #[test]
    fn test_parse_summon_with_alias() {
        let source = r#"summon Math from "std/math.gw" as M"#;

        let result = parse_single_statement(source);
        assert!(result.is_ok(), "Failed to parse summon with alias: {:?}", result);

        if let Ok(AstNode::Import { module_name, path, items, alias, .. }) = result {
            assert_eq!(module_name, "Math"); // Module name from "summon Math"
            assert_eq!(path, "std/math.gw");
            assert!(items.is_none());
            assert_eq!(alias, Some("M".to_string()));
        } else {
            panic!("Expected Import, got: {:?}", result);
        }
    }

    #[test]
    fn test_parse_gather_specific_items() {
        let source = r#"gather sqrt, pow from Math"#;

        let result = parse_single_statement(source);
        assert!(result.is_ok(), "Failed to parse gather statement: {:?}", result);

        if let Ok(AstNode::Import { module_name, path, items, alias, .. }) = result {
            assert_eq!(module_name, "Math");
            assert_eq!(path, "Math.gw");
            assert_eq!(items, Some(vec!["sqrt".to_string(), "pow".to_string()]));
            assert!(alias.is_none());
        } else {
            panic!("Expected Import, got: {:?}", result);
        }
    }

    // === Export Tests ===

    #[test]
    fn test_parse_export_single_item() {
        let source = r#"offer sqrt"#;

        let result = parse_single_statement(source);
        assert!(result.is_ok(), "Failed to parse export: {:?}", result);

        if let Ok(AstNode::Export { items, .. }) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0], "sqrt");
        } else {
            panic!("Expected Export, got: {:?}", result);
        }
    }

    #[test]
    fn test_parse_export_multiple_items() {
        let source = r#"offer sqrt, pow, abs"#;

        let result = parse_single_statement(source);
        assert!(result.is_ok(), "Failed to parse export: {:?}", result);

        if let Ok(AstNode::Export { items, .. }) = result {
            assert_eq!(items.len(), 3);
            assert_eq!(items, vec!["sqrt".to_string(), "pow".to_string(), "abs".to_string()]);
        } else {
            panic!("Expected Export, got: {:?}", result);
        }
    }

    // === Integration Test ===

    #[test]
    fn test_parse_complete_module() {
        let source = r#"
grove Math with
    chant sqrt(x) then
        yield x * 0.5
    end

    chant pow(base, exp) then
        yield base * exp
    end

    chant _helper() then
        yield 42
    end

    offer sqrt, pow
end
        "#;

        let result = parse_single_statement(source);
        assert!(result.is_ok(), "Failed to parse complete module: {:?}", result);

        if let Ok(AstNode::ModuleDecl { name, body, exports, .. }) = result {
            assert_eq!(name, "Math");
            assert_eq!(body.len(), 4); // 3 functions + 1 export statement
            assert_eq!(exports, vec!["sqrt".to_string(), "pow".to_string()]);

            // Verify first function is sqrt
            if let AstNode::ChantDef { name, ..  } = &body[0] {
                assert_eq!(name, "sqrt");
            } else {
                panic!("Expected ChantDef for sqrt");
            }

            // Verify export statement is last
            if let AstNode::Export { items, .. } = &body[3] {
                assert_eq!(items, &vec!["sqrt".to_string(), "pow".to_string()]);
            } else {
                panic!("Expected Export statement");
            }
        } else {
            panic!("Expected ModuleDecl, got: {:?}", result);
        }
    }
}
