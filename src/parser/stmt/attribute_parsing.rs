//! Attribute parsing utilities
//!
//! Handles parsing of PHP 8.0+ attributes: #[AttributeName(args)]

use super::{Precedence, StmtParser};
use crate::ast::{Attribute, AttributeArgument, Stmt};
use crate::token::TokenKind;

impl<'a> StmtParser<'a> {
    /// Parse attributes: #[AttributeName(args)] or #[AttributeName]
    /// Can have multiple attributes: #[Attr1] #[Attr2(arg)] or #[Attr1, Attr2]
    pub(crate) fn parse_attributes(&mut self) -> Result<Vec<Attribute>, String> {
        let mut attributes = Vec::new();

        while self.check(&TokenKind::Hash) {
            let current_pos = *self.pos;
            self.advance();

            if !self.check(&TokenKind::LeftBracket) {
                *self.pos = current_pos;
                break;
            }

            self.advance();

            loop {
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

                let mut arguments = Vec::new();
                if self.check(&TokenKind::LeftParen) {
                    self.advance();

                    if !self.check(&TokenKind::RightParen) {
                        loop {
                            let mut arg_name = None;
                            if let TokenKind::Identifier(id) = &self.current().kind {
                                let lookahead_pos = *self.pos + 1;
                                if lookahead_pos < self.tokens.len() {
                                    if let TokenKind::Colon = self.tokens[lookahead_pos].kind {
                                        arg_name = Some(id.clone());
                                        self.advance();
                                        self.advance();
                                    }
                                }
                            }

                            let value = self.parse_expression(Precedence::None)?;

                            arguments.push(AttributeArgument {
                                name: arg_name,
                                value,
                            });

                            if !self.check(&TokenKind::Comma) {
                                break;
                            }
                            self.advance();
                        }
                    }

                    self.consume(
                        TokenKind::RightParen,
                        "Expected ')' after attribute arguments",
                    )?;
                }

                attributes.push(Attribute { name, arguments });

                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }

            self.consume(TokenKind::RightBracket, "Expected ']' after attribute")?;
        }

        Ok(attributes)
    }

    /// Parse echo statement
    pub(crate) fn parse_echo(&mut self) -> Result<Stmt, String> {
        self.advance();
        let mut expressions = Vec::new();

        expressions.push(self.parse_expression(Precedence::None)?);

        while self.check(&TokenKind::Comma) {
            self.advance();
            expressions.push(self.parse_expression(Precedence::None)?);
        }

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

    /// Parse throw statement
    pub(crate) fn parse_throw(&mut self) -> Result<Option<Stmt>, String> {
        self.advance();
        let expr = self.parse_expression(Precedence::None)?;

        if self.check(&TokenKind::Semicolon) {
            self.advance();
        } else if !self.check(&TokenKind::CloseTag) && !self.check(&TokenKind::Eof) {
            return Err(format!(
                "Expected ';' after throw at line {}, column {}",
                self.current().line,
                self.current().column
            ));
        }

        Ok(Some(Stmt::Throw(expr)))
    }
}
