//! Property and method parsing for classes and traits
//!
//! Handles parsing of class/trait properties and methods.

use super::StmtParser;
use crate::ast::{Method, Property, PropertyHook, PropertyHookBody, PropertyHookType, Visibility};

impl<'a> StmtParser<'a> {
    /// Parse visibility modifier (public, private, protected)
    pub fn parse_visibility(&mut self) -> Visibility {
        match &self.current().kind {
            crate::token::TokenKind::Public => {
                self.advance();
                Visibility::Public
            }
            crate::token::TokenKind::Protected => {
                self.advance();
                Visibility::Protected
            }
            crate::token::TokenKind::Private => {
                self.advance();
                Visibility::Private
            }
            _ => Visibility::Public,
        }
    }

    /// Parse class property (shared between class and trait)
    pub fn parse_property(&mut self, visibility: Visibility) -> Result<Property, String> {
        let name = if let crate::token::TokenKind::Variable(name) = &self.current().kind {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(format!(
                "Expected property name at line {}, column {}",
                self.current().line,
                self.current().column
            ));
        };

        if self.check(&crate::token::TokenKind::LeftBrace) {
            let hooks = self.parse_property_hooks()?;
            return Ok(Property {
                name,
                visibility,
                write_visibility: None,
                default: None,
                readonly: false,
                is_static: false,
                attributes: Vec::new(),
                hooks,
            });
        }

        let default = if self.check(&crate::token::TokenKind::Assign) {
            self.advance();
            Some(self.parse_expression(super::super::precedence::Precedence::None)?)
        } else {
            None
        };

        if self.check(&crate::token::TokenKind::Semicolon) {
            self.advance();
        }

        Ok(Property {
            name,
            visibility,
            write_visibility: None,
            default,
            readonly: false,
            is_static: false,
            attributes: Vec::new(),
            hooks: Vec::new(),
        })
    }

    /// Parse property hooks (PHP 8.4)
    fn parse_property_hooks(&mut self) -> Result<Vec<PropertyHook>, String> {
        if !self.check(&crate::token::TokenKind::LeftBrace) {
            return Err(format!(
                "Expected '{{' at line {}, column {}",
                self.current().line,
                self.current().column
            ));
        }
        self.advance();

        let mut hooks = Vec::new();

        while !self.check(&crate::token::TokenKind::RightBrace)
            && !self.check(&crate::token::TokenKind::Eof)
        {
            let hook_type = if self.check(&crate::token::TokenKind::Get) {
                self.advance();
                PropertyHookType::Get
            } else if self.check(&crate::token::TokenKind::Set) {
                self.advance();
                PropertyHookType::Set
            } else {
                return Err(format!(
                    "Expected 'get' or 'set' in property hook at line {}",
                    self.current().line
                ));
            };

            let body = if self.check(&crate::token::TokenKind::DoubleArrow) {
                self.advance();
                let expr = self.parse_expression(super::super::precedence::Precedence::None)?;
                if !self.check(&crate::token::TokenKind::Semicolon) {
                    return Err(format!(
                        "Expected ';' after property hook expression at line {}",
                        self.current().line
                    ));
                }
                self.advance();
                PropertyHookBody::Expression(Box::new(expr))
            } else if self.check(&crate::token::TokenKind::LeftBrace) {
                self.advance();
                let mut statements = Vec::new();

                while !self.check(&crate::token::TokenKind::RightBrace)
                    && !self.check(&crate::token::TokenKind::Eof)
                {
                    if let Some(stmt) = self.parse_statement()? {
                        statements.push(stmt);
                    }
                }

                if !self.check(&crate::token::TokenKind::RightBrace) {
                    return Err(format!(
                        "Expected '}}' after property hook block at line {}",
                        self.current().line
                    ));
                }
                self.advance();
                PropertyHookBody::Block(statements)
            } else {
                return Err(format!(
                    "Expected '=>' or '{{' after hook type at line {}",
                    self.current().line
                ));
            };

            hooks.push(PropertyHook { hook_type, body });
        }

        if !self.check(&crate::token::TokenKind::RightBrace) {
            return Err(format!(
                "Expected '}}' at line {}, column {}",
                self.current().line,
                self.current().column
            ));
        }
        self.advance();

        if hooks.is_empty() {
            return Err("Property hooks cannot be empty".to_string());
        }

        let get_count = hooks
            .iter()
            .filter(|h| matches!(h.hook_type, PropertyHookType::Get))
            .count();
        let set_count = hooks
            .iter()
            .filter(|h| matches!(h.hook_type, PropertyHookType::Set))
            .count();

        if get_count > 1 {
            return Err("Duplicate 'get' hook in property".to_string());
        }
        if set_count > 1 {
            return Err("Duplicate 'set' hook in property".to_string());
        }

        Ok(hooks)
    }

    /// Parse class method (shared between class and trait)
    pub fn parse_method(
        &mut self,
        visibility: Visibility,
        is_abstract_method: bool,
        is_final_method: bool,
    ) -> Result<Method, String> {
        self.advance();

        let name = if let crate::token::TokenKind::Identifier(name) = &self.current().kind {
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

        let is_constructor = name.to_lowercase() == "__construct";

        self.consume(
            crate::token::TokenKind::LeftParen,
            "Expected '(' after method name",
        )?;

        let mut params = Vec::new();
        let mut found_variadic = false;
        if !self.check(&crate::token::TokenKind::RightParen) {
            loop {
                let param_attributes = self.parse_attributes()?;

                let param_visibility = if is_constructor {
                    match &self.current().kind {
                        crate::token::TokenKind::Public => {
                            self.advance();
                            Some(Visibility::Public)
                        }
                        crate::token::TokenKind::Protected => {
                            self.advance();
                            Some(Visibility::Protected)
                        }
                        crate::token::TokenKind::Private => {
                            self.advance();
                            Some(Visibility::Private)
                        }
                        _ => None,
                    }
                } else {
                    match &self.current().kind {
                        crate::token::TokenKind::Public
                        | crate::token::TokenKind::Protected
                        | crate::token::TokenKind::Private => {
                            return Err(format!(
                                "Constructor property promotion can only be used in __construct at line {}, column {}",
                                self.current().line,
                                self.current().column
                            ));
                        }
                        _ => None,
                    }
                };

                let param_readonly = if is_constructor && param_visibility.is_some() {
                    if self.check(&crate::token::TokenKind::Readonly) {
                        self.advance();
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                let type_hint = if let crate::token::TokenKind::Identifier(_) = &self.current().kind
                {
                    Some(self.parse_type_hint()?)
                } else if self.check(&crate::token::TokenKind::QuestionMark) {
                    Some(self.parse_type_hint()?)
                } else if self.check(&crate::token::TokenKind::LeftParen) {
                    Some(self.parse_type_hint()?)
                } else {
                    None
                };

                let by_ref = if let crate::token::TokenKind::Identifier(s) = &self.current().kind {
                    if s == "&" {
                        self.advance();
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                let is_variadic = if self.check(&crate::token::TokenKind::Ellipsis) {
                    self.advance();
                    true
                } else {
                    false
                };

                let param_name =
                    if let crate::token::TokenKind::Variable(name) = &self.current().kind {
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

                let default = if self.check(&crate::token::TokenKind::Assign) {
                    if is_variadic {
                        return Err(format!(
                            "Variadic parameter cannot have a default value at line {}, column {}",
                            self.current().line,
                            self.current().column
                        ));
                    }
                    self.advance();
                    Some(self.parse_expression(super::super::precedence::Precedence::None)?)
                } else {
                    None
                };

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

                params.push(crate::ast::FunctionParam {
                    name: param_name,
                    type_hint,
                    default,
                    by_ref,
                    is_variadic,
                    visibility: param_visibility,
                    readonly: param_readonly,
                    attributes: param_attributes,
                });

                if !self.check(&crate::token::TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }

        self.consume(
            crate::token::TokenKind::RightParen,
            "Expected ')' after parameters",
        )?;

        let return_type = if self.check(&crate::token::TokenKind::Colon) {
            self.advance();
            Some(self.parse_type_hint()?)
        } else {
            None
        };

        let body = if is_abstract_method {
            self.consume(
                crate::token::TokenKind::Semicolon,
                "Expected ';' after abstract method declaration",
            )?;
            Vec::new()
        } else {
            self.consume(
                crate::token::TokenKind::LeftBrace,
                "Expected '{' before method body",
            )?;

            let mut body = Vec::new();
            while !self.check(&crate::token::TokenKind::RightBrace)
                && !self.check(&crate::token::TokenKind::Eof)
            {
                if let Some(stmt) = self.parse_statement()? {
                    body.push(stmt);
                }
            }

            self.consume(
                crate::token::TokenKind::RightBrace,
                "Expected '}' after method body",
            )?;
            body
        };

        Ok(Method {
            name,
            visibility,
            is_static: false,
            is_abstract: is_abstract_method,
            is_final: is_final_method,
            params,
            return_type,
            body,
            attributes: Vec::new(),
        })
    }
}
