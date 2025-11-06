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
use crate::token::Token;

/// Parser for Glimmer-Weave source code
pub struct Parser {
    tokens: Vec<Token>,
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
    /// Create a new parser from a vector of tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, position: 0 }
    }

    /// Get current token
    fn current(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or(&Token::Eof)
    }

    /// Peek at next token
    fn peek(&self) -> &Token {
        self.tokens.get(self.position + 1).unwrap_or(&Token::Eof)
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
            Token::Yield => self.parse_yield(),
            Token::Break => self.parse_break(),
            Token::Continue => self.parse_continue(),
            Token::Match => self.parse_match(),
            Token::Attempt => self.parse_attempt(),
            Token::Request => self.parse_request(),
            _ => {
                // Try expression statement
                let expr = self.parse_expression()?;
                Ok(AstNode::ExprStmt(Box::new(expr)))
            }
        }
    }

    /// Parse: bind x to 42  OR  bind x: Number to 42
    fn parse_bind(&mut self) -> ParseResult<AstNode> {
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

        Ok(AstNode::BindStmt { name, typ, value })
    }

    /// Parse: weave counter as 0  OR  weave counter: Number as 0
    fn parse_weave(&mut self) -> ParseResult<AstNode> {
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

        Ok(AstNode::WeaveStmt { name, typ, value })
    }

    /// Parse: set counter to 10
    fn parse_set(&mut self) -> ParseResult<AstNode> {
        self.expect(Token::Set)?;

        let name = match self.current() {
            Token::Ident(n) => n.clone(),
            _ => {
                return Err(ParseError {
                    message: "Expected identifier after 'set'".to_string(),
                    position: self.position,
                })
            }
        };
        self.advance();

        self.expect(Token::To)?;

        let value = Box::new(self.parse_expression()?);

        Ok(AstNode::SetStmt { name, value })
    }

    /// Parse: should x > 5 then ... otherwise ... end
    fn parse_if(&mut self) -> ParseResult<AstNode> {
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
        })
    }

    /// Parse: for each x in list then ... end
    fn parse_for(&mut self) -> ParseResult<AstNode> {
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
        })
    }

    /// Parse: whilst condition then ... end
    fn parse_while(&mut self) -> ParseResult<AstNode> {
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
        })
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

        let mut params = Vec::new();
        if !matches!(self.current(), Token::RightParen) {
            loop {
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

                // Check for optional type annotation: ': Type'
                let param_type = if self.match_token(Token::Colon) {
                    Some(self.parse_type_annotation()?)
                } else {
                    None
                };

                params.push(Parameter {
                    name: param_name,
                    typ: param_type,
                });

                if !self.match_token(Token::Comma) {
                    break;
                }
            }
        }

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
        })
    }

    /// Parse: yield result
    fn parse_yield(&mut self) -> ParseResult<AstNode> {
        self.expect(Token::Yield)?;

        let value = Box::new(self.parse_expression()?);

        Ok(AstNode::YieldStmt { value })
    }

    /// Parse: break
    fn parse_break(&mut self) -> ParseResult<AstNode> {
        self.expect(Token::Break)?;
        Ok(AstNode::Break)
    }

    /// Parse: continue
    fn parse_continue(&mut self) -> ParseResult<AstNode> {
        self.expect(Token::Continue)?;
        Ok(AstNode::Continue)
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

        Ok(AstNode::MatchStmt { value, arms })
    }

    /// Parse pattern for match
    fn parse_pattern(&mut self) -> ParseResult<Pattern> {
        match self.current() {
            Token::Number(n) => {
                let val = *n;
                self.advance();
                Ok(Pattern::Literal(AstNode::Number(val)))
            }
            Token::Text(s) => {
                let val = s.clone();
                self.advance();
                Ok(Pattern::Literal(AstNode::Text(val)))
            }
            Token::Truth(b) => {
                let val = *b;
                self.advance();
                Ok(Pattern::Literal(AstNode::Truth(val)))
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
                                AstNode::List(
                                    inner_patterns.into_iter()
                                        .map(|p| match p {
                                            Pattern::Ident(name) => AstNode::Ident(name),
                                            _ => AstNode::Nothing, // Placeholder
                                        })
                                        .collect()
                                )
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

        Ok(AstNode::AttemptStmt { body, handlers })
    }

    /// Parse: request VGA.write with justification "message"
    fn parse_request(&mut self) -> ParseResult<AstNode> {
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
        })
    }

    /// Parse an expression
    fn parse_expression(&mut self) -> ParseResult<AstNode> {
        self.parse_pipeline()
    }

    /// Parse pipeline: x | filter | sort
    fn parse_pipeline(&mut self) -> ParseResult<AstNode> {
        let mut expr = self.parse_logical_or()?;

        if matches!(self.current(), Token::Pipe) {
            let mut stages = Vec::new();
            stages.push(expr);

            while self.match_token(Token::Pipe) {
                stages.push(self.parse_logical_or()?);
            }

            expr = AstNode::Pipeline { stages };
        }

        Ok(expr)
    }

    /// Parse logical OR: a or b
    fn parse_logical_or(&mut self) -> ParseResult<AstNode> {
        let mut left = self.parse_logical_and()?;

        while self.match_token(Token::Or) {
            let right = self.parse_logical_and()?;
            left = AstNode::BinaryOp {
                left: Box::new(left),
                op: BinaryOperator::Or,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse logical AND: a and b
    fn parse_logical_and(&mut self) -> ParseResult<AstNode> {
        let mut left = self.parse_comparison()?;

        while self.match_token(Token::And) {
            let right = self.parse_comparison()?;
            left = AstNode::BinaryOp {
                left: Box::new(left),
                op: BinaryOperator::And,
                right: Box::new(right),
            };
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

            self.advance();
            let right = self.parse_additive()?;
            left = AstNode::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
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

            self.advance();
            let right = self.parse_multiplicative()?;
            left = AstNode::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
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

            self.advance();
            let right = self.parse_unary()?;
            left = AstNode::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse unary: not x, -y
    fn parse_unary(&mut self) -> ParseResult<AstNode> {
        match self.current() {
            Token::Not => {
                self.advance();
                Ok(AstNode::UnaryOp {
                    op: UnaryOperator::Not,
                    operand: Box::new(self.parse_unary()?),
                })
            }
            Token::Minus => {
                self.advance();
                Ok(AstNode::UnaryOp {
                    op: UnaryOperator::Negate,
                    operand: Box::new(self.parse_unary()?),
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
                            };
                        }
                        Token::LeftBrace => {
                            // Generic struct literal: Box<Number> { value: 42 }
                            if let AstNode::Ident(struct_name) = expr {
                                self.advance(); // consume {

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
                                    }
                                }

                                self.expect(Token::RightBrace)?;
                                expr = AstNode::StructLiteral {
                                    struct_name,
                                    type_args,
                                    fields,
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
                    };
                }
                Token::LeftBracket => {
                    self.advance();
                    let index = Box::new(self.parse_expression()?);
                    self.expect(Token::RightBracket)?;
                    expr = AstNode::IndexAccess {
                        object: Box::new(expr),
                        index,
                    };
                }
                Token::LeftBrace => {
                    // Struct literal: Person { name: "Alice", age: 30 }
                    // Only valid if expr is an identifier
                    if let AstNode::Ident(struct_name) = expr {
                        self.advance(); // consume '{'

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
                            }
                        }

                        self.expect(Token::RightBrace)?;
                        expr = AstNode::StructLiteral {
                            struct_name,
                            type_args: Vec::new(), // No type arguments
                            fields,
                        };
                    } else {
                        // Not a struct literal, could be a map literal
                        break;
                    }
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
                self.advance();
                Ok(AstNode::Number(n))
            }
            Token::Text(s) => {
                self.advance();
                Ok(AstNode::Text(s))
            }
            Token::Truth(b) => {
                self.advance();
                Ok(AstNode::Truth(b))
            }
            Token::Nothing => {
                self.advance();
                Ok(AstNode::Nothing)
            }
            Token::Ident(name) => {
                self.advance();
                Ok(AstNode::Ident(name))
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
                self.advance();
                self.expect(Token::LeftParen)?;
                let value = Box::new(self.parse_expression()?);
                self.expect(Token::RightParen)?;
                Ok(AstNode::Triumph(value))
            }
            Token::Mishap => {
                self.advance();
                self.expect(Token::LeftParen)?;
                let value = Box::new(self.parse_expression()?);
                self.expect(Token::RightParen)?;
                Ok(AstNode::Mishap(value))
            }
            Token::Present => {
                self.advance();
                self.expect(Token::LeftParen)?;
                let value = Box::new(self.parse_expression()?);
                self.expect(Token::RightParen)?;
                Ok(AstNode::Present(value))
            }
            Token::Absent => {
                self.advance();
                Ok(AstNode::Absent)
            }

            _ => Err(ParseError {
                message: alloc::format!("Unexpected token: {:?}", self.current()),
                position: self.position,
            }),
        }
    }

    /// Parse list: [1, 2, 3]
    fn parse_list(&mut self) -> ParseResult<AstNode> {
        self.expect(Token::LeftBracket)?;

        let mut elements = Vec::new();
        if !matches!(self.current(), Token::RightBracket) {
            loop {
                elements.push(self.parse_expression()?);
                if !self.match_token(Token::Comma) {
                    break;
                }
            }
        }

        self.expect(Token::RightBracket)?;
        Ok(AstNode::List(elements))
    }

    /// Parse map: {name: "Elara", age: 42}
    fn parse_map(&mut self) -> ParseResult<AstNode> {
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
        Ok(AstNode::Map(pairs))
    }

    /// Parse seek expression
    fn parse_seek(&mut self) -> ParseResult<AstNode> {
        self.expect(Token::Seek)?;
        self.expect(Token::Where)?;

        let mut conditions = Vec::new();

        // Parse conditions
        loop {
            let field = match self.current() {
                Token::Ident(f) => f.clone(),
                _ => break,
            };
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

        Ok(AstNode::SeekExpr { conditions })
    }

    /// Parse range: range(1, 10)
    fn parse_range(&mut self) -> ParseResult<AstNode> {
        self.expect(Token::Range)?;
        self.expect(Token::LeftParen)?;

        let start = Box::new(self.parse_expression()?);
        self.expect(Token::Comma)?;
        let end = Box::new(self.parse_expression()?);

        self.expect(Token::RightParen)?;

        Ok(AstNode::Range { start, end })
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
