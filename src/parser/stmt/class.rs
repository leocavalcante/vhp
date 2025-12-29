//! Class definition parsing
//!
//! Handles parsing of class declarations including:
//! - Class declaration with extends/implements
//! - Class properties
//! - Class methods
//! - Visibility modifiers
//! - Trait usage within classes

use crate::ast::Stmt;
use crate::token::TokenKind;
use super::StmtParser;

impl<'a> StmtParser<'a> {
    /// Parse class declaration
    pub fn parse_class(&mut self) -> Result<Stmt, String> {
        // Check for readonly modifier before class keyword
        let readonly = if self.check(&TokenKind::Readonly) {
            self.advance();
            true
        } else {
            false
        };

        self.consume(TokenKind::Class, "Expected 'class' keyword")?;

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

        // Check for implements
        let mut interfaces = Vec::new();
        if self.check(&TokenKind::Implements) {
            self.advance();
            loop {
                if let TokenKind::Identifier(iface) = &self.current().kind {
                    interfaces.push(iface.clone());
                    self.advance();
                } else {
                    return Err(format!(
                        "Expected interface name after 'implements' at line {}, column {}",
                        self.current().line,
                        self.current().column
                    ));
                }

                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }

        self.consume(TokenKind::LeftBrace, "Expected '{' after class name")?;

        // Parse trait uses at the start of class body
        let mut trait_uses = Vec::new();
        while self.check(&TokenKind::Use) {
            trait_uses.push(self.parse_trait_use()?);
        }

        let mut properties = Vec::new();
        let mut methods = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.check(&TokenKind::Eof) {
            // Parse attributes that may precede property or method
            let attributes = self.parse_attributes()?;

            // Check for readonly modifier (can appear before visibility)
            let readonly_first = if self.check(&TokenKind::Readonly) {
                self.advance();
                true
            } else {
                false
            };

            let visibility = self.parse_visibility();

            // Check for readonly modifier if not already found (can appear after visibility)
            let readonly = readonly_first || if self.check(&TokenKind::Readonly) {
                self.advance();
                true
            } else {
                false
            };

            // Skip type hints (not supported yet, but we need to skip them)
            // Common PHP types: string, int, float, bool, array, object, mixed, etc.
            if let TokenKind::Identifier(type_name) = &self.current().kind {
                let type_lower = type_name.to_lowercase();
                if matches!(type_lower.as_str(),
                    "string" | "int" | "float" | "bool" | "array" | "object" | "mixed" |
                    "callable" | "iterable" | "void" | "never" | "true" | "false" | "null" |
                    "self" | "parent" | "static") {
                    // Skip the type
                    self.advance();
                    // Handle array type brackets if present
                    if self.check(&TokenKind::LeftBracket) {
                        self.advance();
                        self.consume(TokenKind::RightBracket, "Expected ']' after array type")?;
                    }
                }
            }

            if self.check(&TokenKind::Function) {
                let mut method = self.parse_method(visibility)?;
                method.attributes = attributes;
                methods.push(method);
            } else if self.check(&TokenKind::Variable(String::new())) {
                // Parse property with readonly modifier
                let mut prop = self.parse_property(visibility)?;
                prop.readonly = readonly;
                prop.attributes = attributes;
                properties.push(prop);
            } else {
                return Err(format!(
                    "Expected property or method in class at line {}, column {}",
                    self.current().line,
                    self.current().column
                ));
            }
        }

        self.consume(TokenKind::RightBrace, "Expected '}' after class body")?;

        // If class is readonly, validate that properties don't have explicit readonly modifier
        if readonly {
            for property in &properties {
                if property.readonly {
                    return Err(format!(
                        "Property '{}' cannot have explicit 'readonly' modifier in readonly class '{}' at line {}, column {}",
                        property.name,
                        name,
                        self.current().line,
                        self.current().column
                    ));
                }
            }
        }

        Ok(Stmt::Class {
            name,
            readonly,
            parent,
            interfaces,
            trait_uses,
            properties,
            methods,
            attributes: Vec::new(),
        })
    }

}
