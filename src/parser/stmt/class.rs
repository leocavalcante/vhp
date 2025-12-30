//! Class definition parsing
//!
//! Handles parsing of class declarations including:
//! - Class declaration with extends/implements
//! - Class properties
//! - Class methods
//! - Visibility modifiers
//! - Trait usage within classes
//! - Abstract classes and methods

use super::StmtParser;
use crate::ast::Stmt;
use crate::token::TokenKind;

impl<'a> StmtParser<'a> {
    /// Parse class declaration
    pub fn parse_class(&mut self) -> Result<Stmt, String> {
        // Parse class modifiers in any order: abstract, final, readonly
        let mut is_abstract = false;
        let mut is_final = false;
        let mut readonly = false;

        loop {
            if self.check(&TokenKind::Abstract) {
                if is_final {
                    return Err(format!(
                        "Cannot use 'abstract' with 'final' at line {}, column {}",
                        self.current().line,
                        self.current().column
                    ));
                }
                is_abstract = true;
                self.advance();
            } else if self.check(&TokenKind::Final) {
                if is_abstract {
                    return Err(format!(
                        "Cannot use 'final' with 'abstract' at line {}, column {}",
                        self.current().line,
                        self.current().column
                    ));
                }
                is_final = true;
                self.advance();
            } else if self.check(&TokenKind::Readonly) {
                readonly = true;
                self.advance();
            } else {
                break;
            }
        }

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
            Some(self.parse_qualified_name()?)
        } else {
            None
        };

        // Check for implements
        let mut interfaces = Vec::new();
        if self.check(&TokenKind::Implements) {
            self.advance();
            loop {
                interfaces.push(self.parse_qualified_name()?);

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

            // Check for abstract modifier
            let member_is_abstract = if self.check(&TokenKind::Abstract) {
                if !is_abstract {
                    return Err(format!(
                        "Non-abstract class '{}' cannot contain abstract methods at line {}, column {}",
                        name,
                        self.current().line,
                        self.current().column
                    ));
                }
                self.advance();
                true
            } else {
                false
            };

            // Check for final modifier for method
            let member_is_final = if self.check(&TokenKind::Final) {
                if member_is_abstract {
                    return Err(format!(
                        "Cannot use 'final' with 'abstract' at line {}, column {}",
                        self.current().line,
                        self.current().column
                    ));
                }
                self.advance();
                true
            } else {
                false
            };

            // Check for readonly modifier (can appear before visibility)
            let readonly_first = if self.check(&TokenKind::Readonly) {
                self.advance();
                true
            } else {
                false
            };

            let visibility = self.parse_visibility();

            // Check for readonly modifier if not already found (can appear after visibility)
            let readonly = readonly_first
                || if self.check(&TokenKind::Readonly) {
                    self.advance();
                    true
                } else {
                    false
                };
            
            // Check for static modifier
            let is_static = if let TokenKind::Identifier(s) = &self.current().kind {
                if s.to_lowercase() == "static" {
                    self.advance();
                    true
                } else {
                    false
                }
            } else {
                false
            };

            // Parse type hints if present (for property types)
            // Note: Currently type hints are parsed but not yet enforced for properties
            let _property_type = if let TokenKind::Identifier(_) = &self.current().kind {
                Some(self.parse_type_hint()?)
            } else if self.check(&TokenKind::QuestionMark) {
                Some(self.parse_type_hint()?)
            } else {
                None
            };

            if self.check(&TokenKind::Function) {
                let mut method = self.parse_method(visibility, member_is_abstract, member_is_final)?;
                method.is_static = is_static;
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
            is_abstract,
            is_final,
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
