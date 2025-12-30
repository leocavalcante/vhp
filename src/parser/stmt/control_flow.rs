//! Control flow statement parsing
//!
//! Handles parsing of conditional and loop statements:
//! - if/elseif/else statements
//! - while loops
//! - do-while loops
//! - for loops
//! - foreach loops
//! - switch statements
//! - break and continue statements

use super::super::precedence::Precedence;
use super::StmtParser;
use crate::ast::{Stmt, SwitchCase};
use crate::token::TokenKind;

impl<'a> StmtParser<'a> {
    /// Parse if statement
    pub fn parse_if(&mut self) -> Result<Stmt, String> {
        self.parse_if_internal(false)
    }

    /// Internal parse if statement with flag for nested else-if
    fn parse_if_internal(&mut self, is_nested_else_if: bool) -> Result<Stmt, String> {
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
                let nested_if = self.parse_if_internal(true)?;
                else_branch = Some(vec![nested_if]);
            } else {
                else_branch = Some(self.parse_block()?);
            }
        }

        // Handle endif for alternative syntax (but not for nested else-if)
        if using_alt_syntax && !is_nested_else_if {
            self.consume(
                TokenKind::Endif,
                "Expected 'endif' to close alternative if syntax",
            )?;
            self.consume(TokenKind::Semicolon, "Expected ';' after 'endif'")?;
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
            self.consume(
                TokenKind::Endwhile,
                "Expected 'endwhile' to close alternative while syntax",
            )?;
            self.consume(TokenKind::Semicolon, "Expected ';' after 'endwhile'")?;
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
            self.consume(
                TokenKind::Endfor,
                "Expected 'endfor' to close alternative for syntax",
            )?;
            self.consume(TokenKind::Semicolon, "Expected ';' after 'endfor'")?;
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
            self.consume(
                TokenKind::Endforeach,
                "Expected 'endforeach' to close alternative foreach syntax",
            )?;
            self.consume(TokenKind::Semicolon, "Expected ';' after 'endforeach'")?;
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
        self.consume(
            TokenKind::RightParen,
            "Expected ')' after switch expression",
        )?;

        let using_alt_syntax = self.check(&TokenKind::Colon);

        if using_alt_syntax {
            self.advance();
        } else {
            self.consume(TokenKind::LeftBrace, "Expected '{' or ':' after switch")?;
        }

        let mut cases = Vec::new();
        let mut default = None;

        while !self.check(&TokenKind::RightBrace) && !self.check(&TokenKind::Eof) {
            if using_alt_syntax && self.check(&TokenKind::Endswitch) {
                break;
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
                    if using_alt_syntax && self.check(&TokenKind::Endswitch) {
                        break;
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
                    if using_alt_syntax && self.check(&TokenKind::Endswitch) {
                        break;
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
            self.consume(
                TokenKind::Endswitch,
                "Expected 'endswitch' to close alternative switch syntax",
            )?;
            self.consume(TokenKind::Semicolon, "Expected ';' after 'endswitch'")?;
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
}
