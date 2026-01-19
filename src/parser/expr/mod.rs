//! Expression parsing with modular architecture
//!
//! This module provides expression parsing through:
//! - mod.rs: Main Pratt parser with operator precedence climbing
//! - primary.rs: Dispatcher for primary expression parsing
//! - literals_parsing.rs: Literals, variables, arrays, and grouped expressions
//! - callable_parsing.rs: Function calls, static method calls, object instantiation
//! - arrow_anonymous_parsing.rs: Arrow functions and anonymous classes
//! - postfix.rs: Postfix operations (array access, property access, method calls)
//! - special.rs: Complex expressions (match, clone)

mod arrow_anonymous_parsing;
mod callable_parsing;
mod literals_parsing;
mod postfix;
mod special;

use super::precedence::{get_precedence, is_right_assoc, Precedence};
use crate::ast::{AssignOp, BinaryOp, Expr};
use crate::token::{Token, TokenKind};

pub use postfix::parse_postfix;
pub use special::{parse_clone, parse_match};

pub struct ExprParser<'a> {
    tokens: &'a [Token],
    pos: &'a mut usize,
}

impl<'a> ExprParser<'a> {
    pub fn new(tokens: &'a [Token], pos: &'a mut usize) -> Self {
        Self { tokens, pos }
    }

    pub fn current(&self) -> &Token {
        self.tokens.get(*self.pos).unwrap_or(&Token {
            kind: TokenKind::Eof,
            line: 0,
            column: 0,
        })
    }

    pub fn advance(&mut self) -> Token {
        let token = self.current().clone();
        if *self.pos < self.tokens.len() {
            *self.pos += 1;
        }
        token
    }

    pub fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.current().kind) == std::mem::discriminant(kind)
    }

    pub fn consume(&mut self, kind: TokenKind, msg: &str) -> Result<Token, String> {
        if self.check(&kind) {
            Ok(self.advance())
        } else {
            Err(format!(
                "{} at line {}, column {} (found {:?})",
                msg,
                self.current().line,
                self.current().column,
                self.current().kind
            ))
        }
    }

    /// Convert token to binary operator
    fn token_to_binop(&self, kind: &TokenKind) -> Option<BinaryOp> {
        match kind {
            TokenKind::Plus => Some(BinaryOp::Add),
            TokenKind::Minus => Some(BinaryOp::Sub),
            TokenKind::Mul => Some(BinaryOp::Mul),
            TokenKind::Div => Some(BinaryOp::Div),
            TokenKind::Mod => Some(BinaryOp::Mod),
            TokenKind::Pow => Some(BinaryOp::Pow),
            TokenKind::Concat => Some(BinaryOp::Concat),
            TokenKind::Equal => Some(BinaryOp::Equal),
            TokenKind::Identical => Some(BinaryOp::Identical),
            TokenKind::NotEqual => Some(BinaryOp::NotEqual),
            TokenKind::NotIdentical => Some(BinaryOp::NotIdentical),
            TokenKind::LessThan => Some(BinaryOp::LessThan),
            TokenKind::GreaterThan => Some(BinaryOp::GreaterThan),
            TokenKind::LessEqual => Some(BinaryOp::LessEqual),
            TokenKind::GreaterEqual => Some(BinaryOp::GreaterEqual),
            TokenKind::Spaceship => Some(BinaryOp::Spaceship),
            TokenKind::And => Some(BinaryOp::And),
            TokenKind::Or => Some(BinaryOp::Or),
            TokenKind::Xor => Some(BinaryOp::Xor),
            TokenKind::BitwiseOr => Some(BinaryOp::BitwiseOr),
            TokenKind::NullCoalesce => Some(BinaryOp::NullCoalesce),
            TokenKind::Pipe => Some(BinaryOp::Pipe),
            _ => None,
        }
    }

    /// Convert token to assignment operator
    fn token_to_assignop(&self, kind: &TokenKind) -> Option<AssignOp> {
        match kind {
            TokenKind::Assign => Some(AssignOp::Assign),
            TokenKind::PlusAssign => Some(AssignOp::AddAssign),
            TokenKind::MinusAssign => Some(AssignOp::SubAssign),
            TokenKind::MulAssign => Some(AssignOp::MulAssign),
            TokenKind::DivAssign => Some(AssignOp::DivAssign),
            TokenKind::ModAssign => Some(AssignOp::ModAssign),
            TokenKind::ConcatAssign => Some(AssignOp::ConcatAssign),
            _ => None,
        }
    }

    /// Check if expression is an array access for append ($arr[])
    fn is_array_append(&self, expr: &Expr) -> bool {
        if let Expr::ArrayAccess { index, .. } = expr {
            matches!(**index, Expr::Null)
        } else {
            false
        }
    }

    /// Parse primary expression (literals, variables, grouped expressions, etc.)
    pub fn parse_primary(&mut self) -> Result<Expr, String> {
        let token = self.current().clone();

        match &token.kind {
            TokenKind::Integer(_n) => self.parse_literal(),
            TokenKind::Float(_n) => self.parse_literal(),
            TokenKind::String(_s) => self.parse_literal(),
            TokenKind::True => self.parse_literal(),
            TokenKind::False => self.parse_literal(),
            TokenKind::Null => self.parse_literal(),
            TokenKind::LeftBracket => self.parse_array_literal(),
            TokenKind::Variable(_) => self.parse_variable(),
            TokenKind::LeftParen => self.parse_grouped(),
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
                self.advance();

                // Check if this is a qualified name (namespace/class path)
                let mut qualified_name = name.clone();
                while self.check(&TokenKind::Backslash) {
                    self.advance(); // consume '\'
                    if let TokenKind::Identifier(next_part) = &self.current().kind {
                        qualified_name.push('\\');
                        qualified_name.push_str(next_part);
                        self.advance();
                    } else {
                        return Err(format!(
                            "Expected identifier after '\\' at line {}, column {}",
                            self.current().line,
                            self.current().column
                        ));
                    }
                }

                if self.check(&TokenKind::DoubleColon) {
                    self.parse_static_access(qualified_name)
                } else if self.check(&TokenKind::LeftParen) {
                    self.parse_function_call(qualified_name)
                } else {
                    Err(format!(
                        "Unexpected identifier '{}' at line {}, column {}",
                        qualified_name, token.line, token.column
                    ))
                }
            }
            TokenKind::Fiber => {
                let name = "Fiber".to_string();
                self.advance();

                if self.check(&TokenKind::DoubleColon) {
                    self.parse_static_access(name)
                } else {
                    Err(format!(
                        "Unexpected 'Fiber' token at line {}, column {}",
                        token.line, token.column
                    ))
                }
            }
            TokenKind::Parent => {
                self.advance();

                if self.check(&TokenKind::DoubleColon) {
                    self.parse_static_access("parent".to_string())
                } else {
                    Err(format!(
                        "Expected '::' after 'parent' at line {}, column {}",
                        token.line, token.column
                    ))
                }
            }
            TokenKind::Static => {
                self.advance();

                if self.check(&TokenKind::DoubleColon) {
                    self.parse_static_access("static".to_string())
                } else {
                    Err(format!(
                        "Expected '::' after 'static' at line {}, column {}",
                        token.line, token.column
                    ))
                }
            }
            TokenKind::New => self.parse_new_object(),
            TokenKind::Clone => {
                self.advance();
                let clone_expr = parse_clone(self)?;
                parse_postfix(self, clone_expr)
            }
            TokenKind::Match => {
                let match_expr = parse_match(self)?;
                parse_postfix(self, match_expr)
            }
            TokenKind::Fn => {
                self.advance();
                let arrow_func = self.parse_arrow_function()?;
                parse_postfix(self, arrow_func)
            }
            TokenKind::Throw => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Throw(Box::new(expr)))
            }
            TokenKind::Yield => {
                self.advance();
                let mut key: Option<Box<Expr>> = None;
                let value;

                if self.check(&TokenKind::From) {
                    self.advance();
                    let expr = self.parse_unary()?;
                    return Ok(Expr::YieldFrom(Box::new(expr)));
                }

                let first_expr = self.parse_unary()?;
                if self.check(&TokenKind::DoubleArrow) {
                    self.advance();
                    key = Some(Box::new(first_expr));
                    value = Some(Box::new(self.parse_unary()?));
                } else {
                    value = Some(Box::new(first_expr));
                }

                Ok(Expr::Yield { key, value })
            }
            // Magic constants
            TokenKind::MagicFile => {
                self.advance();
                Ok(Expr::MagicFile)
            }
            TokenKind::MagicLine => {
                let line = token.line;
                self.advance();
                Ok(Expr::MagicLine { 0: line })
            }
            TokenKind::MagicDir => {
                self.advance();
                Ok(Expr::MagicDir)
            }
            TokenKind::MagicFunction => {
                self.advance();
                Ok(Expr::MagicFunction)
            }
            TokenKind::MagicClass => {
                self.advance();
                Ok(Expr::MagicClass)
            }
            TokenKind::MagicMethod => {
                self.advance();
                Ok(Expr::MagicMethod)
            }
            TokenKind::MagicNamespace => {
                self.advance();
                Ok(Expr::MagicNamespace)
            }
            TokenKind::MagicTrait => {
                self.advance();
                Ok(Expr::MagicTrait)
            }
            _ => Err(format!(
                "Expected expression but found {:?} at line {}, column {}",
                token.kind, token.line, token.column
            )),
        }
    }

    /// Parse unary expression
    pub fn parse_unary(&mut self) -> Result<Expr, String> {
        match &self.current().kind {
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
            _ => self.parse_primary(),
        }
    }

    /// Pratt parser for expressions with precedence climbing
    pub fn parse_expression(&mut self, min_prec: Precedence) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;

        loop {
            let op_token = self.current().clone();
            let prec = get_precedence(&op_token.kind);

            // Stop if current operator has lower or equal precedence than minimum
            if prec == Precedence::None || prec <= min_prec {
                break;
            }

            // Handle ternary operator
            if matches!(op_token.kind, TokenKind::QuestionMark) {
                self.advance();
                let then_expr = self.parse_expression(Precedence::None)?;
                self.consume(TokenKind::Colon, "Expected ':' in ternary expression")?;
                let else_expr = self.parse_expression(Precedence::Assignment)?;
                left = Expr::Ternary {
                    condition: Box::new(left),
                    then_expr: Box::new(then_expr),
                    else_expr: Box::new(else_expr),
                };
                continue;
            }

            // Handle assignment operators
            if let Some(assign_op) = self.token_to_assignop(&op_token.kind) {
                match &left {
                    Expr::Variable(name) => {
                        self.advance();
                        let right = self.parse_expression(Precedence::None)?;
                        left = Expr::Assign {
                            var: name.clone(),
                            op: assign_op,
                            value: Box::new(right),
                        };
                        continue;
                    }
                    Expr::ArrayAccess { array, index } => {
                        self.advance();
                        let right = self.parse_expression(Precedence::None)?;
                        // Check if this is append syntax ($arr[] = ...)
                        let index_opt = if self.is_array_append(&left) {
                            None
                        } else {
                            Some(index.clone())
                        };
                        left = Expr::ArrayAssign {
                            array: array.clone(),
                            index: index_opt,
                            op: assign_op,
                            value: Box::new(right),
                        };
                        continue;
                    }
                    Expr::PropertyAccess { object, property } => {
                        // Only support simple assignment for properties
                        if !matches!(assign_op, AssignOp::Assign) {
                            return Err(format!(
                                "Compound assignment not supported for properties at line {}, column {}",
                                op_token.line, op_token.column
                            ));
                        }
                        self.advance();
                        let right = self.parse_expression(Precedence::None)?;
                        left = Expr::PropertyAssign {
                            object: object.clone(),
                            property: property.clone(),
                            value: Box::new(right),
                        };
                        continue;
                    }
                    Expr::StaticPropertyAccess { class, property } => {
                        // Static property assignment: ClassName::$prop = value
                        if !matches!(assign_op, AssignOp::Assign) {
                            return Err(format!(
                                "Compound assignment not supported for static properties at line {}, column {}",
                                op_token.line, op_token.column
                            ));
                        }
                        self.advance();
                        let right = self.parse_expression(Precedence::None)?;
                        left = Expr::StaticPropertyAssign {
                            class: class.clone(),
                            property: property.clone(),
                            value: Box::new(right),
                        };
                        continue;
                    }
                    _ => {
                        return Err(format!(
                            "Left side of assignment must be a variable, array element, property, or static property at line {}, column {}",
                            op_token.line, op_token.column
                        ));
                    }
                }
            }

            // Handle binary operators
            if let Some(bin_op) = self.token_to_binop(&op_token.kind) {
                self.advance();
                let right = self.parse_expression(if is_right_assoc(&op_token.kind) {
                    match prec {
                        Precedence::Pow => Precedence::MulDiv,
                        Precedence::NullCoalesce => Precedence::Or,
                        _ => prec,
                    }
                } else {
                    prec
                })?;
                left = Expr::Binary {
                    left: Box::new(left),
                    op: bin_op,
                    right: Box::new(right),
                };
                continue;
            }

            break;
        }

        Ok(left)
    }
}
