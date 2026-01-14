//! Callable expression parsing (functions, methods, classes)
//!
//! Handles parsing of function calls, static method calls, class instantiation,
//! and related expressions.

use super::{parse_postfix, ExprParser};
use crate::ast::{Argument, Expr};
use crate::token::TokenKind;

impl<'a> ExprParser<'a> {
    /// Parse function/method arguments with support for named arguments (PHP 8.0)
    /// Syntax: expr, expr, name: expr, name: expr
    /// Rule: positional arguments must come before named arguments
    pub(crate) fn parse_arguments(&mut self) -> Result<Vec<Argument>, String> {
        let mut args = Vec::new();
        let mut seen_named = false;

        if !self.check(&TokenKind::RightParen) {
            loop {
                let is_named = if let TokenKind::Identifier(_) = &self.current().kind {
                    *self.pos + 1 < self.tokens.len()
                        && std::mem::discriminant(&self.tokens[*self.pos + 1].kind)
                            == std::mem::discriminant(&TokenKind::Colon)
                } else {
                    false
                };

                if is_named {
                    let name = if let TokenKind::Identifier(n) = &self.current().kind {
                        n.clone()
                    } else {
                        unreachable!()
                    };
                    self.advance();
                    self.advance();
                    let value =
                        self.parse_expression(super::super::precedence::Precedence::None)?;
                    args.push(Argument {
                        name: Some(name),
                        value: Box::new(value),
                    });
                    seen_named = true;
                } else {
                    if seen_named {
                        return Err(format!(
                            "Positional arguments cannot follow named arguments at line {}, column {}",
                            self.current().line,
                            self.current().column
                        ));
                    }

                    if self.check(&TokenKind::Ellipsis) {
                        self.advance();

                        if self.check(&TokenKind::RightParen) || self.check(&TokenKind::Comma) {
                            args.push(Argument {
                                name: None,
                                value: Box::new(Expr::Placeholder),
                            });
                        } else {
                            let expr =
                                self.parse_expression(super::super::precedence::Precedence::None)?;
                            args.push(Argument {
                                name: None,
                                value: Box::new(Expr::Spread(Box::new(expr))),
                            });
                        }
                    } else {
                        let value =
                            self.parse_expression(super::super::precedence::Precedence::None)?;
                        args.push(Argument {
                            name: None,
                            value: Box::new(value),
                        });
                    }
                }

                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }

        Ok(args)
    }

    /// Parse function call
    pub(crate) fn parse_function_call(&mut self, name: String) -> Result<Expr, String> {
        self.advance();

        if self.check(&TokenKind::Ellipsis) {
            let start_pos = *self.pos;
            self.advance();

            if self.check(&TokenKind::RightParen) {
                self.advance();
                let callable = Expr::CallableFromFunction(name);
                return parse_postfix(self, callable);
            } else {
                *self.pos = start_pos;
            }
        }

        let args = self.parse_arguments()?;
        self.consume(
            TokenKind::RightParen,
            "Expected ')' after function arguments",
        )?;
        let call = Expr::FunctionCall { name, args };
        parse_postfix(self, call)
    }

    /// Parse static method call or enum case access
    pub(crate) fn parse_static_access(&mut self, class_name: String) -> Result<Expr, String> {
        self.advance();

        if let TokenKind::Variable(prop_name) = &self.current().kind {
            let prop_name = prop_name.clone();
            self.advance();
            let expr = Expr::StaticPropertyAccess {
                class: class_name,
                property: prop_name,
            };
            return parse_postfix(self, expr);
        }

        let method_or_case = if let TokenKind::Identifier(id) = &self.current().kind {
            let id = id.clone();
            self.advance();
            id
        } else if self.check(&TokenKind::From) {
            self.advance();
            "from".to_string()
        } else {
            return Err(format!(
                "Expected method, case name, or property after '::' at line {}, column {}",
                self.current().line,
                self.current().column
            ));
        };

        if self.check(&TokenKind::LeftParen) {
            self.advance();

            if class_name.to_lowercase() == "fiber" {
                match method_or_case.to_lowercase().as_str() {
                    "suspend" => {
                        let value = if self.check(&TokenKind::RightParen) {
                            None
                        } else {
                            Some(Box::new(self.parse_expression(
                                super::super::precedence::Precedence::None,
                            )?))
                        };
                        self.consume(
                            TokenKind::RightParen,
                            "Expected ')' after Fiber::suspend arguments",
                        )?;
                        let expr = Expr::FiberSuspend { value };
                        return parse_postfix(self, expr);
                    }
                    "getcurrent" => {
                        self.consume(
                            TokenKind::RightParen,
                            "Expected ')' after Fiber::getCurrent",
                        )?;
                        let expr = Expr::FiberGetCurrent;
                        return parse_postfix(self, expr);
                    }
                    _ => {}
                }
            }

            if self.check(&TokenKind::Ellipsis) {
                let start_pos = *self.pos;
                self.advance();

                if self.check(&TokenKind::RightParen) {
                    self.advance();
                    let callable = Expr::CallableFromStaticMethod {
                        class: class_name,
                        method: method_or_case,
                    };
                    return parse_postfix(self, callable);
                } else {
                    *self.pos = start_pos;
                }
            }

            let args = self.parse_arguments()?;
            self.consume(
                TokenKind::RightParen,
                "Expected ')' after static method arguments",
            )?;
            let call = Expr::StaticMethodCall {
                class_name,
                method: method_or_case,
                args,
            };
            parse_postfix(self, call)
        } else {
            let expr = Expr::EnumCase {
                enum_name: class_name,
                case_name: method_or_case,
            };
            parse_postfix(self, expr)
        }
    }

    /// Parse new object instantiation
    pub(crate) fn parse_new_object(&mut self) -> Result<Expr, String> {
        self.advance();

        if self.check(&TokenKind::Class) {
            let anon_class = self.parse_anonymous_class()?;
            return parse_postfix(self, anon_class);
        }

        let mut class_name_parts = Vec::new();
        let is_fully_qualified = if self.check(&TokenKind::Backslash) {
            self.advance();
            true
        } else {
            false
        };

        let class_name = match &self.current().kind {
            TokenKind::Identifier(name) => {
                class_name_parts.push(name.clone());
                self.advance();

                while self.check(&TokenKind::Backslash) {
                    self.advance();
                    if let TokenKind::Identifier(part) = &self.current().kind {
                        class_name_parts.push(part.clone());
                        self.advance();
                    } else {
                        return Err(format!(
                            "Expected identifier after '\\' at line {}, column {}",
                            self.current().line,
                            self.current().column
                        ));
                    }
                }

                if is_fully_qualified {
                    format!("\\{}", class_name_parts.join("\\"))
                } else {
                    class_name_parts.join("\\")
                }
            }
            TokenKind::Fiber => {
                self.advance();
                "Fiber".to_string()
            }
            _ => {
                return Err(format!(
                    "Expected class name after 'new' at line {}, column {}",
                    self.current().line,
                    self.current().column
                ));
            }
        };

        if class_name.to_lowercase() == "fiber" {
            self.consume(TokenKind::LeftParen, "Expected '(' after 'new Fiber'")?;
            let callback = self.parse_expression(super::super::precedence::Precedence::None)?;
            self.consume(TokenKind::RightParen, "Expected ')' after fiber callback")?;
            let fiber_expr = Expr::NewFiber {
                callback: Box::new(callback),
            };
            return parse_postfix(self, fiber_expr);
        }

        let mut args = Vec::new();
        if self.check(&TokenKind::LeftParen) {
            self.advance();
            args = self.parse_arguments()?;
            self.consume(
                TokenKind::RightParen,
                "Expected ')' after constructor arguments",
            )?;
        }

        let new_expr = Expr::New { class_name, args };
        parse_postfix(self, new_expr)
    }
}
