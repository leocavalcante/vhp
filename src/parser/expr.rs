//! Expression parsing

use crate::ast::{Argument, ArrayElement, AssignOp, BinaryOp, Expr, MatchArm, UnaryOp};
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

    /// Parse function/method arguments with support for named arguments (PHP 8.0)
    /// Syntax: expr, expr, name: expr, name: expr
    /// Rule: positional arguments must come before named arguments
    fn parse_arguments(&mut self) -> Result<Vec<Argument>, String> {
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
                    let value = self.parse_expression(Precedence::None)?;
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
                    let value = self.parse_expression(Precedence::None)?;
                    args.push(Argument {
                        name: None,
                        value: Box::new(value),
                    });
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

    /// Parse match expression: match ($expr) { cond1, cond2 => result, default => result }
    fn parse_match(&mut self) -> Result<Expr, String> {
        self.advance(); // consume 'match'
        self.consume(TokenKind::LeftParen, "Expected '(' after 'match'")?;
        let expr = self.parse_expression(Precedence::None)?;
        self.consume(TokenKind::RightParen, "Expected ')' after match expression")?;
        self.consume(TokenKind::LeftBrace, "Expected '{' to start match body")?;

        let mut arms = Vec::new();
        let mut default: Option<Box<Expr>> = None;

        while !self.check(&TokenKind::RightBrace) && !self.check(&TokenKind::Eof) {
            // Check for default arm
            if self.check(&TokenKind::Default) {
                self.advance(); // consume 'default'
                self.consume(TokenKind::DoubleArrow, "Expected '=>' after 'default'")?;
                let result = self.parse_expression(Precedence::None)?;
                default = Some(Box::new(result));

                // Optional comma after arm
                if self.check(&TokenKind::Comma) {
                    self.advance();
                }
                continue;
            }

            // Parse conditions (can be multiple, comma-separated before =>)
            let mut conditions = Vec::new();
            conditions.push(self.parse_expression(Precedence::None)?);

            // Handle multiple conditions: 1, 2 => ...
            while self.check(&TokenKind::Comma) {
                // Peek ahead to see if this is a continuation of conditions or the next arm
                let next_pos = *self.pos + 1;
                if next_pos < self.tokens.len() {
                    // Check if after comma we have '=>' (end of conditions) or '}' (malformed)
                    // or another expression (more conditions)
                    let after_comma = &self.tokens[next_pos].kind;
                    if matches!(after_comma, TokenKind::RightBrace | TokenKind::Default) {
                        break;
                    }
                }
                self.advance(); // consume comma

                // If next is '=>', we're done with conditions for this arm
                if self.check(&TokenKind::DoubleArrow) {
                    break;
                }

                conditions.push(self.parse_expression(Precedence::None)?);
            }

            self.consume(TokenKind::DoubleArrow, "Expected '=>' after match condition(s)")?;
            let result = self.parse_expression(Precedence::None)?;

            arms.push(MatchArm {
                conditions,
                result: Box::new(result),
            });

            // Optional comma after arm
            if self.check(&TokenKind::Comma) {
                self.advance();
            }
        }

        self.consume(TokenKind::RightBrace, "Expected '}' to end match body")?;

        Ok(Expr::Match {
            expr: Box::new(expr),
            arms,
            default,
        })
    }

    /// Parse postfix operations (array access, property access, method calls, increment/decrement)
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
                TokenKind::Arrow => {
                    self.advance(); // consume '->'
                    let member = if let TokenKind::Identifier(name) = &self.current().kind {
                        let name = name.clone();
                        self.advance();
                        name
                    } else {
                        return Err(format!(
                            "Expected property or method name after '->' at line {}, column {}",
                            self.current().line,
                            self.current().column
                        ));
                    };

                    // Check if it's a method call or property access
                    if self.check(&TokenKind::LeftParen) {
                        self.advance(); // consume '('
                        let args = self.parse_arguments()?;
                        self.consume(TokenKind::RightParen, "Expected ')' after method arguments")?;
                        expr = Expr::MethodCall {
                            object: Box::new(expr),
                            method: member,
                            args,
                        };
                    } else {
                        expr = Expr::PropertyAccess {
                            object: Box::new(expr),
                            property: member,
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
                // $this is a special reference
                let expr = if name == "this" {
                    Expr::This
                } else {
                    Expr::Variable(name)
                };
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

                // Check for static method call (ClassName::method())
                if self.check(&TokenKind::DoubleColon) {
                    self.advance(); // consume '::'
                    let method = if let TokenKind::Identifier(method_name) = &self.current().kind {
                        let method_name = method_name.clone();
                        self.advance();
                        method_name
                    } else {
                        return Err(format!(
                            "Expected method name after '::' at line {}, column {}",
                            self.current().line,
                            self.current().column
                        ));
                    };

                    self.consume(TokenKind::LeftParen, "Expected '(' after static method name")?;
                    let args = self.parse_arguments()?;
                    self.consume(TokenKind::RightParen, "Expected ')' after static method arguments")?;
                    let call = Expr::StaticMethodCall {
                        class_name: name,
                        method,
                        args,
                    };
                    self.parse_postfix(call)
                } else if self.check(&TokenKind::LeftParen) {
                    // Regular function call
                    self.advance(); // consume '('
                    let args = self.parse_arguments()?;
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

                    self.consume(TokenKind::LeftParen, "Expected '(' after parent method name")?;
                    let args = self.parse_arguments()?;
                    self.consume(TokenKind::RightParen, "Expected ')' after parent method arguments")?;
                    let call = Expr::StaticMethodCall {
                        class_name: "parent".to_string(),
                        method,
                        args,
                    };
                    self.parse_postfix(call)
                } else {
                    Err(format!(
                        "Expected '::' after 'parent' at line {}, column {}",
                        token.line,
                        token.column
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
                    self.consume(TokenKind::RightParen, "Expected ')' after constructor arguments")?;
                }

                let new_expr = Expr::New { class_name, args };
                self.parse_postfix(new_expr)
            }
            TokenKind::Match => {
                let match_expr = self.parse_match()?;
                self.parse_postfix(match_expr)
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
                    _ => {
                        return Err(format!(
                            "Left side of assignment must be a variable, array element, or property at line {}, column {}",
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
