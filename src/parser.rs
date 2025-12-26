use crate::ast::{AssignOp, BinaryOp, Expr, Program, Stmt, UnaryOp};
use crate::token::{Token, TokenKind};

/// Operator precedence levels (higher = binds tighter)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    None = 0,
    Assignment = 1,    // = += -= etc.
    Ternary = 2,       // ?:
    NullCoalesce = 3,  // ??
    Or = 4,            // || or
    And = 5,           // && and
    Xor = 6,           // xor
    Equality = 7,      // == === != !==
    Comparison = 8,    // < > <= >= <=>
    Concat = 9,        // .
    AddSub = 10,       // + -
    MulDiv = 11,       // * / %
    Pow = 12,          // ** (right associative)
    Unary = 13,        // ! - ++ --
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token {
            kind: TokenKind::Eof,
            line: 0,
            column: 0,
        })
    }

    fn peek(&self, offset: usize) -> &Token {
        self.tokens.get(self.pos + offset).unwrap_or(&Token {
            kind: TokenKind::Eof,
            line: 0,
            column: 0,
        })
    }

    fn advance(&mut self) -> Token {
        let token = self.current().clone();
        if self.pos < self.tokens.len() {
            self.pos += 1;
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

    /// Get precedence for binary operators
    fn get_precedence(&self, kind: &TokenKind) -> Precedence {
        match kind {
            TokenKind::Assign
            | TokenKind::PlusAssign
            | TokenKind::MinusAssign
            | TokenKind::MulAssign
            | TokenKind::DivAssign
            | TokenKind::ModAssign
            | TokenKind::ConcatAssign => Precedence::Assignment,

            TokenKind::QuestionMark => Precedence::Ternary,
            TokenKind::NullCoalesce => Precedence::NullCoalesce,

            TokenKind::Or => Precedence::Or,
            TokenKind::And => Precedence::And,
            TokenKind::Xor => Precedence::Xor,

            TokenKind::Equal
            | TokenKind::Identical
            | TokenKind::NotEqual
            | TokenKind::NotIdentical => Precedence::Equality,

            TokenKind::LessThan
            | TokenKind::GreaterThan
            | TokenKind::LessEqual
            | TokenKind::GreaterEqual
            | TokenKind::Spaceship => Precedence::Comparison,

            TokenKind::Concat => Precedence::Concat,
            TokenKind::Plus | TokenKind::Minus => Precedence::AddSub,
            TokenKind::Mul | TokenKind::Div | TokenKind::Mod => Precedence::MulDiv,
            TokenKind::Pow => Precedence::Pow,

            _ => Precedence::None,
        }
    }

    /// Check if operator is right-associative
    fn is_right_assoc(&self, kind: &TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::Pow
                | TokenKind::Assign
                | TokenKind::PlusAssign
                | TokenKind::MinusAssign
                | TokenKind::MulAssign
                | TokenKind::DivAssign
                | TokenKind::ModAssign
                | TokenKind::ConcatAssign
                | TokenKind::NullCoalesce
        )
    }

    /// Parse primary expression (literals, variables, grouped expressions)
    fn parse_primary(&mut self) -> Result<Expr, String> {
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
            TokenKind::Variable(name) => {
                let name = name.clone();
                self.advance();

                // Check for postfix increment/decrement
                match &self.current().kind {
                    TokenKind::Increment => {
                        self.advance();
                        Ok(Expr::Unary {
                            op: UnaryOp::PostInc,
                            expr: Box::new(Expr::Variable(name)),
                        })
                    }
                    TokenKind::Decrement => {
                        self.advance();
                        Ok(Expr::Unary {
                            op: UnaryOp::PostDec,
                            expr: Box::new(Expr::Variable(name)),
                        })
                    }
                    _ => Ok(Expr::Variable(name)),
                }
            }
            TokenKind::LeftParen => {
                self.advance();
                let expr = self.parse_expression(Precedence::None)?;
                self.consume(TokenKind::RightParen, "Expected ')' after expression")?;
                Ok(Expr::Grouped(Box::new(expr)))
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
            _ => Err(format!(
                "Expected expression but found {:?} at line {}, column {}",
                token.kind, token.line, token.column
            )),
        }
    }

    /// Parse unary expression
    fn parse_unary(&mut self) -> Result<Expr, String> {
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

    /// Pratt parser for expressions with precedence
    fn parse_expression(&mut self, min_prec: Precedence) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;

        loop {
            let op_token = self.current().clone();
            let prec = self.get_precedence(&op_token.kind);

            // Stop if current operator has lower or equal precedence than minimum
            if prec == Precedence::None || prec <= min_prec {
                break;
            }

            // Handle ternary operator
            if matches!(op_token.kind, TokenKind::QuestionMark) {
                self.advance();
                let then_expr = self.parse_expression(Precedence::None)?;
                self.consume(TokenKind::Colon, "Expected ':' in ternary expression")?;
                // Ternary is right-associative, so use same precedence minus one
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
                // Left side must be a variable
                if let Expr::Variable(name) = left {
                    self.advance();
                    // Assignment is right-associative, so use same precedence minus one
                    let right = self.parse_expression(Precedence::None)?;
                    left = Expr::Assign {
                        var: name,
                        op: assign_op,
                        value: Box::new(right),
                    };
                    continue;
                } else {
                    return Err(format!(
                        "Left side of assignment must be a variable at line {}, column {}",
                        op_token.line, op_token.column
                    ));
                }
            }

            // Handle binary operators
            if let Some(bin_op) = self.token_to_binop(&op_token.kind) {
                self.advance();
                // For right-associative operators, use (prec - 1) to allow same-precedence operators on the right
                // For left-associative operators, use prec to prevent same-precedence operators on the right
                let right = self.parse_expression(if self.is_right_assoc(&op_token.kind) {
                    // Decrement precedence by one for right-associative
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

    /// Parse echo statement
    fn parse_echo(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'echo'
        let mut expressions = Vec::new();

        expressions.push(self.parse_expression(Precedence::None)?);

        while self.check(&TokenKind::Comma) {
            self.advance();
            expressions.push(self.parse_expression(Precedence::None)?);
        }

        // Semicolon is optional before close tag
        if self.check(&TokenKind::Semicolon) {
            self.advance();
        } else if !self.check(&TokenKind::CloseTag) && !self.check(&TokenKind::Eof) {
            return Err(format!(
                "Expected ';' or '?>' after echo at line {}, column {}",
                self.current().line,
                self.current().column
            ));
        }

        Ok(Stmt::Echo(expressions))
    }

    /// Parse expression statement
    fn parse_expression_statement(&mut self) -> Result<Stmt, String> {
        let expr = self.parse_expression(Precedence::None)?;

        if self.check(&TokenKind::Semicolon) {
            self.advance();
        } else if !self.check(&TokenKind::CloseTag) && !self.check(&TokenKind::Eof) {
            return Err(format!(
                "Expected ';' after expression at line {}, column {}",
                self.current().line,
                self.current().column
            ));
        }

        Ok(Stmt::Expression(expr))
    }

    fn parse_statement(&mut self) -> Result<Option<Stmt>, String> {
        let token = self.current().clone();
        match token.kind {
            TokenKind::OpenTag => {
                self.advance();
                Ok(None)
            }
            TokenKind::CloseTag => {
                self.advance();
                Ok(None)
            }
            TokenKind::Echo => Ok(Some(self.parse_echo()?)),
            TokenKind::Html(html) => {
                self.advance();
                Ok(Some(Stmt::Html(html)))
            }
            TokenKind::Eof => Ok(None),
            // Everything else is an expression statement
            TokenKind::Variable(_)
            | TokenKind::Integer(_)
            | TokenKind::Float(_)
            | TokenKind::String(_)
            | TokenKind::True
            | TokenKind::False
            | TokenKind::Null
            | TokenKind::LeftParen
            | TokenKind::Minus
            | TokenKind::Not
            | TokenKind::Increment
            | TokenKind::Decrement => Ok(Some(self.parse_expression_statement()?)),
            _ => Err(format!(
                "Unexpected token {:?} at line {}, column {}",
                token.kind, token.line, token.column
            )),
        }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut statements = Vec::new();

        while !self.check(&TokenKind::Eof) {
            if let Some(stmt) = self.parse_statement()? {
                statements.push(stmt);
            }
        }

        Ok(Program { statements })
    }
}
