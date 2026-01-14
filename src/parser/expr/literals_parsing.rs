//! Literal and variable parsing for expressions
//!
//! Handles parsing of literals (int, float, string, bool, null) and simple variables.

use super::{parse_postfix, ExprParser};
use crate::ast::Expr;
use crate::token::TokenKind;

impl<'a> ExprParser<'a> {
    /// Parse literal expressions (integers, floats, strings, booleans, null)
    pub(crate) fn parse_literal(&mut self) -> Result<Expr, String> {
        let token = self.current().clone();

        match &token.kind {
            TokenKind::Integer(n) => {
                let n = *n;
                self.advance();
                Ok(Expr::Integer(n))
            }
            TokenKind::Float(n) => {
                let n = *n;
                self.advance();
                Ok(Expr::Float(n))
            }
            TokenKind::String(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::String(s))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr::Bool(true))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::Bool(false))
            }
            TokenKind::Null => {
                self.advance();
                Ok(Expr::Null)
            }
            _ => Err(format!(
                "Expected literal but found {:?} at line {}, column {}",
                token.kind, token.line, token.column
            )),
        }
    }

    /// Parse variable expression
    pub(crate) fn parse_variable(&mut self) -> Result<Expr, String> {
        if let TokenKind::Variable(name) = &self.current().kind {
            let name = name.clone();
            self.advance();
            let expr = if name == "this" {
                Expr::This
            } else {
                Expr::Variable(name)
            };
            parse_postfix(self, expr)
        } else {
            Err(format!(
                "Expected variable at line {}, column {}",
                self.current().line,
                self.current().column
            ))
        }
    }

    /// Parse array literal: [elem1, elem2] or [key => value, ...]
    pub(crate) fn parse_array_literal(&mut self) -> Result<Expr, String> {
        self.advance();
        let mut elements = Vec::new();

        if !self.check(&TokenKind::RightBracket) {
            loop {
                let first = self.parse_expression(super::super::precedence::Precedence::None)?;

                if self.check(&TokenKind::DoubleArrow) {
                    self.advance();
                    let value =
                        self.parse_expression(super::super::precedence::Precedence::None)?;
                    elements.push(crate::ast::ArrayElement {
                        key: Some(Box::new(first)),
                        value: Box::new(value),
                    });
                } else {
                    elements.push(crate::ast::ArrayElement {
                        key: None,
                        value: Box::new(first),
                    });
                }

                if self.check(&TokenKind::Comma) {
                    self.advance();
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

    /// Parse grouped expression: (expr)
    pub(crate) fn parse_grouped(&mut self) -> Result<Expr, String> {
        self.advance();
        let expr = self.parse_expression(super::super::precedence::Precedence::None)?;
        self.consume(TokenKind::RightParen, "Expected ')' after expression")?;
        let grouped = Expr::Grouped(Box::new(expr));
        parse_postfix(self, grouped)
    }
}
