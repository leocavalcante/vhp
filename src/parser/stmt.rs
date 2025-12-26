//! Statement parsing

use crate::ast::{Expr, FunctionParam, Method, Property, Stmt, SwitchCase, Visibility};
use crate::token::{Token, TokenKind};
use super::expr::ExprParser;
use super::precedence::Precedence;

pub struct StmtParser<'a> {
    tokens: &'a [Token],
    pos: &'a mut usize,
}

impl<'a> StmtParser<'a> {
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

    fn parse_expression(&mut self, min_prec: Precedence) -> Result<Expr, String> {
        let mut expr_parser = ExprParser::new(self.tokens, self.pos);
        expr_parser.parse_expression(min_prec)
    }

    /// Parse echo statement
    pub fn parse_echo(&mut self) -> Result<Stmt, String> {
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
    pub fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
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
    pub fn parse_if(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'if'
        self.consume(TokenKind::LeftParen, "Expected '(' after 'if'")?;
        let condition = self.parse_expression(Precedence::None)?;
        self.consume(TokenKind::RightParen, "Expected ')' after if condition")?;

        let using_alt_syntax = self.check(&TokenKind::Colon);
        let then_branch = self.parse_block()?;

        let mut elseif_branches = Vec::new();
        let mut else_branch = None;

        // Parse elseif clauses
        while self.check(&TokenKind::Elseif) {
            self.advance();
            self.consume(TokenKind::LeftParen, "Expected '(' after 'elseif'")?;
            let elseif_condition = self.parse_expression(Precedence::None)?;
            self.consume(TokenKind::RightParen, "Expected ')' after elseif condition")?;
            let elseif_body = self.parse_block()?;
            elseif_branches.push((elseif_condition, elseif_body));
        }

        // Parse else clause
        if self.check(&TokenKind::Else) {
            self.advance();

            // Check for else if (two separate tokens)
            if self.check(&TokenKind::If) {
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
                    self.advance();
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
    pub fn parse_while(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'while'
        self.consume(TokenKind::LeftParen, "Expected '(' after 'while'")?;
        let condition = self.parse_expression(Precedence::None)?;
        self.consume(TokenKind::RightParen, "Expected ')' after while condition")?;

        let using_alt_syntax = self.check(&TokenKind::Colon);
        let body = self.parse_block()?;

        if using_alt_syntax {
            if let TokenKind::Identifier(s) = &self.current().kind {
                if s.to_lowercase() == "endwhile" {
                    self.advance();
                    if self.check(&TokenKind::Semicolon) {
                        self.advance();
                    }
                }
            }
        }

        Ok(Stmt::While { condition, body })
    }

    /// Parse do-while statement
    pub fn parse_do_while(&mut self) -> Result<Stmt, String> {
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
    pub fn parse_for(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'for'
        self.consume(TokenKind::LeftParen, "Expected '(' after 'for'")?;

        let init = if !self.check(&TokenKind::Semicolon) {
            Some(self.parse_expression(Precedence::None)?)
        } else {
            None
        };
        self.consume(TokenKind::Semicolon, "Expected ';' after for init")?;

        let condition = if !self.check(&TokenKind::Semicolon) {
            Some(self.parse_expression(Precedence::None)?)
        } else {
            None
        };
        self.consume(TokenKind::Semicolon, "Expected ';' after for condition")?;

        let update = if !self.check(&TokenKind::RightParen) {
            Some(self.parse_expression(Precedence::None)?)
        } else {
            None
        };
        self.consume(TokenKind::RightParen, "Expected ')' after for clauses")?;

        let using_alt_syntax = self.check(&TokenKind::Colon);
        let body = self.parse_block()?;

        if using_alt_syntax {
            if let TokenKind::Identifier(s) = &self.current().kind {
                if s.to_lowercase() == "endfor" {
                    self.advance();
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
    pub fn parse_foreach(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'foreach'
        self.consume(TokenKind::LeftParen, "Expected '(' after 'foreach'")?;

        let array = self.parse_expression(Precedence::None)?;
        self.consume(TokenKind::As, "Expected 'as' in foreach")?;

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

        let (key, value) = if self.check(&TokenKind::DoubleArrow) {
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
        };

        self.consume(TokenKind::RightParen, "Expected ')' after foreach")?;

        let using_alt_syntax = self.check(&TokenKind::Colon);
        let body = self.parse_block()?;

        if using_alt_syntax {
            if let TokenKind::Identifier(s) = &self.current().kind {
                if s.to_lowercase() == "endforeach" {
                    self.advance();
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
    pub fn parse_switch(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'switch'
        self.consume(TokenKind::LeftParen, "Expected '(' after 'switch'")?;
        let expr = self.parse_expression(Precedence::None)?;
        self.consume(TokenKind::RightParen, "Expected ')' after switch expression")?;

        let using_alt_syntax = self.check(&TokenKind::Colon);

        if using_alt_syntax {
            self.advance();
        } else {
            self.consume(TokenKind::LeftBrace, "Expected '{' or ':' after switch")?;
        }

        let mut cases = Vec::new();
        let mut default = None;

        while !self.check(&TokenKind::RightBrace) && !self.check(&TokenKind::Eof) {
            if using_alt_syntax {
                if let TokenKind::Identifier(s) = &self.current().kind {
                    if s.to_lowercase() == "endswitch" {
                        break;
                    }
                }
            }

            if self.check(&TokenKind::Case) {
                self.advance();
                let value = self.parse_expression(Precedence::None)?;
                self.consume(TokenKind::Colon, "Expected ':' after case value")?;

                let mut body = Vec::new();
                while !self.check(&TokenKind::Case)
                    && !self.check(&TokenKind::Default)
                    && !self.check(&TokenKind::RightBrace)
                    && !self.check(&TokenKind::Eof)
                {
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
                self.advance();
                self.consume(TokenKind::Colon, "Expected ':' after 'default'")?;

                let mut body = Vec::new();
                while !self.check(&TokenKind::Case)
                    && !self.check(&TokenKind::RightBrace)
                    && !self.check(&TokenKind::Eof)
                {
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
                    self.advance();
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
    pub fn parse_break(&mut self) -> Result<Stmt, String> {
        self.advance();
        if self.check(&TokenKind::Semicolon) {
            self.advance();
        }
        Ok(Stmt::Break)
    }

    /// Parse continue statement
    pub fn parse_continue(&mut self) -> Result<Stmt, String> {
        self.advance();
        if self.check(&TokenKind::Semicolon) {
            self.advance();
        }
        Ok(Stmt::Continue)
    }

    /// Parse function declaration
    pub fn parse_function(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'function'

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

        let mut params = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            loop {
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
    pub fn parse_return(&mut self) -> Result<Stmt, String> {
        self.advance();

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

    /// Parse visibility modifier
    fn parse_visibility(&mut self) -> Visibility {
        match &self.current().kind {
            TokenKind::Public => {
                self.advance();
                Visibility::Public
            }
            TokenKind::Protected => {
                self.advance();
                Visibility::Protected
            }
            TokenKind::Private => {
                self.advance();
                Visibility::Private
            }
            _ => Visibility::Public, // Default visibility is public
        }
    }

    /// Parse class property
    fn parse_property(&mut self, visibility: Visibility) -> Result<Property, String> {
        let name = if let TokenKind::Variable(name) = &self.current().kind {
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

        let default = if self.check(&TokenKind::Assign) {
            self.advance();
            Some(self.parse_expression(Precedence::None)?)
        } else {
            None
        };

        if self.check(&TokenKind::Semicolon) {
            self.advance();
        }

        Ok(Property {
            name,
            visibility,
            default,
        })
    }

    /// Parse class method
    fn parse_method(&mut self, visibility: Visibility) -> Result<Method, String> {
        self.advance(); // consume 'function'

        let name = if let TokenKind::Identifier(name) = &self.current().kind {
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

        self.consume(TokenKind::LeftParen, "Expected '(' after method name")?;

        let mut params = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            loop {
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
        self.consume(TokenKind::LeftBrace, "Expected '{' before method body")?;

        let mut body = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.check(&TokenKind::Eof) {
            if let Some(stmt) = self.parse_statement()? {
                body.push(stmt);
            }
        }

        self.consume(TokenKind::RightBrace, "Expected '}' after method body")?;

        Ok(Method {
            name,
            visibility,
            params,
            body,
        })
    }

    /// Parse class declaration
    pub fn parse_class(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'class'

        let name = if let TokenKind::Identifier(name) = &self.current().kind {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(format!(
                "Expected class name at line {}, column {}",
                self.current().line,
                self.current().column
            ));
        };

        // Check for extends
        let parent = if self.check(&TokenKind::Extends) {
            self.advance();
            if let TokenKind::Identifier(parent_name) = &self.current().kind {
                let parent_name = parent_name.clone();
                self.advance();
                Some(parent_name)
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

        self.consume(TokenKind::LeftBrace, "Expected '{' after class name")?;

        let mut properties = Vec::new();
        let mut methods = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.check(&TokenKind::Eof) {
            let visibility = self.parse_visibility();

            if self.check(&TokenKind::Function) {
                methods.push(self.parse_method(visibility)?);
            } else if self.check(&TokenKind::Variable(String::new())) {
                properties.push(self.parse_property(visibility)?);
            } else {
                return Err(format!(
                    "Expected property or method in class at line {}, column {}",
                    self.current().line,
                    self.current().column
                ));
            }
        }

        self.consume(TokenKind::RightBrace, "Expected '}' after class body")?;

        Ok(Stmt::Class {
            name,
            parent,
            properties,
            methods,
        })
    }

    /// Parse expression statement
    pub fn parse_expression_statement(&mut self) -> Result<Stmt, String> {
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

    pub fn parse_statement(&mut self) -> Result<Option<Stmt>, String> {
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
            TokenKind::Class => Ok(Some(self.parse_class()?)),
            TokenKind::Return => Ok(Some(self.parse_return()?)),
            TokenKind::Html(html) => {
                self.advance();
                Ok(Some(Stmt::Html(html)))
            }
            TokenKind::Eof => Ok(None),
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
            | TokenKind::Identifier(_)
            | TokenKind::New => Ok(Some(self.parse_expression_statement()?)),
            _ => Err(format!(
                "Unexpected token {:?} at line {}, column {}",
                token.kind, token.line, token.column
            )),
        }
    }
}
