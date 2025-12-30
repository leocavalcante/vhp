//! Interface definition parsing
//!
//! Handles parsing of interface declarations including:
//! - Interface declaration with extends
//! - Interface methods
//! - Interface constants

use super::super::precedence::Precedence;
use super::StmtParser;
use crate::ast::{FunctionParam, InterfaceConstant, InterfaceMethodSignature, Stmt};
use crate::token::TokenKind;

impl<'a> StmtParser<'a> {
    /// Parse interface declaration
    pub fn parse_interface(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'interface'

        let name = if let TokenKind::Identifier(name) = &self.current().kind {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(format!(
                "Expected interface name at line {}, column {}",
                self.current().line,
                self.current().column
            ));
        };

        // Check for extends
        let mut parents = Vec::new();
        if self.check(&TokenKind::Extends) {
            self.advance();
            loop {
                parents.push(self.parse_qualified_name()?);

                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }

        self.consume(TokenKind::LeftBrace, "Expected '{' after interface name")?;

        let mut methods = Vec::new();
        let mut constants = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.check(&TokenKind::Eof) {
            // Parse attributes that may precede method or constant
            let attributes = self.parse_attributes()?;

            // Skip optional visibility modifier (interface methods are always public)
            if self.check(&TokenKind::Public) || self.check(&TokenKind::Protected) || self.check(&TokenKind::Private) {
                self.advance();
            }

            // Check for 'const' keyword
            if self.check(&TokenKind::Const) {
                // Parse const with the attributes we already parsed
                self.advance(); // consume 'const'

                let name = if let TokenKind::Identifier(name) = &self.current().kind {
                    let name = name.clone();
                    self.advance();
                    name
                } else {
                    return Err(format!(
                        "Expected constant name at line {}, column {}",
                        self.current().line,
                        self.current().column
                    ));
                };

                self.consume(TokenKind::Assign, "Expected '=' after constant name")?;
                let value = self.parse_expression(Precedence::None)?;
                self.consume(TokenKind::Semicolon, "Expected ';' after constant value")?;

                constants.push(InterfaceConstant {
                    name,
                    value,
                    attributes,
                });
                continue;
            }

            if self.check(&TokenKind::Function) {
                let mut method = self.parse_interface_method()?;
                method.attributes = attributes;
                methods.push(method);
            } else {
                return Err(format!(
                    "Expected method or constant in interface at line {}, column {}",
                    self.current().line,
                    self.current().column
                ));
            }
        }

        self.consume(TokenKind::RightBrace, "Expected '}' after interface body")?;

        Ok(Stmt::Interface {
            name,
            parents,
            methods,
            constants,
            attributes: Vec::new(),
        })
    }

    /// Parse interface method signature (no body)
    fn parse_interface_method(&mut self) -> Result<InterfaceMethodSignature, String> {
        self.advance(); // consume 'function'

        let name = if let TokenKind::Identifier(name) = &self.current().kind {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(format!(
                "Expected method name at line {}, column {}",
                self.current().line,
                self.current().column
            ));
        };

        self.consume(TokenKind::LeftParen, "Expected '(' after method name")?;

        let mut params = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            loop {
                // Parse attributes for this parameter
                let param_attributes = self.parse_attributes()?;

                // Parse type hint if present
                let type_hint = if let TokenKind::Identifier(_) = &self.current().kind {
                    Some(self.parse_type_hint()?)
                } else if self.check(&TokenKind::QuestionMark) {
                    Some(self.parse_type_hint()?)
                } else if self.check(&TokenKind::LeftParen) {
                    // Parenthesized intersection or DNF type
                    Some(self.parse_type_hint()?)
                } else {
                    None
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
                    type_hint,
                    default,
                    by_ref: false,
                    is_variadic: false,
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

        // Parse return type hint if present (: type)
        let return_type = if self.check(&TokenKind::Colon) {
            self.advance();
            Some(self.parse_type_hint()?)
        } else {
            None
        };

        self.consume(TokenKind::Semicolon, "Expected ';' after method signature")?;

        Ok(InterfaceMethodSignature {
            name,
            params,
            return_type,
            attributes: Vec::new(),
        })
    }

    /// Parse interface constant (attributes are already parsed by caller)
    #[allow(dead_code)]
    fn parse_interface_constant(&mut self) -> Result<InterfaceConstant, String> {
        // consume 'const'
        self.advance();

        let name = if let TokenKind::Identifier(name) = &self.current().kind {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(format!(
                "Expected constant name at line {}, column {}",
                self.current().line,
                self.current().column
            ));
        };

        self.consume(TokenKind::Assign, "Expected '=' after constant name")?;
        let value = self.parse_expression(Precedence::None)?;
        self.consume(TokenKind::Semicolon, "Expected ';' after constant value")?;

        Ok(InterfaceConstant {
            name,
            value,
            attributes: Vec::new(),
        })
    }
}
