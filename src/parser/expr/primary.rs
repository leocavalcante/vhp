//! Primary expression parsing
//!
//! Handles literals, variables, arrays, function calls, and basic expressions.
//! Primary expressions are the foundation for all higher-level expressions.

use super::{parse_postfix, parse_match, parse_clone, ExprParser};
use crate::ast::{Argument, ArrayElement, Expr};
use crate::token::TokenKind;

impl<'a> ExprParser<'a> {
    /// Parse function/method arguments with support for named arguments (PHP 8.0)
    /// Syntax: expr, expr, name: expr, name: expr
    /// Rule: positional arguments must come before named arguments
    pub fn parse_arguments(&mut self) -> Result<Vec<Argument>, String> {
        let mut args = Vec::new();
        let mut seen_named = false;

        if !self.check(&TokenKind::RightParen) {
            loop {
                // Try to detect named argument pattern: identifier ':'
                let is_named = if let TokenKind::Identifier(_) = &self.current().kind {
                    // Peek ahead to see if there's a colon
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
                    self.advance(); // consume identifier
                    self.advance(); // consume ':'
                    let value = self.parse_expression(super::super::precedence::Precedence::None)?;
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

                    // Check for placeholder ... (three dots)
                    if self.check(&TokenKind::Concat)
                        && *self.pos + 1 < self.tokens.len()
                        && std::mem::discriminant(&self.tokens[*self.pos + 1].kind)
                            == std::mem::discriminant(&TokenKind::Concat)
                        && *self.pos + 2 < self.tokens.len()
                        && std::mem::discriminant(&self.tokens[*self.pos + 2].kind)
                            == std::mem::discriminant(&TokenKind::Concat)
                    {
                        // This is a placeholder (...)
                        self.advance(); // consume first .
                        self.advance(); // consume second .
                        self.advance(); // consume third .
                        args.push(Argument {
                            name: None,
                            value: Box::new(Expr::Placeholder),
                        });
                    } else {
                        let value = self.parse_expression(super::super::precedence::Precedence::None)?;
                        args.push(Argument {
                            name: None,
                            value: Box::new(value),
                        });
                    }
                }

                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance(); // consume ','
            }
        }

        Ok(args)
    }

    /// Parse array literal: [elem1, elem2] or [key => value, ...]
    pub fn parse_array_literal(&mut self) -> Result<Expr, String> {
        self.advance(); // consume '['
        let mut elements = Vec::new();

        if !self.check(&TokenKind::RightBracket) {
            loop {
                // Parse first expression (could be key or value)
                let first = self.parse_expression(super::super::precedence::Precedence::None)?;

                // Check for => (key => value syntax)
                if self.check(&TokenKind::DoubleArrow) {
                    self.advance(); // consume '=>'
                    let value = self.parse_expression(super::super::precedence::Precedence::None)?;
                    elements.push(ArrayElement {
                        key: Some(Box::new(first)),
                        value: Box::new(value),
                    });
                } else {
                    // Just a value, no explicit key
                    elements.push(ArrayElement {
                        key: None,
                        value: Box::new(first),
                    });
                }

                // Check for comma or end
                if self.check(&TokenKind::Comma) {
                    self.advance();
                    // Allow trailing comma
                    if self.check(&TokenKind::RightBracket) {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        self.consume(TokenKind::RightBracket, "Expected ']' after array elements")?;
        Ok(Expr::Array(elements))
    }

    /// Parse primary expression (literals, variables, grouped expressions)
    pub fn parse_primary(&mut self) -> Result<Expr, String> {
        let token = self.current().clone();

        match &token.kind {
            TokenKind::Integer(n) => {
                let n = *n;
                self.advance();
                parse_postfix(self, Expr::Integer(n))
            }
            TokenKind::Float(n) => {
                let n = *n;
                self.advance();
                parse_postfix(self, Expr::Float(n))
            }
            TokenKind::String(s) => {
                let s = s.clone();
                self.advance();
                parse_postfix(self, Expr::String(s))
            }
            TokenKind::True => {
                self.advance();
                parse_postfix(self, Expr::Bool(true))
            }
            TokenKind::False => {
                self.advance();
                parse_postfix(self, Expr::Bool(false))
            }
            TokenKind::Null => {
                self.advance();
                parse_postfix(self, Expr::Null)
            }
            TokenKind::LeftBracket => {
                let arr = self.parse_array_literal()?;
                parse_postfix(self, arr)
            }
            TokenKind::Variable(name) => {
                let name = name.clone();
                self.advance();
                // $this is a special reference
                let expr = if name == "this" {
                    Expr::This
                } else {
                    Expr::Variable(name)
                };
                parse_postfix(self, expr)
            }
            TokenKind::LeftParen => {
                self.advance();
                let expr = self.parse_expression(super::super::precedence::Precedence::None)?;
                self.consume(TokenKind::RightParen, "Expected ')' after expression")?;
                let grouped = Expr::Grouped(Box::new(expr));
                parse_postfix(self, grouped)
            }
            // Unary operators in primary context
            TokenKind::Minus => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: crate::ast::UnaryOp::Neg,
                    expr: Box::new(expr),
                })
            }
            TokenKind::Not => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: crate::ast::UnaryOp::Not,
                    expr: Box::new(expr),
                })
            }
            TokenKind::Increment => {
                self.advance();
                if let TokenKind::Variable(name) = &self.current().kind {
                    let name = name.clone();
                    self.advance();
                    Ok(Expr::Unary {
                        op: crate::ast::UnaryOp::PreInc,
                        expr: Box::new(Expr::Variable(name)),
                    })
                } else {
                    Err(format!(
                        "Expected variable after '++' at line {}, column {}",
                        self.current().line,
                        self.current().column
                    ))
                }
            }
            TokenKind::Decrement => {
                self.advance();
                if let TokenKind::Variable(name) = &self.current().kind {
                    let name = name.clone();
                    self.advance();
                    Ok(Expr::Unary {
                        op: crate::ast::UnaryOp::PreDec,
                        expr: Box::new(Expr::Variable(name)),
                    })
                } else {
                    Err(format!(
                        "Expected variable after '--' at line {}, column {}",
                        self.current().line,
                        self.current().column
                    ))
                }
            }
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();

                // Check for static method call (ClassName::method()) or enum case access (EnumName::CASE)
                if self.check(&TokenKind::DoubleColon) {
                    self.advance(); // consume '::'
                    let method_or_case = if let TokenKind::Identifier(id) = &self.current().kind {
                        let id = id.clone();
                        self.advance();
                        id
                    } else {
                        return Err(format!(
                            "Expected method or case name after '::' at line {}, column {}",
                            self.current().line,
                            self.current().column
                        ));
                    };

                    // Check if this is a method call (with parentheses) or enum case access
                    if self.check(&TokenKind::LeftParen) {
                        self.advance(); // consume '('
                        let args = self.parse_arguments()?;
                        self.consume(
                            TokenKind::RightParen,
                            "Expected ')' after static method arguments",
                        )?;
                        let call = Expr::StaticMethodCall {
                            class_name: name,
                            method: method_or_case,
                            args,
                        };
                        parse_postfix(self, call)
                    } else {
                        // This is an enum case access (no parentheses)
                        let expr = Expr::EnumCase {
                            enum_name: name,
                            case_name: method_or_case,
                        };
                        parse_postfix(self, expr)
                    }
                } else if self.check(&TokenKind::LeftParen) {
                    // Regular function call
                    self.advance(); // consume '('
                    let args = self.parse_arguments()?;
                    self.consume(
                        TokenKind::RightParen,
                        "Expected ')' after function arguments",
                    )?;
                    let call = Expr::FunctionCall { name, args };
                    parse_postfix(self, call)
                } else {
                    Err(format!(
                        "Unexpected identifier '{}' at line {}, column {}",
                        name, token.line, token.column
                    ))
                }
            }
            TokenKind::Parent => {
                self.advance(); // consume 'parent'

                // parent::method() call
                if self.check(&TokenKind::DoubleColon) {
                    self.advance(); // consume '::'
                    let method = if let TokenKind::Identifier(method_name) = &self.current().kind {
                        let method_name = method_name.clone();
                        self.advance();
                        method_name
                    } else {
                        return Err(format!(
                            "Expected method name after 'parent::' at line {}, column {}",
                            self.current().line,
                            self.current().column
                        ));
                    };

                    self.consume(
                        TokenKind::LeftParen,
                        "Expected '(' after parent method name",
                    )?;
                    let args = self.parse_arguments()?;
                    self.consume(
                        TokenKind::RightParen,
                        "Expected ')' after parent method arguments",
                    )?;
                    let call = Expr::StaticMethodCall {
                        class_name: "parent".to_string(),
                        method,
                        args,
                    };
                    parse_postfix(self, call)
                } else {
                    Err(format!(
                        "Expected '::' after 'parent' at line {}, column {}",
                        token.line, token.column
                    ))
                }
            }
            TokenKind::New => {
                self.advance(); // consume 'new'
                let class_name = if let TokenKind::Identifier(name) = &self.current().kind {
                    let name = name.clone();
                    self.advance();
                    name
                } else {
                    return Err(format!(
                        "Expected class name after 'new' at line {}, column {}",
                        self.current().line,
                        self.current().column
                    ));
                };

                let mut args = Vec::new();
                if self.check(&TokenKind::LeftParen) {
                    self.advance(); // consume '('
                    args = self.parse_arguments()?;
                    self.consume(
                        TokenKind::RightParen,
                        "Expected ')' after constructor arguments",
                    )?;
                }

                let new_expr = Expr::New { class_name, args };
                parse_postfix(self, new_expr)
            }
            TokenKind::Clone => {
                self.advance(); // consume 'clone'
                let clone_expr = parse_clone(self)?;
                parse_postfix(self, clone_expr)
            }
            TokenKind::Match => {
                let match_expr = parse_match(self)?;
                parse_postfix(self, match_expr)
            }
            _ => Err(format!(
                "Expected expression but found {:?} at line {}, column {}",
                token.kind, token.line, token.column
            )),
        }
    }
}
