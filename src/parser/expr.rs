//! Expression parsing

use crate::ast::{ArrayElement, AssignOp, BinaryOp, Expr, UnaryOp};
use crate::token::{Token, TokenKind};
use super::precedence::{Precedence, get_precedence, is_right_assoc};

pub struct ExprParser<'a> {
    tokens: &'a [Token],
    pos: &'a mut usize,
}

impl<'a> ExprParser<'a> {
    pub fn new(tokens: &'a [Token], pos: &'a mut usize) -> Self {
        Self { tokens, pos }
    }

    fn current(&self) -> &Token {
        self.tokens.get(*self.pos).unwrap_or(&Token {
            kind: TokenKind::Eof,
            line: 0,
            column: 0,
        })
    }

    fn advance(&mut self) -> Token {
        let token = self.current().clone();
        if *self.pos < self.tokens.len() {
            *self.pos += 1;
        }
        token
    }

    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.current().kind) == std::mem::discriminant(kind)
    }

    fn consume(&mut self, kind: TokenKind, msg: &str) -> Result<Token, String> {
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

    /// Parse array literal: [elem1, elem2] or [key => value, ...]
    fn parse_array_literal(&mut self) -> Result<Expr, String> {
        self.advance(); // consume '['
        let mut elements = Vec::new();

        if !self.check(&TokenKind::RightBracket) {
            loop {
                // Parse first expression (could be key or value)
                let first = self.parse_expression(Precedence::None)?;

                // Check for => (key => value syntax)
                if self.check(&TokenKind::DoubleArrow) {
                    self.advance(); // consume '=>'
                    let value = self.parse_expression(Precedence::None)?;
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

    /// Parse postfix operations (array access, increment/decrement)
    fn parse_postfix(&mut self, mut expr: Expr) -> Result<Expr, String> {
        loop {
            match &self.current().kind {
                TokenKind::LeftBracket => {
                    self.advance(); // consume '['

                    // Check for empty brackets (append syntax: $arr[] = ...)
                    if self.check(&TokenKind::RightBracket) {
                        self.advance(); // consume ']'
                        // This creates an ArrayAccess with a special marker
                        // The assignment handling will recognize this
                        expr = Expr::ArrayAccess {
                            array: Box::new(expr),
                            index: Box::new(Expr::Null), // Placeholder for append
                        };
                    } else {
                        let index = self.parse_expression(Precedence::None)?;
                        self.consume(TokenKind::RightBracket, "Expected ']' after array index")?;
                        expr = Expr::ArrayAccess {
                            array: Box::new(expr),
                            index: Box::new(index),
                        };
                    }
                }
                TokenKind::Increment => {
                    if let Expr::Variable(_) = &expr {
                        self.advance();
                        expr = Expr::Unary {
                            op: UnaryOp::PostInc,
                            expr: Box::new(expr),
                        };
                    } else {
                        break;
                    }
                }
                TokenKind::Decrement => {
                    if let Expr::Variable(_) = &expr {
                        self.advance();
                        expr = Expr::Unary {
                            op: UnaryOp::PostDec,
                            expr: Box::new(expr),
                        };
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    /// Parse primary expression (literals, variables, grouped expressions)
    pub fn parse_primary(&mut self) -> Result<Expr, String> {
        let token = self.current().clone();

        match &token.kind {
            TokenKind::Integer(n) => {
                let n = *n;
                self.advance();
                self.parse_postfix(Expr::Integer(n))
            }
            TokenKind::Float(n) => {
                let n = *n;
                self.advance();
                self.parse_postfix(Expr::Float(n))
            }
            TokenKind::String(s) => {
                let s = s.clone();
                self.advance();
                self.parse_postfix(Expr::String(s))
            }
            TokenKind::True => {
                self.advance();
                self.parse_postfix(Expr::Bool(true))
            }
            TokenKind::False => {
                self.advance();
                self.parse_postfix(Expr::Bool(false))
            }
            TokenKind::Null => {
                self.advance();
                self.parse_postfix(Expr::Null)
            }
            TokenKind::LeftBracket => {
                let arr = self.parse_array_literal()?;
                self.parse_postfix(arr)
            }
            TokenKind::Variable(name) => {
                let name = name.clone();
                self.advance();
                let expr = Expr::Variable(name);
                self.parse_postfix(expr)
            }
            TokenKind::LeftParen => {
                self.advance();
                let expr = self.parse_expression(Precedence::None)?;
                self.consume(TokenKind::RightParen, "Expected ')' after expression")?;
                let grouped = Expr::Grouped(Box::new(expr));
                self.parse_postfix(grouped)
            }
            // Unary operators
            TokenKind::Minus => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                })
            }
            TokenKind::Not => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                })
            }
            TokenKind::Increment => {
                self.advance();
                if let TokenKind::Variable(name) = &self.current().kind {
                    let name = name.clone();
                    self.advance();
                    Ok(Expr::Unary {
                        op: UnaryOp::PreInc,
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
                        op: UnaryOp::PreDec,
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

                // Check for function call
                if self.check(&TokenKind::LeftParen) {
                    self.advance(); // consume '('
                    let mut args = Vec::new();

                    if !self.check(&TokenKind::RightParen) {
                        args.push(self.parse_expression(Precedence::None)?);

                        while self.check(&TokenKind::Comma) {
                            self.advance();
                            args.push(self.parse_expression(Precedence::None)?);
                        }
                    }

                    self.consume(TokenKind::RightParen, "Expected ')' after function arguments")?;
                    let call = Expr::FunctionCall { name, args };
                    self.parse_postfix(call)
                } else {
                    Err(format!(
                        "Unexpected identifier '{}' at line {}, column {}",
                        name, token.line, token.column
                    ))
                }
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
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                })
            }
            TokenKind::Not => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                })
            }
            TokenKind::Increment => {
                self.advance();
                if let TokenKind::Variable(name) = &self.current().kind {
                    let name = name.clone();
                    self.advance();
                    Ok(Expr::Unary {
                        op: UnaryOp::PreInc,
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
                        op: UnaryOp::PreDec,
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
            TokenKind::NullCoalesce => Some(BinaryOp::NullCoalesce),
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

    /// Pratt parser for expressions with precedence
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
                    _ => {
                        return Err(format!(
                            "Left side of assignment must be a variable or array element at line {}, column {}",
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
