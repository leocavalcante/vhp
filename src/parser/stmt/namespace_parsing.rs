//! Namespace, declare, and use statement parsing
//!
//! Handles parsing of PHP namespace declarations, declare directives,
//! and use statements (imports).

use super::StmtParser;
use crate::ast::{DeclareDirective, NamespaceBody, Stmt, UseItem, UseType};
use crate::token::TokenKind;

impl<'a> StmtParser<'a> {
    /// Parse namespace declaration
    /// namespace Foo\Bar { ... } or namespace Foo\Bar; or namespace { ... }
    pub fn parse_namespace(&mut self) -> Result<Stmt, String> {
        self.advance();

        let name = if self.check(&TokenKind::LeftBrace) || self.check(&TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_qualified_name()?)
        };

        let body = if self.check(&TokenKind::LeftBrace) {
            self.advance();
            let mut stmts = Vec::new();
            while !self.check(&TokenKind::RightBrace) && !self.check(&TokenKind::Eof) {
                if let Some(stmt) = self.parse_statement()? {
                    stmts.push(stmt);
                }
            }
            self.consume(TokenKind::RightBrace, "Expected '}' after namespace body")?;
            NamespaceBody::Braced(stmts)
        } else {
            self.consume(
                TokenKind::Semicolon,
                "Expected ';' or '{' after namespace declaration",
            )?;
            NamespaceBody::Unbraced
        };

        Ok(Stmt::Namespace { name, body })
    }

    /// Parse qualified name (e.g., Foo\Bar\Baz or \Foo\Bar\Baz)
    pub fn parse_qualified_name(&mut self) -> Result<crate::ast::QualifiedName, String> {
        use crate::ast::QualifiedName;

        let is_fully_qualified = if self.check(&TokenKind::Backslash) {
            self.advance();
            true
        } else {
            false
        };

        let mut parts = vec![];

        if let TokenKind::Identifier(name) = &self.current().kind {
            parts.push(name.clone());
            self.advance();
        } else {
            return Err(format!(
                "Expected identifier in qualified name at line {}, column {}",
                self.current().line,
                self.current().column
            ));
        }

        while self.check(&TokenKind::Backslash) {
            let next_idx = *self.pos + 1;
            if next_idx < self.tokens.len() {
                if let TokenKind::LeftBrace = &self.tokens[next_idx].kind {
                    break;
                }
            }

            self.advance();
            if let TokenKind::Identifier(name) = &self.current().kind {
                parts.push(name.clone());
                self.advance();
            } else {
                return Err(format!(
                    "Expected identifier after '\\' at line {}, column {}",
                    self.current().line,
                    self.current().column
                ));
            }
        }

        Ok(QualifiedName::new(parts, is_fully_qualified))
    }

    /// Parse declare statement
    /// declare(strict_types=1); or declare(strict_types=1) { ... }
    pub fn parse_declare(&mut self) -> Result<Stmt, String> {
        self.advance();
        self.consume(TokenKind::LeftParen, "Expected '(' after 'declare'")?;

        let mut directives = vec![];

        loop {
            let name = if let TokenKind::Identifier(name) = &self.current().kind {
                let name = name.clone();
                self.advance();
                name
            } else {
                return Err(format!(
                    "Expected directive name at line {}, column {}",
                    self.current().line,
                    self.current().column
                ));
            };

            self.consume(TokenKind::Assign, "Expected '=' after directive name")?;

            let directive = match name.to_lowercase().as_str() {
                "strict_types" => {
                    let value =
                        self.parse_expression(super::super::precedence::Precedence::None)?;
                    match value {
                        crate::ast::Expr::Integer(0) => DeclareDirective::StrictTypes(false),
                        crate::ast::Expr::Integer(1) => DeclareDirective::StrictTypes(true),
                        _ => {
                            return Err(format!(
                                "strict_types value must be 0 or 1 at line {}, column {}",
                                self.current().line,
                                self.current().column
                            ))
                        }
                    }
                }
                "encoding" => {
                    let value =
                        self.parse_expression(super::super::precedence::Precedence::None)?;
                    match value {
                        crate::ast::Expr::String(s) => DeclareDirective::Encoding(s),
                        _ => {
                            return Err(format!(
                                "encoding value must be a string at line {}, column {}",
                                self.current().line,
                                self.current().column
                            ))
                        }
                    }
                }
                "ticks" => {
                    let value =
                        self.parse_expression(super::super::precedence::Precedence::None)?;
                    match value {
                        crate::ast::Expr::Integer(n) => DeclareDirective::Ticks(n),
                        _ => {
                            return Err(format!(
                                "ticks value must be an integer at line {}, column {}",
                                self.current().line,
                                self.current().column
                            ))
                        }
                    }
                }
                _ => {
                    return Err(format!(
                        "Unknown declare directive: {} at line {}, column {}",
                        name,
                        self.current().line,
                        self.current().column
                    ))
                }
            };

            directives.push(directive);

            if !self.check(&TokenKind::Comma) {
                break;
            }
            self.advance();
        }

        self.consume(
            TokenKind::RightParen,
            "Expected ')' after declare directives",
        )?;

        let body = if self.check(&TokenKind::LeftBrace) {
            self.advance();
            let mut stmts = Vec::new();
            while !self.check(&TokenKind::RightBrace) && !self.check(&TokenKind::Eof) {
                if let Some(stmt) = self.parse_statement()? {
                    stmts.push(stmt);
                }
            }
            self.consume(TokenKind::RightBrace, "Expected '}' after declare block")?;
            Some(stmts)
        } else if self.check(&TokenKind::Colon) {
            self.advance();
            let mut stmts = Vec::new();
            loop {
                if let TokenKind::Identifier(id) = &self.current().kind {
                    if id.to_lowercase() == "enddeclare" {
                        self.advance();
                        break;
                    }
                }
                if self.check(&TokenKind::Eof) {
                    return Err("Expected 'enddeclare' to close declare statement".to_string());
                }
                if let Some(stmt) = self.parse_statement()? {
                    stmts.push(stmt);
                }
            }
            self.consume(TokenKind::Semicolon, "Expected ';' after 'enddeclare'")?;
            Some(stmts)
        } else {
            self.consume(TokenKind::Semicolon, "Expected ';' after declare")?;
            None
        };

        Ok(Stmt::Declare { directives, body })
    }

    /// Parse use statement
    /// use Foo\Bar; use Foo\Bar as Baz; use function Foo\bar; use const Foo\BAR;
    /// use Foo\{Bar, Baz}; (group use)
    pub fn parse_use_statement(&mut self) -> Result<Stmt, String> {
        self.advance();

        let default_type = if self.check(&TokenKind::Function) {
            self.advance();
            UseType::Function
        } else if self.check(&TokenKind::Const) {
            self.advance();
            UseType::Constant
        } else {
            UseType::Class
        };

        let name = self.parse_qualified_name()?;

        if self.check(&TokenKind::Backslash) {
            self.advance();
            if !self.check(&TokenKind::LeftBrace) {
                return Err(format!(
                    "Expected '{{' after '\\' in use statement at line {}, column {}",
                    self.current().line,
                    self.current().column
                ));
            }
        }

        if self.check(&TokenKind::LeftBrace) {
            return self.parse_group_use(name, default_type);
        }

        let mut items = vec![];

        let alias = if self.check(&TokenKind::As) {
            self.advance();
            if let TokenKind::Identifier(alias_name) = &self.current().kind {
                let alias_name = alias_name.clone();
                self.advance();
                Some(alias_name)
            } else {
                return Err(format!(
                    "Expected identifier after 'as' at line {}, column {}",
                    self.current().line,
                    self.current().column
                ));
            }
        } else {
            None
        };

        items.push(UseItem {
            name: name.clone(),
            alias,
            use_type: default_type.clone(),
        });

        while self.check(&TokenKind::Comma) {
            self.advance();
            let name = self.parse_qualified_name()?;
            let alias = if self.check(&TokenKind::As) {
                self.advance();
                if let TokenKind::Identifier(alias_name) = &self.current().kind {
                    let alias_name = alias_name.clone();
                    self.advance();
                    Some(alias_name)
                } else {
                    return Err(format!(
                        "Expected identifier after 'as' at line {}, column {}",
                        self.current().line,
                        self.current().column
                    ));
                }
            } else {
                None
            };
            items.push(UseItem {
                name,
                alias,
                use_type: default_type.clone(),
            });
        }

        self.consume(TokenKind::Semicolon, "Expected ';' after use statement")?;
        Ok(Stmt::Use(items))
    }

    /// Parse group use statement: use Foo\{Bar, Baz};
    fn parse_group_use(
        &mut self,
        prefix: crate::ast::QualifiedName,
        default_type: crate::ast::UseType,
    ) -> Result<Stmt, String> {
        use crate::ast::{GroupUse, UseItem, UseType};

        self.advance();

        let mut items = vec![];

        loop {
            let use_type = if self.check(&TokenKind::Function) {
                self.advance();
                UseType::Function
            } else if self.check(&TokenKind::Const) {
                self.advance();
                UseType::Constant
            } else {
                default_type.clone()
            };

            let name = self.parse_qualified_name()?;
            let alias = if self.check(&TokenKind::As) {
                self.advance();
                if let TokenKind::Identifier(alias_name) = &self.current().kind {
                    let alias_name = alias_name.clone();
                    self.advance();
                    Some(alias_name)
                } else {
                    return Err(format!(
                        "Expected identifier after 'as' at line {}, column {}",
                        self.current().line,
                        self.current().column
                    ));
                }
            } else {
                None
            };

            items.push(UseItem {
                name,
                alias,
                use_type,
            });

            if !self.check(&TokenKind::Comma) {
                break;
            }
            self.advance();
        }

        self.consume(TokenKind::RightBrace, "Expected '}' after group use items")?;
        self.consume(TokenKind::Semicolon, "Expected ';' after use statement")?;

        Ok(Stmt::GroupUse(GroupUse { prefix, items }))
    }
}
