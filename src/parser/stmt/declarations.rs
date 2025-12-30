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
        let mut found_variadic = false;
        if !self.check(&TokenKind::RightParen) {
            loop {
                // Parse attributes for this parameter
                let param_attributes = self.parse_attributes()?;

                // Parse type hint if present
                let type_hint = if let TokenKind::Identifier(_) = &self.current().kind {
                    // Check if this looks like a type (not preceded by $)
                    Some(self.parse_type_hint()?)
                } else if self.check(&TokenKind::QuestionMark) {
                    // Nullable type
                    Some(self.parse_type_hint()?)
                } else {
                    None
                };

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

                // Check for variadic: ...
                let is_variadic = if self.check(&TokenKind::Ellipsis) {
                    self.advance();
                    true
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
                    if is_variadic {
                        return Err(format!(
                            "Variadic parameter cannot have a default value at line {}, column {}",
                            self.current().line,
                            self.current().column
                        ));
                    }
                    self.advance();
                    Some(self.parse_expression(Precedence::None)?)
                } else {
                    None
                };

                // Variadic must be the last parameter
                if found_variadic {
                    return Err(format!(
                        "Only the last parameter can be variadic at line {}, column {}",
                        self.current().line,
                        self.current().column
                    ));
                }
                if is_variadic {
                    found_variadic = true;
                }

                params.push(FunctionParam {
                    name: param_name,
                    type_hint,
                    default,
                    by_ref,
                    is_variadic,
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

        // Parse return type hint if present (after : )
        let return_type = if self.check(&TokenKind::Colon) {
            self.advance();
            Some(self.parse_type_hint()?)
        } else {
            None
        };

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
            return_type,
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
