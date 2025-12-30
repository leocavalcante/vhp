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
use crate::ast::{Stmt, Visibility};
use crate::token::TokenKind;

impl<'a> StmtParser<'a> {
    /// Check if current token is an identifier with a specific value
    #[allow(dead_code)]
    fn check_identifier(&self, expected: &str) -> bool {
        if let TokenKind::Identifier(name) = &self.current().kind {
            name.eq_ignore_ascii_case(expected)
        } else {
            false
        }
    }

    /// Validate that write visibility is more restrictive than read visibility
    fn validate_asymmetric_visibility(
        &self,
        read: Visibility,
        write: Visibility,
    ) -> Result<(), String> {
        use crate::ast::Visibility::*;

        let valid = match (read, write) {
            (Public, Public) => false,       // No point - same visibility
            (Public, Protected) => true,     // OK: write is more restrictive
            (Public, Private) => true,       // OK: write is more restrictive
            (Protected, Public) => false,    // Invalid: write is less restrictive
            (Protected, Protected) => false, // No point - same visibility
            (Protected, Private) => true,    // OK: write is more restrictive
            (Private, Public) => false,      // Invalid: write is less restrictive
            (Private, Protected) => false,   // Invalid: write is less restrictive
            (Private, Private) => false,     // No point - same visibility
        };

        if !valid {
            Err(format!(
                "Write visibility must be more restrictive than read visibility at line {}, column {}",
                self.current().line,
                self.current().column
            ))
        } else {
            Ok(())
        }
    }

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

            // Parse first visibility modifier
            let first_visibility = self.parse_visibility();

            // Check for asymmetric visibility: read_visibility write_visibility(set)
            // Example: public private(set) means public read, private write
            let (read_visibility, write_visibility) = if self.check(&TokenKind::Public)
                || self.check(&TokenKind::Protected)
                || self.check(&TokenKind::Private)
            {
                // Found a second visibility keyword - this might be asymmetric visibility
                let second_vis = self.parse_visibility();

                // Check if followed by (set)
                if self.check(&TokenKind::LeftParen) {
                    self.advance(); // consume '('

                    if self.check(&TokenKind::Set) {
                        self.advance(); // consume 'set'

                        self.consume(
                            TokenKind::RightParen,
                            "Expected ')' after 'set' in asymmetric visibility",
                        )?;

                        // first_visibility is read, second_vis is write
                        (first_visibility, Some(second_vis))
                    } else {
                        return Err(format!(
                            "Expected 'set' after '(' in asymmetric visibility at line {}, column {}",
                            self.current().line,
                            self.current().column
                        ));
                    }
                } else {
                    return Err(format!(
                        "Expected '(set)' after second visibility modifier at line {}, column {}",
                        self.current().line,
                        self.current().column
                    ));
                }
            } else {
                (first_visibility, None)
            };

            // Validate asymmetric visibility: write must be more restrictive than read
            if let Some(write_vis) = write_visibility {
                self.validate_asymmetric_visibility(read_visibility, write_vis)?;
            }

            // Check for readonly modifier if not already found (can appear after visibility)
            let readonly = readonly_first
                || if self.check(&TokenKind::Readonly) {
                    self.advance();
                    true
                } else {
                    false
                };

            // Check for static modifier
            let is_static = if self.check(&TokenKind::Static) {
                self.advance();
                true
            } else {
                false
            };

            // Check for readonly modifier if not already found (can appear after static too)
            let readonly = readonly
                || if self.check(&TokenKind::Readonly) {
                    self.advance();
                    true
                } else {
                    false
                };

            // Validate: readonly and asymmetric visibility are incompatible
            if readonly && write_visibility.is_some() {
                return Err(format!(
                    "Readonly properties cannot have asymmetric visibility at line {}, column {}",
                    self.current().line,
                    self.current().column
                ));
            }

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
                let mut method =
                    self.parse_method(read_visibility, member_is_abstract, member_is_final)?;
                method.is_static = is_static;
                method.attributes = attributes;
                methods.push(method);
            } else if self.check(&TokenKind::Variable(String::new())) {
                // Parse property with readonly and static modifiers
                let mut prop = self.parse_property(read_visibility)?;
                prop.write_visibility = write_visibility;
                prop.readonly = readonly;
                prop.is_static = is_static;
                prop.attributes = attributes;

                // Validation: property hooks and asymmetric visibility are incompatible
                if !prop.hooks.is_empty() && write_visibility.is_some() {
                    return Err(format!(
                        "Property hooks cannot be combined with asymmetric visibility at line {}, column {}",
                        self.current().line,
                        self.current().column
                    ));
                }

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
