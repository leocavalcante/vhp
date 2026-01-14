//! Arrow function and anonymous class parsing
//!
//! Handles parsing of PHP arrow functions (fn) and anonymous classes.

use super::ExprParser;
use crate::ast::{Expr, Visibility};
use crate::token::TokenKind;

impl<'a> ExprParser<'a> {
    /// Parse arrow function: fn(params) => expression
    /// PHP 7.4+ feature for short closures
    pub(crate) fn parse_arrow_function(&mut self) -> Result<Expr, String> {
        self.consume(TokenKind::LeftParen, "Expected '(' after 'fn'")?;

        let mut params = Vec::new();

        if !self.check(&TokenKind::RightParen) {
            loop {
                let by_ref = if self.check(&TokenKind::And) {
                    self.advance();
                    true
                } else {
                    false
                };

                let is_variadic = if self.check(&TokenKind::Ellipsis) {
                    self.advance();
                    true
                } else {
                    false
                };

                let param_name = if let TokenKind::Variable(name) = &self.current().kind {
                    let n = name.clone();
                    self.advance();
                    n
                } else {
                    return Err(format!(
                        "Expected parameter name at line {}, column {}",
                        self.current().line,
                        self.current().column
                    ));
                };

                let default = if self.check(&TokenKind::Assign) {
                    self.advance();
                    Some(self.parse_expression(super::super::precedence::Precedence::None)?)
                } else {
                    None
                };

                params.push(crate::ast::FunctionParam {
                    name: param_name,
                    type_hint: None,
                    default,
                    by_ref,
                    is_variadic,
                    visibility: None,
                    readonly: false,
                    attributes: Vec::new(),
                });

                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }

        self.consume(TokenKind::RightParen, "Expected ')' after parameters")?;
        self.consume(
            TokenKind::DoubleArrow,
            "Expected '=>' after arrow function parameters",
        )?;

        let body = self.parse_expression(super::super::precedence::Precedence::None)?;

        Ok(Expr::ArrowFunction {
            params,
            body: Box::new(body),
        })
    }

    /// Parse anonymous class: new class(...) extends X implements Y { ... }
    pub(crate) fn parse_anonymous_class(&mut self) -> Result<Expr, String> {
        self.consume(TokenKind::Class, "Expected 'class'")?;

        let constructor_args = if self.check(&TokenKind::LeftParen) {
            self.advance();
            let args = self.parse_arguments()?;
            self.consume(
                TokenKind::RightParen,
                "Expected ')' after constructor arguments",
            )?;
            args
        } else {
            vec![]
        };

        let parent = if self.check(&TokenKind::Extends) {
            self.advance();
            if let TokenKind::Identifier(name) = &self.current().kind {
                let name = name.clone();
                self.advance();
                Some(name)
            } else {
                return Err(format!(
                    "Expected parent class name after 'extends' at line {}, column {}",
                    self.current().line,
                    self.current().column
                ));
            }
        } else {
            None
        };

        let mut interfaces = vec![];
        if self.check(&TokenKind::Implements) {
            self.advance();
            loop {
                if let TokenKind::Identifier(name) = &self.current().kind {
                    interfaces.push(name.clone());
                    self.advance();
                } else {
                    return Err(format!(
                        "Expected interface name at line {}, column {}",
                        self.current().line,
                        self.current().column
                    ));
                }

                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }

        self.consume(
            TokenKind::LeftBrace,
            "Expected '{' for anonymous class body",
        )?;

        let mut stmt_parser = crate::parser::stmt::StmtParser::new(self.tokens, self.pos);

        let mut traits = vec![];
        let mut properties = vec![];
        let mut methods = vec![];

        while !stmt_parser.check(&TokenKind::RightBrace) && !stmt_parser.check(&TokenKind::Eof) {
            if stmt_parser.check(&TokenKind::Use) {
                stmt_parser.advance();
                let mut trait_names = vec![];
                loop {
                    if let TokenKind::Identifier(name) = &stmt_parser.current().kind {
                        trait_names.push(name.clone());
                        stmt_parser.advance();
                    } else {
                        return Err(format!(
                            "Expected trait name at line {}, column {}",
                            stmt_parser.current().line,
                            stmt_parser.current().column
                        ));
                    }

                    if !stmt_parser.check(&TokenKind::Comma) {
                        break;
                    }
                    stmt_parser.advance();
                }
                stmt_parser.consume(TokenKind::Semicolon, "Expected ';' after trait use")?;
                traits.push(crate::ast::TraitUse {
                    traits: trait_names,
                    resolutions: vec![],
                });
            } else {
                let mut visibility = Visibility::Public;
                let mut is_abstract = false;
                let mut is_final = false;

                loop {
                    if stmt_parser.check(&TokenKind::Public) {
                        visibility = Visibility::Public;
                        stmt_parser.advance();
                    } else if stmt_parser.check(&TokenKind::Protected) {
                        visibility = Visibility::Protected;
                        stmt_parser.advance();
                    } else if stmt_parser.check(&TokenKind::Private) {
                        visibility = Visibility::Private;
                        stmt_parser.advance();
                    } else if stmt_parser.check(&TokenKind::Abstract) {
                        is_abstract = true;
                        stmt_parser.advance();
                    } else if stmt_parser.check(&TokenKind::Final) {
                        is_final = true;
                        stmt_parser.advance();
                    } else {
                        break;
                    }
                }

                if stmt_parser.check(&TokenKind::Function) {
                    let method = stmt_parser.parse_method(visibility, is_abstract, is_final)?;
                    methods.push(method);
                } else {
                    let property = stmt_parser.parse_property(visibility)?;
                    properties.push(property);
                }
            }
        }

        stmt_parser.consume(
            TokenKind::RightBrace,
            "Expected '}' after anonymous class body",
        )?;

        Ok(Expr::NewAnonymousClass {
            constructor_args,
            parent,
            interfaces,
            traits,
            properties,
            methods,
        })
    }
}
