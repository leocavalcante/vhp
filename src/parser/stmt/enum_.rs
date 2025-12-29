//! Enum definition parsing
//!
//! Handles parsing of enum declarations including:
//! - Enum declaration with backing type
//! - Enum cases (pure and backed)
//! - Enum methods

use super::super::precedence::Precedence;
use super::StmtParser;
use crate::ast::{EnumBackingType, EnumCase, FunctionParam, Method, Stmt, Visibility};
use crate::token::TokenKind;

impl<'a> StmtParser<'a> {
    /// Parse enum declaration: enum Name: type { case Value; case Value = expr; ... }
    pub fn parse_enum(&mut self) -> Result<Stmt, String> {
        self.consume(TokenKind::Enum, "Expected 'enum' keyword")?;

        // Parse enum name
        let name = if let TokenKind::Identifier(id) = &self.current().kind {
            let name = id.clone();
            self.advance();
            name
        } else {
            return Err(format!(
                "Expected enum name at line {}, column {}",
                self.current().line,
                self.current().column
            ));
        };

        // Check for backing type (: int or : string)
        let backing_type = if self.check(&TokenKind::Colon) {
            self.advance(); // consume ':'

            if let TokenKind::Identifier(type_name) = &self.current().kind {
                let type_lower = type_name.to_lowercase();
                let backing = match type_lower.as_str() {
                    "int" => EnumBackingType::Int,
                    "string" => EnumBackingType::String,
                    _ => {
                        return Err(format!(
                            "Invalid enum backing type '{}'. Only 'int' and 'string' are supported at line {}, column {}",
                            type_name,
                            self.current().line,
                            self.current().column
                        ));
                    }
                };
                self.advance(); // consume type name
                backing
            } else {
                return Err(format!(
                    "Expected backing type (int or string) at line {}, column {}",
                    self.current().line,
                    self.current().column
                ));
            }
        } else {
            EnumBackingType::None
        };

        self.consume(TokenKind::LeftBrace, "Expected '{' after enum name")?;

        let mut cases = Vec::new();
        let mut methods = Vec::new();

        // Parse cases and methods
        while !self.check(&TokenKind::RightBrace) && !self.check(&TokenKind::Eof) {
            // Check for case or method
            if self.check(&TokenKind::Case) {
                // Parse enum case
                self.advance(); // consume 'case'

                let case_name = if let TokenKind::Identifier(id) = &self.current().kind {
                    let name = id.clone();
                    self.advance();
                    name
                } else {
                    return Err(format!(
                        "Expected case name at line {}, column {}",
                        self.current().line,
                        self.current().column
                    ));
                };

                // Check for value assignment (backed enums)
                let value = if self.check(&TokenKind::Assign) {
                    if backing_type == EnumBackingType::None {
                        return Err(format!(
                            "Pure enum cannot have case values at line {}, column {}",
                            self.current().line,
                            self.current().column
                        ));
                    }
                    self.advance(); // consume '='
                    Some(self.parse_expression(Precedence::None)?)
                } else {
                    if backing_type != EnumBackingType::None {
                        return Err(format!(
                            "Backed enum must have case values at line {}, column {}",
                            self.current().line,
                            self.current().column
                        ));
                    }
                    None
                };

                self.consume(TokenKind::Semicolon, "Expected ';' after case declaration")?;

                cases.push(EnumCase {
                    name: case_name,
                    value,
                });
            } else if self.check(&TokenKind::Public)
                || self.check(&TokenKind::Private)
                || self.check(&TokenKind::Protected)
            {
                // Parse method (enums can have methods)
                let visibility = match &self.current().kind {
                    TokenKind::Public => Visibility::Public,
                    TokenKind::Private => Visibility::Private,
                    TokenKind::Protected => Visibility::Protected,
                    _ => unreachable!(),
                };
                self.advance();

                self.consume(
                    TokenKind::Function,
                    "Expected 'function' after visibility modifier in enum",
                )?;

                let method_name = if let TokenKind::Identifier(id) = &self.current().kind {
                    let name = id.clone();
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
                        // Parse parameter
                        let param_name = if let TokenKind::Variable(var) = &self.current().kind {
                            let name = var.clone();
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
                            by_ref: false,
                            visibility: None,
                            readonly: false,
                            attributes: Vec::new(),
                        });

                        if !self.check(&TokenKind::Comma) {
                            break;
                        }
                        self.advance(); // consume ','
                    }
                }

                self.consume(TokenKind::RightParen, "Expected ')' after parameters")?;
                self.consume(TokenKind::LeftBrace, "Expected '{' before method body")?;

                let body = self.parse_block()?;

                methods.push(Method {
                    name: method_name,
                    visibility,
                    is_static: false,
                    is_abstract: false,
                    is_final: false,
                    params,
                    body,
                    attributes: Vec::new(),
                });
            } else {
                return Err(format!(
                    "Expected 'case' or method declaration in enum at line {}, column {}",
                    self.current().line,
                    self.current().column
                ));
            }
        }

        self.consume(TokenKind::RightBrace, "Expected '}' after enum body")?;

        if cases.is_empty() {
            return Err(format!("Enum '{}' must have at least one case", name));
        }

        Ok(Stmt::Enum {
            name,
            backing_type,
            cases,
            methods,
            attributes: Vec::new(),
        })
    }
}
