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

pub mod class;
pub mod control_flow;
pub mod declarations;
pub mod enum_;
pub mod interface;
pub mod trait_;

use super::expr::ExprParser;
use super::precedence::Precedence;
use crate::ast::{Attribute, AttributeArgument, Expr, Method, Property, Stmt, Visibility};
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

    /// Parse attributes: #[AttributeName(args)] or #[AttributeName]
    /// Can have multiple attributes: #[Attr1] #[Attr2(arg)] or #[Attr1, Attr2]
    pub fn parse_attributes(&mut self) -> Result<Vec<Attribute>, String> {
        let mut attributes = Vec::new();

        // Keep parsing while we see #[
        while self.check(&TokenKind::Hash) {
            // Check if next token is [
            let current_pos = *self.pos;
            self.advance(); // consume '#'

            if !self.check(&TokenKind::LeftBracket) {
                // Not an attribute, restore position
                *self.pos = current_pos;
                break;
            }

            self.advance(); // consume '['

            // Parse comma-separated list of attributes within the same #[...]
            loop {
                // Parse attribute name (identifier)
                let name = if let TokenKind::Identifier(name) = &self.current().kind {
                    let name = name.clone();
                    self.advance();
                    name
                } else {
                    return Err(format!(
                        "Expected attribute name at line {}, column {}",
                        self.current().line,
                        self.current().column
                    ));
                };

                // Parse optional arguments
                let mut arguments = Vec::new();
                if self.check(&TokenKind::LeftParen) {
                    self.advance(); // consume '('

                    if !self.check(&TokenKind::RightParen) {
                        loop {
                            // Check for named argument (name: value)
                            let mut arg_name = None;
                            if let TokenKind::Identifier(id) = &self.current().kind {
                                // Look ahead for colon
                                let lookahead_pos = *self.pos + 1;
                                if lookahead_pos < self.tokens.len() {
                                    if let TokenKind::Colon = self.tokens[lookahead_pos].kind {
                                        // This is a named argument
                                        arg_name = Some(id.clone());
                                        self.advance(); // consume identifier
                                        self.advance(); // consume ':'
                                    }
                                }
                            }

                            // Parse argument value
                            let value = self.parse_expression(Precedence::None)?;

                            arguments.push(AttributeArgument {
                                name: arg_name,
                                value,
                            });

                            if !self.check(&TokenKind::Comma) {
                                break;
                            }
                            self.advance(); // consume ','
                        }
                    }

                    self.consume(
                        TokenKind::RightParen,
                        "Expected ')' after attribute arguments",
                    )?;
                }

                attributes.push(Attribute { name, arguments });

                // Check for comma (multiple attributes in same #[...])
                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance(); // consume ','
            }

            self.consume(TokenKind::RightBracket, "Expected ']' after attribute")?;
        }

        Ok(attributes)
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
                            || s.to_lowercase() == "endswitch" =>
                    {
                        break
                    }
                    TokenKind::Else | TokenKind::Elseif | TokenKind::Case | TokenKind::Default => {
                        break
                    }
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

    /// Parse visibility modifier (public, private, protected)
    /// Used by class, trait, and enum parsing
    pub fn parse_visibility(&mut self) -> Visibility {
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

    /// Parse class property (shared between class and trait)
    pub fn parse_property(&mut self, visibility: Visibility) -> Result<Property, String> {
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
            readonly: false,        // Will be set by caller if needed
            attributes: Vec::new(), // Will be set by caller
        })
    }

    /// Parse class method (shared between class and trait)
    pub fn parse_method(&mut self, visibility: Visibility, is_abstract_method: bool) -> Result<Method, String> {
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

        // Detect if this is a constructor
        let is_constructor = name.to_lowercase() == "__construct";

        self.consume(TokenKind::LeftParen, "Expected '(' after method name")?;

        let mut params = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            loop {
                // Parse attributes for this parameter
                let param_attributes = self.parse_attributes()?;

                // Check for visibility modifiers only on constructors
                let param_visibility = if is_constructor {
                    match &self.current().kind {
                        TokenKind::Public => {
                            self.advance();
                            Some(Visibility::Public)
                        }
                        TokenKind::Protected => {
                            self.advance();
                            Some(Visibility::Protected)
                        }
                        TokenKind::Private => {
                            self.advance();
                            Some(Visibility::Private)
                        }
                        _ => None,
                    }
                } else {
                    // Error if visibility is used on non-constructor method
                    match &self.current().kind {
                        TokenKind::Public | TokenKind::Protected | TokenKind::Private => {
                            return Err(format!(
                                "Constructor property promotion can only be used in __construct at line {}, column {}",
                                self.current().line,
                                self.current().column
                            ));
                        }
                        _ => None,
                    }
                };

                // Check for readonly modifier (only in constructors with visibility)
                let param_readonly = if is_constructor && param_visibility.is_some() {
                    if self.check(&TokenKind::Readonly) {
                        self.advance();
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                // Skip type hints (not supported yet)
                if let TokenKind::Identifier(type_name) = &self.current().kind {
                    let type_lower = type_name.to_lowercase();
                    if matches!(
                        type_lower.as_str(),
                        "string"
                            | "int"
                            | "float"
                            | "bool"
                            | "array"
                            | "object"
                            | "mixed"
                            | "callable"
                            | "iterable"
                            | "void"
                            | "never"
                            | "true"
                            | "false"
                            | "null"
                            | "self"
                            | "parent"
                            | "static"
                    ) {
                        // Skip the type
                        self.advance();
                        // Handle array type brackets if present
                        if self.check(&TokenKind::LeftBracket) {
                            self.advance();
                            self.consume(TokenKind::RightBracket, "Expected ']' after array type")?;
                        }
                    }
                }

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

                params.push(crate::ast::FunctionParam {
                    name: param_name,
                    default,
                    by_ref,
                    visibility: param_visibility,
                    readonly: param_readonly,
                    attributes: param_attributes,
                });

                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }

        self.consume(TokenKind::RightParen, "Expected ')' after parameters")?;
        
        // Skip return type hint if present (: type)
        if self.check(&TokenKind::Colon) {
            self.advance(); // consume ':'
            // Skip the type identifier
            if let TokenKind::Identifier(_) = &self.current().kind {
                self.advance();
            } else if self.check(&TokenKind::QuestionMark) {
                // Nullable type ?Type
                self.advance();
                if let TokenKind::Identifier(_) = &self.current().kind {
                    self.advance();
                }
            }
        }

        // Abstract methods end with semicolon, concrete methods have body
        let body = if is_abstract_method {
            self.consume(TokenKind::Semicolon, "Expected ';' after abstract method declaration")?;
            Vec::new()
        } else {
            self.consume(TokenKind::LeftBrace, "Expected '{' before method body")?;

            let mut body = Vec::new();
            while !self.check(&TokenKind::RightBrace) && !self.check(&TokenKind::Eof) {
                if let Some(stmt) = self.parse_statement()? {
                    body.push(stmt);
                }
            }

            self.consume(TokenKind::RightBrace, "Expected '}' after method body")?;
            body
        };

        Ok(Method {
            name,
            visibility,
            is_static: false, // Will be set by caller if needed
            is_abstract: is_abstract_method,
            params,
            body,
            attributes: Vec::new(), // Will be set by caller
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
