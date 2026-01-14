//! Statement parsing
//!
//! This module provides the main statement parser dispatcher and utility functions.
//! Specialized parsing is delegated to focused submodules:
//! - control_flow: if/else, loops, switch, break/continue
//! - declarations: functions and return statements
//! - class: class definitions and members
//! - interface: interface definitions
//! - trait_: trait definitions and usage
//! - enum_: enum definitions

pub mod attribute_parsing;
pub mod class;
pub mod control_flow;
pub mod declarations;
pub mod enum_;
pub mod interface;
pub mod member_parsing;
pub mod namespace_parsing;
pub mod trait_;
pub mod type_parsing;

use super::expr::ExprParser;
use super::precedence::Precedence;
use crate::ast::{Attribute, AttributeArgument, Expr, Stmt};
use crate::token::{Token, TokenKind};

pub struct StmtParser<'a> {
    pub tokens: &'a [Token],
    pub pos: &'a mut usize,
}

impl<'a> StmtParser<'a> {
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

    pub fn parse_expression(&mut self, min_prec: Precedence) -> Result<Expr, String> {
        let mut expr_parser = ExprParser::new(self.tokens, self.pos);
        expr_parser.parse_expression(min_prec)
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
                    TokenKind::Endif
                    | TokenKind::Endwhile
                    | TokenKind::Endfor
                    | TokenKind::Endforeach
                    | TokenKind::Endswitch
                    | TokenKind::Else
                    | TokenKind::Elseif
                    | TokenKind::Case
                    | TokenKind::Default => break,
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

    /// Parse try/catch/finally statement
    /// try { ... } catch (ExceptionType $e) { ... } finally { ... }
    pub fn parse_try(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'try'

        // Parse try block
        let try_body = self.parse_block()?;

        let mut catch_clauses = Vec::new();
        let mut finally_body = None;

        // Parse catch clauses
        while self.check(&TokenKind::Catch) {
            self.advance(); // consume 'catch'
            self.consume(TokenKind::LeftParen, "Expected '(' after 'catch'")?;

            // Parse exception types (supports Type1 | Type2)
            let mut exception_types = Vec::new();
            loop {
                if let TokenKind::Identifier(name) = &self.current().kind {
                    exception_types.push(name.clone());
                    self.advance();
                } else {
                    return Err(format!(
                        "Expected exception type at line {}, column {}",
                        self.current().line,
                        self.current().column
                    ));
                }

                // Check for multi-catch separator |
                if self.check(&TokenKind::BitwiseOr) {
                    self.advance();
                } else {
                    break;
                }
            }

            // Parse variable name
            let variable = if let TokenKind::Variable(name) = &self.current().kind {
                let name = name.clone();
                self.advance();
                name
            } else {
                return Err(format!(
                    "Expected exception variable at line {}, column {}",
                    self.current().line,
                    self.current().column
                ));
            };

            self.consume(TokenKind::RightParen, "Expected ')' after catch clause")?;
            let catch_body = self.parse_block()?;

            catch_clauses.push(crate::ast::CatchClause {
                exception_types,
                variable,
                body: catch_body,
            });
        }

        // Parse optional finally
        if self.check(&TokenKind::Finally) {
            self.advance(); // consume 'finally'
            finally_body = Some(self.parse_block()?);
        }

        // Must have at least one catch or finally
        if catch_clauses.is_empty() && finally_body.is_none() {
            return Err(format!(
                "Try must have at least one catch or finally block at line {}, column {}",
                self.current().line,
                self.current().column
            ));
        }

        Ok(Stmt::TryCatch {
            try_body,
            catch_clauses,
            finally_body,
        })
    }

    /// Main statement dispatcher
    pub fn parse_statement(&mut self) -> Result<Option<Stmt>, String> {
        // Parse any attributes that may precede declarations
        let attributes = self.parse_attributes()?;

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
            TokenKind::Function => {
                let mut func = self.parse_function()?;
                if let Stmt::Function {
                    attributes: ref mut attrs,
                    ..
                } = func
                {
                    *attrs = attributes;
                }
                Ok(Some(func))
            }
            TokenKind::Class => {
                let mut class = self.parse_class()?;
                if let Stmt::Class {
                    attributes: ref mut attrs,
                    ..
                } = class
                {
                    *attrs = attributes;
                }
                Ok(Some(class))
            }
            TokenKind::Readonly => {
                // readonly can be used before class keyword (PHP 8.2)
                let mut class = self.parse_class()?;
                if let Stmt::Class {
                    attributes: ref mut attrs,
                    ..
                } = class
                {
                    *attrs = attributes;
                }
                Ok(Some(class))
            }
            TokenKind::Abstract => {
                // abstract can be used before class keyword
                let mut class = self.parse_class()?;
                if let Stmt::Class {
                    attributes: ref mut attrs,
                    ..
                } = class
                {
                    *attrs = attributes;
                }
                Ok(Some(class))
            }
            TokenKind::Final => {
                // final can be used before class keyword
                let mut class = self.parse_class()?;
                if let Stmt::Class {
                    attributes: ref mut attrs,
                    ..
                } = class
                {
                    *attrs = attributes;
                }
                Ok(Some(class))
            }
            TokenKind::Interface => {
                let mut iface = self.parse_interface()?;
                if let Stmt::Interface {
                    attributes: ref mut attrs,
                    ..
                } = iface
                {
                    *attrs = attributes;
                }
                Ok(Some(iface))
            }
            TokenKind::Trait => {
                let mut trait_stmt = self.parse_trait()?;
                if let Stmt::Trait {
                    attributes: ref mut attrs,
                    ..
                } = trait_stmt
                {
                    *attrs = attributes;
                }
                Ok(Some(trait_stmt))
            }
            TokenKind::Enum => {
                let mut enum_stmt = self.parse_enum()?;
                if let Stmt::Enum {
                    attributes: ref mut attrs,
                    ..
                } = enum_stmt
                {
                    *attrs = attributes;
                }
                Ok(Some(enum_stmt))
            }
            TokenKind::Return => Ok(Some(self.parse_return()?)),
            TokenKind::Try => Ok(Some(self.parse_try()?)),
            TokenKind::Throw => self.parse_throw(),
            TokenKind::Yield => {
                // Yield as a statement: yield $value, yield $key => $value, yield from, or yield;
                self.advance(); // consume 'yield'

                // Check for yield from $iterable
                if self.check(&TokenKind::From) {
                    self.advance(); // consume 'from'
                    let expr = self.parse_expression(Precedence::None)?;
                    if self.check(&TokenKind::Semicolon) {
                        self.advance();
                    }
                    let yield_from_expr = Expr::YieldFrom(Box::new(expr));
                    return Ok(Some(Stmt::Expression(yield_from_expr)));
                }

                // Check if this is "yield;" or "yield $expr;"
                let (key, value) = if self.check(&TokenKind::Semicolon) {
                    // yield; (yields NULL)
                    (None, None)
                } else {
                    // yield $expr or yield $key => $value
                    let key_expr = self.parse_expression(Precedence::None)?;

                    if self.check(&TokenKind::DoubleArrow) {
                        // yield $key => $value
                        self.advance(); // consume '=>'
                        let val_expr = self.parse_expression(Precedence::None)?;
                        (Some(Box::new(key_expr)), Some(Box::new(val_expr)))
                    } else {
                        // yield $expr
                        (None, Some(Box::new(key_expr)))
                    }
                };

                let yield_expr = Expr::Yield { key, value };

                if self.check(&TokenKind::Semicolon) {
                    self.advance();
                }
                Ok(Some(Stmt::Expression(yield_expr)))
            }
            TokenKind::Namespace => Ok(Some(self.parse_namespace()?)),
            TokenKind::Use => {
                // Distinguish between use statements (at top level) and trait use (in class)
                // This is a top-level use statement (namespace import)
                Ok(Some(self.parse_use_statement()?))
            }
            TokenKind::Declare => Ok(Some(self.parse_declare()?)),
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
