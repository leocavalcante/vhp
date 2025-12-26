use crate::ast::{AssignOp, BinaryOp, Expr, FunctionParam, Program, Stmt, SwitchCase, UnaryOp};
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
                    Ok(Expr::FunctionCall { name, args })
                } else {
                    // Just an identifier - could be a constant, treat as undefined for now
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

    /// Parse a block of statements enclosed in braces or a single statement
    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        if self.check(&TokenKind::LeftBrace) {
            self.advance(); // consume '{'
            let mut statements = Vec::new();

            while !self.check(&TokenKind::RightBrace) && !self.check(&TokenKind::Eof) {
                if let Some(stmt) = self.parse_statement()? {
                    statements.push(stmt);
                }
            }

            self.consume(TokenKind::RightBrace, "Expected '}' after block")?;
            Ok(statements)
        } else if self.check(&TokenKind::Colon) {
            // Alternative syntax: if (...): ... endif;
            self.advance(); // consume ':'
            let mut statements = Vec::new();

            while !self.check(&TokenKind::Eof) {
                // Check for endif, endwhile, endfor, endforeach, endswitch, else, elseif, case, default
                match &self.current().kind {
                    TokenKind::Identifier(s)
                        if s.to_lowercase() == "endif"
                            || s.to_lowercase() == "endwhile"
                            || s.to_lowercase() == "endfor"
                            || s.to_lowercase() == "endforeach"
                            || s.to_lowercase() == "endswitch" => break,
                    TokenKind::Else | TokenKind::Elseif | TokenKind::Case | TokenKind::Default => break,
                    _ => {}
                }

                if let Some(stmt) = self.parse_statement()? {
                    statements.push(stmt);
                }
            }

            Ok(statements)
        } else {
            // Single statement
            let mut statements = Vec::new();
            if let Some(stmt) = self.parse_statement()? {
                statements.push(stmt);
            }
            Ok(statements)
        }
    }

    /// Parse if statement
    fn parse_if(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'if'
        self.consume(TokenKind::LeftParen, "Expected '(' after 'if'")?;
        let condition = self.parse_expression(Precedence::None)?;
        self.consume(TokenKind::RightParen, "Expected ')' after if condition")?;

        // Check if using alternative syntax
        let using_alt_syntax = self.check(&TokenKind::Colon);
        let then_branch = self.parse_block()?;

        let mut elseif_branches = Vec::new();
        let mut else_branch = None;

        // Parse elseif clauses
        while self.check(&TokenKind::Elseif) {
            self.advance(); // consume 'elseif'
            self.consume(TokenKind::LeftParen, "Expected '(' after 'elseif'")?;
            let elseif_condition = self.parse_expression(Precedence::None)?;
            self.consume(TokenKind::RightParen, "Expected ')' after elseif condition")?;
            let elseif_body = self.parse_block()?;
            elseif_branches.push((elseif_condition, elseif_body));
        }

        // Parse else clause
        if self.check(&TokenKind::Else) {
            self.advance(); // consume 'else'

            // Check for else if (two separate tokens)
            if self.check(&TokenKind::If) {
                // Parse as nested if and wrap in else branch
                let nested_if = self.parse_if()?;
                else_branch = Some(vec![nested_if]);
            } else {
                else_branch = Some(self.parse_block()?);
            }
        }

        // Handle endif for alternative syntax
        if using_alt_syntax {
            if let TokenKind::Identifier(s) = &self.current().kind {
                if s.to_lowercase() == "endif" {
                    self.advance(); // consume 'endif'
                    if self.check(&TokenKind::Semicolon) {
                        self.advance();
                    }
                }
            }
        }

        Ok(Stmt::If {
            condition,
            then_branch,
            elseif_branches,
            else_branch,
        })
    }

    /// Parse while statement
    fn parse_while(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'while'
        self.consume(TokenKind::LeftParen, "Expected '(' after 'while'")?;
        let condition = self.parse_expression(Precedence::None)?;
        self.consume(TokenKind::RightParen, "Expected ')' after while condition")?;

        let using_alt_syntax = self.check(&TokenKind::Colon);
        let body = self.parse_block()?;

        // Handle endwhile for alternative syntax
        if using_alt_syntax {
            if let TokenKind::Identifier(s) = &self.current().kind {
                if s.to_lowercase() == "endwhile" {
                    self.advance(); // consume 'endwhile'
                    if self.check(&TokenKind::Semicolon) {
                        self.advance();
                    }
                }
            }
        }

        Ok(Stmt::While { condition, body })
    }

    /// Parse do-while statement
    fn parse_do_while(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'do'
        let body = self.parse_block()?;
        self.consume(TokenKind::While, "Expected 'while' after do block")?;
        self.consume(TokenKind::LeftParen, "Expected '(' after 'while'")?;
        let condition = self.parse_expression(Precedence::None)?;
        self.consume(TokenKind::RightParen, "Expected ')' after while condition")?;

        if self.check(&TokenKind::Semicolon) {
            self.advance();
        }

        Ok(Stmt::DoWhile { body, condition })
    }

    /// Parse for statement
    fn parse_for(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'for'
        self.consume(TokenKind::LeftParen, "Expected '(' after 'for'")?;

        // Parse init expression (optional)
        let init = if !self.check(&TokenKind::Semicolon) {
            Some(self.parse_expression(Precedence::None)?)
        } else {
            None
        };
        self.consume(TokenKind::Semicolon, "Expected ';' after for init")?;

        // Parse condition (optional)
        let condition = if !self.check(&TokenKind::Semicolon) {
            Some(self.parse_expression(Precedence::None)?)
        } else {
            None
        };
        self.consume(TokenKind::Semicolon, "Expected ';' after for condition")?;

        // Parse update expression (optional)
        let update = if !self.check(&TokenKind::RightParen) {
            Some(self.parse_expression(Precedence::None)?)
        } else {
            None
        };
        self.consume(TokenKind::RightParen, "Expected ')' after for clauses")?;

        let using_alt_syntax = self.check(&TokenKind::Colon);
        let body = self.parse_block()?;

        // Handle endfor for alternative syntax
        if using_alt_syntax {
            if let TokenKind::Identifier(s) = &self.current().kind {
                if s.to_lowercase() == "endfor" {
                    self.advance(); // consume 'endfor'
                    if self.check(&TokenKind::Semicolon) {
                        self.advance();
                    }
                }
            }
        }

        Ok(Stmt::For {
            init,
            condition,
            update,
            body,
        })
    }

    /// Parse foreach statement
    fn parse_foreach(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'foreach'
        self.consume(TokenKind::LeftParen, "Expected '(' after 'foreach'")?;

        let array = self.parse_expression(Precedence::None)?;
        self.consume(TokenKind::As, "Expected 'as' in foreach")?;

        // Parse key => value or just value
        let first_var = if let TokenKind::Variable(name) = &self.current().kind {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(format!(
                "Expected variable after 'as' at line {}, column {}",
                self.current().line,
                self.current().column
            ));
        };

        let (key, value) = if self.check(&TokenKind::Identifier(String::new())) {
            // Check for => (arrow)
            if let TokenKind::Identifier(s) = &self.current().kind {
                if s == "=>" {
                    self.advance(); // consume '=>'

                    if let TokenKind::Variable(val_name) = &self.current().kind {
                        let val_name = val_name.clone();
                        self.advance();
                        (Some(first_var), val_name)
                    } else {
                        return Err(format!(
                            "Expected variable after '=>' at line {}, column {}",
                            self.current().line,
                            self.current().column
                        ));
                    }
                } else {
                    (None, first_var)
                }
            } else {
                (None, first_var)
            }
        } else {
            (None, first_var)
        };

        self.consume(TokenKind::RightParen, "Expected ')' after foreach")?;

        let using_alt_syntax = self.check(&TokenKind::Colon);
        let body = self.parse_block()?;

        // Handle endforeach for alternative syntax
        if using_alt_syntax {
            if let TokenKind::Identifier(s) = &self.current().kind {
                if s.to_lowercase() == "endforeach" {
                    self.advance(); // consume 'endforeach'
                    if self.check(&TokenKind::Semicolon) {
                        self.advance();
                    }
                }
            }
        }

        Ok(Stmt::Foreach {
            array,
            key,
            value,
            body,
        })
    }

    /// Parse switch statement
    fn parse_switch(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'switch'
        self.consume(TokenKind::LeftParen, "Expected '(' after 'switch'")?;
        let expr = self.parse_expression(Precedence::None)?;
        self.consume(TokenKind::RightParen, "Expected ')' after switch expression")?;

        let using_alt_syntax = self.check(&TokenKind::Colon);

        if using_alt_syntax {
            self.advance(); // consume ':'
        } else {
            self.consume(TokenKind::LeftBrace, "Expected '{' or ':' after switch")?;
        }

        let mut cases = Vec::new();
        let mut default = None;

        while !self.check(&TokenKind::RightBrace) && !self.check(&TokenKind::Eof) {
            // Check for endswitch in alternative syntax
            if using_alt_syntax {
                if let TokenKind::Identifier(s) = &self.current().kind {
                    if s.to_lowercase() == "endswitch" {
                        break;
                    }
                }
            }

            if self.check(&TokenKind::Case) {
                self.advance(); // consume 'case'
                let value = self.parse_expression(Precedence::None)?;
                self.consume(TokenKind::Colon, "Expected ':' after case value")?;

                let mut body = Vec::new();
                while !self.check(&TokenKind::Case)
                    && !self.check(&TokenKind::Default)
                    && !self.check(&TokenKind::RightBrace)
                    && !self.check(&TokenKind::Eof)
                {
                    // Check for endswitch
                    if let TokenKind::Identifier(s) = &self.current().kind {
                        if s.to_lowercase() == "endswitch" {
                            break;
                        }
                    }

                    if let Some(stmt) = self.parse_statement()? {
                        body.push(stmt);
                    }
                }

                cases.push(SwitchCase { value, body });
            } else if self.check(&TokenKind::Default) {
                self.advance(); // consume 'default'
                self.consume(TokenKind::Colon, "Expected ':' after 'default'")?;

                let mut body = Vec::new();
                while !self.check(&TokenKind::Case)
                    && !self.check(&TokenKind::RightBrace)
                    && !self.check(&TokenKind::Eof)
                {
                    // Check for endswitch
                    if let TokenKind::Identifier(s) = &self.current().kind {
                        if s.to_lowercase() == "endswitch" {
                            break;
                        }
                    }

                    if let Some(stmt) = self.parse_statement()? {
                        body.push(stmt);
                    }
                }

                default = Some(body);
            } else {
                break;
            }
        }

        if using_alt_syntax {
            if let TokenKind::Identifier(s) = &self.current().kind {
                if s.to_lowercase() == "endswitch" {
                    self.advance(); // consume 'endswitch'
                    if self.check(&TokenKind::Semicolon) {
                        self.advance();
                    }
                }
            }
        } else {
            self.consume(TokenKind::RightBrace, "Expected '}' after switch")?;
        }

        Ok(Stmt::Switch {
            expr,
            cases,
            default,
        })
    }

    /// Parse break statement
    fn parse_break(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'break'
        if self.check(&TokenKind::Semicolon) {
            self.advance();
        }
        Ok(Stmt::Break)
    }

    /// Parse continue statement
    fn parse_continue(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'continue'
        if self.check(&TokenKind::Semicolon) {
            self.advance();
        }
        Ok(Stmt::Continue)
    }

    /// Parse function declaration
    fn parse_function(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'function'

        // Get function name
        let name = if let TokenKind::Identifier(name) = &self.current().kind {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(format!(
                "Expected function name at line {}, column {}",
                self.current().line,
                self.current().column
            ));
        };

        self.consume(TokenKind::LeftParen, "Expected '(' after function name")?;

        // Parse parameters
        let mut params = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            loop {
                // Check for by-reference parameter
                let by_ref = if let TokenKind::Identifier(s) = &self.current().kind {
                    if s == "&" {
                        self.advance();
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                // Get parameter name
                let param_name = if let TokenKind::Variable(name) = &self.current().kind {
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

                // Check for default value
                let default = if self.check(&TokenKind::Assign) {
                    self.advance();
                    Some(self.parse_expression(Precedence::None)?)
                } else {
                    None
                };

                params.push(FunctionParam {
                    name: param_name,
                    default,
                    by_ref,
                });

                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }

        self.consume(TokenKind::RightParen, "Expected ')' after parameters")?;
        self.consume(TokenKind::LeftBrace, "Expected '{' before function body")?;

        // Parse function body
        let mut body = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.check(&TokenKind::Eof) {
            if let Some(stmt) = self.parse_statement()? {
                body.push(stmt);
            }
        }

        self.consume(TokenKind::RightBrace, "Expected '}' after function body")?;

        Ok(Stmt::Function { name, params, body })
    }

    /// Parse return statement
    fn parse_return(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'return'

        // Check if there's a value to return
        let value = if self.check(&TokenKind::Semicolon)
            || self.check(&TokenKind::CloseTag)
            || self.check(&TokenKind::Eof)
        {
            None
        } else {
            Some(self.parse_expression(Precedence::None)?)
        };

        if self.check(&TokenKind::Semicolon) {
            self.advance();
        }

        Ok(Stmt::Return(value))
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
            TokenKind::If => Ok(Some(self.parse_if()?)),
            TokenKind::While => Ok(Some(self.parse_while()?)),
            TokenKind::Do => Ok(Some(self.parse_do_while()?)),
            TokenKind::For => Ok(Some(self.parse_for()?)),
            TokenKind::Foreach => Ok(Some(self.parse_foreach()?)),
            TokenKind::Switch => Ok(Some(self.parse_switch()?)),
            TokenKind::Break => Ok(Some(self.parse_break()?)),
            TokenKind::Continue => Ok(Some(self.parse_continue()?)),
            TokenKind::Function => Ok(Some(self.parse_function()?)),
            TokenKind::Return => Ok(Some(self.parse_return()?)),
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
            | TokenKind::Decrement
            | TokenKind::Identifier(_) => Ok(Some(self.parse_expression_statement()?)),
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
