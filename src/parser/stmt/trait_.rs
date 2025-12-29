//! Trait definition parsing
//!
//! Handles parsing of trait declarations including:
//! - Trait declaration
//! - Trait usage (use statements)
//! - Trait conflict resolution (insteadof, as)
//! - Trait properties and methods

use super::StmtParser;
use crate::ast::{Stmt, TraitResolution, TraitUse};
use crate::token::TokenKind;

impl<'a> StmtParser<'a> {
    /// Parse trait declaration
    pub fn parse_trait(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'trait'

        let name = if let TokenKind::Identifier(name) = &self.current().kind {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(format!(
                "Expected trait name at line {}, column {}",
                self.current().line,
                self.current().column
            ));
        };

        self.consume(TokenKind::LeftBrace, "Expected '{' after trait name")?;

        // Parse trait uses at the start of trait body
        let mut uses = Vec::new();
        while self.check(&TokenKind::Use) {
            let trait_use = self.parse_trait_use()?;
            uses.extend(trait_use.traits);
        }

        let mut properties = Vec::new();
        let mut methods = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.check(&TokenKind::Eof) {
            // Parse attributes that may precede property or method
            let attributes = self.parse_attributes()?;

            let visibility = self.parse_visibility();

            if self.check(&TokenKind::Function) {
                let mut method = self.parse_method(visibility, false, false)?; // traits don't have abstract/final methods
                method.attributes = attributes;
                methods.push(method);
            } else if self.check(&TokenKind::Variable(String::new())) {
                let mut prop = self.parse_property(visibility)?;
                prop.attributes = attributes;
                properties.push(prop);
            } else {
                return Err(format!(
                    "Expected property or method in trait at line {}, column {}",
                    self.current().line,
                    self.current().column
                ));
            }
        }

        self.consume(TokenKind::RightBrace, "Expected '}' after trait body")?;

        Ok(Stmt::Trait {
            name,
            uses,
            properties,
            methods,
            attributes: Vec::new(),
        })
    }

    /// Parse trait use statement
    pub fn parse_trait_use(&mut self) -> Result<TraitUse, String> {
        self.advance(); // consume 'use'

        let mut traits = Vec::new();
        loop {
            if let TokenKind::Identifier(trait_name) = &self.current().kind {
                traits.push(trait_name.clone());
                self.advance();
            } else {
                return Err(format!(
                    "Expected trait name after 'use' at line {}, column {}",
                    self.current().line,
                    self.current().column
                ));
            }

            if !self.check(&TokenKind::Comma) {
                break;
            }
            self.advance();
        }

        let mut resolutions = Vec::new();

        // Check for conflict resolution (insteadof/as clauses)
        if self.check(&TokenKind::LeftBrace) {
            self.advance();

            while !self.check(&TokenKind::RightBrace) && !self.check(&TokenKind::Eof) {
                resolutions.push(self.parse_trait_resolution()?);
            }

            self.consume(
                TokenKind::RightBrace,
                "Expected '}' after trait resolutions",
            )?;
        } else {
            // No braces means this is a simple use; without resolutions
            self.consume(TokenKind::Semicolon, "Expected ';' after trait use")?;
        }

        Ok(TraitUse {
            traits,
            resolutions,
        })
    }

    /// Parse trait resolution (insteadof or as clause)
    fn parse_trait_resolution(&mut self) -> Result<TraitResolution, String> {
        let first_id = if let TokenKind::Identifier(name) = &self.current().kind {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(format!(
                "Expected name in resolution at line {}, column {}",
                self.current().line,
                self.current().column
            ));
        };

        // Check if we have Trait::method or just method
        let (trait_name, method) = if self.check(&TokenKind::DoubleColon) {
            self.advance();
            let method_name = if let TokenKind::Identifier(name) = &self.current().kind {
                let name = name.clone();
                self.advance();
                name
            } else {
                return Err(format!(
                    "Expected method name after '::' at line {}, column {}",
                    self.current().line,
                    self.current().column
                ));
            };
            (Some(first_id), method_name)
        } else {
            // Just method name without trait prefix (for simple as aliases)
            (None, first_id)
        };

        if self.check(&TokenKind::Insteadof) {
            self.advance();

            let mut excluded_traits = Vec::new();
            loop {
                if let TokenKind::Identifier(excluded_trait) = &self.current().kind {
                    excluded_traits.push(excluded_trait.clone());
                    self.advance();
                } else {
                    return Err(format!(
                        "Expected trait name after 'insteadof' at line {}, column {}",
                        self.current().line,
                        self.current().column
                    ));
                }

                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }

            // Consume optional semicolon at the end of resolution
            if self.check(&TokenKind::Semicolon) {
                self.advance();
            }

            Ok(TraitResolution::InsteadOf {
                trait_name: trait_name.unwrap_or_else(String::new),
                method,
                excluded_traits,
            })
        } else if self.check(&TokenKind::As) {
            self.advance();

            let visibility = if self.check(&TokenKind::Public)
                || self.check(&TokenKind::Protected)
                || self.check(&TokenKind::Private)
            {
                Some(self.parse_visibility())
            } else {
                None
            };

            let alias = if let TokenKind::Identifier(name) = &self.current().kind {
                let name = name.clone();
                self.advance();
                name
            } else {
                return Err(format!(
                    "Expected alias name after 'as' at line {}, column {}",
                    self.current().line,
                    self.current().column
                ));
            };

            // Consume optional semicolon at the end of resolution
            if self.check(&TokenKind::Semicolon) {
                self.advance();
            }

            Ok(TraitResolution::Alias {
                trait_name,
                method,
                alias,
                visibility,
            })
        } else {
            Err(format!(
                "Expected 'insteadof' or 'as' in trait resolution at line {}, column {}",
                self.current().line,
                self.current().column
            ))
        }
    }
}
