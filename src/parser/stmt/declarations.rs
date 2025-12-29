//! Declaration statement parsing
//!
//! Handles parsing of function declarations and return statements.

use super::super::precedence::Precedence;
use super::StmtParser;
use crate::ast::{FunctionParam, Stmt};
use crate::token::TokenKind;

impl<'a> StmtParser<'a> {
    /// Parse function declaration
    pub fn parse_function(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'function'

        let name = if let TokenKind::Identifier(name) = &self.current().kind {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(format!(
                "Expected function name at line {}, column {}",
                self.current().line,
                self.current().column
            ));
        };

        self.consume(TokenKind::LeftParen, "Expected '(' after function name")?;

        let mut params = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            loop {
                // Parse attributes for this parameter
                let param_attributes = self.parse_attributes()?;

                // Skip type hints (not supported yet)
                if let TokenKind::Identifier(type_name) = &self.current().kind {
                    let type_lower = type_name.to_lowercase();
                    if matches!(
                        type_lower.as_str(),
                        "string"
                            | "int"
                            | "float"
                            | "bool"
                            | "array"
                            | "object"
                            | "mixed"
                            | "callable"
                            | "iterable"
                            | "void"
                            | "never"
                            | "true"
                            | "false"
                            | "null"
                            | "self"
                            | "parent"
                            | "static"
                    ) {
                        // Skip the type
                        self.advance();
                        // Handle array type brackets if present
                        if self.check(&TokenKind::LeftBracket) {
                            self.advance();
                            self.consume(TokenKind::RightBracket, "Expected ']' after array type")?;
                        }
                    }
                }

                let by_ref = if let TokenKind::Identifier(s) = &self.current().kind {
                    if s == "&" {
                        self.advance();
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                let param_name = if let TokenKind::Variable(name) = &self.current().kind {
                    let name = name.clone();
                    self.advance();
                    name
                } else {
                    return Err(format!(
                        "Expected parameter name at line {}, column {}",
                        self.current().line,
                        self.current().column
                    ));
                };

                let default = if self.check(&TokenKind::Assign) {
                    self.advance();
                    Some(self.parse_expression(Precedence::None)?)
                } else {
                    None
                };

                params.push(FunctionParam {
                    name: param_name,
                    default,
                    by_ref,
                    visibility: None,
                    readonly: false,
                    attributes: param_attributes,
                });

                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }

        self.consume(TokenKind::RightParen, "Expected ')' after parameters")?;
        self.consume(TokenKind::LeftBrace, "Expected '{' before function body")?;

        let mut body = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.check(&TokenKind::Eof) {
            if let Some(stmt) = self.parse_statement()? {
                body.push(stmt);
            }
        }

        self.consume(TokenKind::RightBrace, "Expected '}' after function body")?;

        Ok(Stmt::Function {
            name,
            params,
            body,
            attributes: Vec::new(),
        })
    }

    /// Parse return statement
    pub fn parse_return(&mut self) -> Result<Stmt, String> {
        self.advance();

        let value = if self.check(&TokenKind::Semicolon)
            || self.check(&TokenKind::CloseTag)
            || self.check(&TokenKind::Eof)
        {
            None
        } else {
            Some(self.parse_expression(Precedence::None)?)
        };

        if self.check(&TokenKind::Semicolon) {
            self.advance();
        }

        Ok(Stmt::Return(value))
    }
}
