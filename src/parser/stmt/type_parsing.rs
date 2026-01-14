//! Type hint parsing
//!
//! Handles parsing of PHP type hints including:
//! - Simple types: int, string, float, bool, array, callable, etc.
//! - Nullable types: ?int, ?string, etc.
//! - Union types: int|string, int|float|null
//! - Intersection types: Iterator&Countable
//! - DNF types: (A&B)|C, (A&B)|(C&D)

use super::StmtParser;
use crate::ast::TypeHint;

impl<'a> StmtParser<'a> {
    /// Parse a type hint
    /// Supports: int, string, ?int, int|string, array, callable, ClassName, Iterator&Countable, (A&B)|C
    pub fn parse_type_hint(&mut self) -> Result<TypeHint, String> {
        let nullable = if self.check(&crate::token::TokenKind::QuestionMark) {
            self.advance();
            true
        } else {
            false
        };

        let base_type = self.parse_type_component()?;

        if self.check(&crate::token::TokenKind::BitwiseOr) {
            let types_or_dnf = self.parse_union_or_dnf(base_type)?;
            if nullable {
                return Err(
                    "Cannot use nullable syntax with union types, use |null instead".to_string(),
                );
            }
            return Ok(types_or_dnf);
        }

        if let crate::token::TokenKind::Identifier(next_id) = &self
            .tokens
            .get(*self.pos)
            .map(|t| &t.kind)
            .unwrap_or(&crate::token::TokenKind::Eof)
        {
            if next_id == "&" {
                if let Some(after_amp) = self.tokens.get(*self.pos + 1) {
                    if matches!(after_amp.kind, crate::token::TokenKind::Identifier(_)) {
                        let mut types = vec![base_type.clone()];
                        while let crate::token::TokenKind::Identifier(amp) = &self.current().kind {
                            if amp == "&" {
                                self.advance();
                                types.push(self.parse_single_type()?);
                            } else {
                                break;
                            }
                        }
                        if types.len() > 1 {
                            if nullable {
                                return Err("Cannot use nullable syntax with intersection types"
                                    .to_string());
                            }
                            return Ok(TypeHint::Intersection(types));
                        }
                    }
                }
            }
        }

        if nullable {
            Ok(TypeHint::Nullable(Box::new(base_type)))
        } else {
            Ok(base_type)
        }
    }

    /// Parse a type component - either a single type or a parenthesized intersection
    fn parse_type_component(&mut self) -> Result<TypeHint, String> {
        if self.check(&crate::token::TokenKind::LeftParen) {
            self.advance();

            let first_type = self.parse_single_type()?;
            let mut types = vec![first_type];

            while let crate::token::TokenKind::Identifier(amp) = &self.current().kind {
                if amp == "&" {
                    self.advance();
                    types.push(self.parse_single_type()?);
                } else {
                    break;
                }
            }

            self.consume(
                crate::token::TokenKind::RightParen,
                "Expected ')' after type in parentheses",
            )?;

            if types.len() > 1 {
                Ok(TypeHint::Intersection(types))
            } else {
                Ok(types.into_iter().next().unwrap())
            }
        } else {
            self.parse_single_type()
        }
    }

    /// Parse union or DNF type after seeing the first component and |
    fn parse_union_or_dnf(&mut self, first: TypeHint) -> Result<TypeHint, String> {
        let mut components = vec![first.clone()];
        let has_intersection = matches!(first, TypeHint::Intersection(_));

        while self.check(&crate::token::TokenKind::BitwiseOr) {
            self.advance();

            let component = self.parse_type_component()?;

            if matches!(component, TypeHint::Intersection(_)) {
                // Keep track that we've seen an intersection
            }

            components.push(component);
        }

        if has_intersection {
            let dnf_components: Result<Vec<Vec<TypeHint>>, String> = components
                .into_iter()
                .map(|comp| match comp {
                    TypeHint::Intersection(types) => Ok(types),
                    other => Ok(vec![other]),
                })
                .collect();

            Ok(TypeHint::DNF(dnf_components?))
        } else {
            Ok(TypeHint::Union(components))
        }
    }

    /// Parse a single type (without union/intersection)
    fn parse_single_type(&mut self) -> Result<TypeHint, String> {
        if let crate::token::TokenKind::Identifier(name) = &self.current().kind {
            let type_name = name.to_lowercase();
            let original_name = name.clone();
            self.advance();

            match type_name.as_str() {
                "int" | "integer" => Ok(TypeHint::Simple("int".to_string())),
                "string" => Ok(TypeHint::Simple("string".to_string())),
                "float" | "double" => Ok(TypeHint::Simple("float".to_string())),
                "bool" | "boolean" => Ok(TypeHint::Simple("bool".to_string())),
                "array" => Ok(TypeHint::Simple("array".to_string())),
                "object" => Ok(TypeHint::Simple("object".to_string())),
                "callable" => Ok(TypeHint::Simple("callable".to_string())),
                "iterable" => Ok(TypeHint::Simple("iterable".to_string())),
                "mixed" => Ok(TypeHint::Simple("mixed".to_string())),
                "void" => Ok(TypeHint::Void),
                "never" => Ok(TypeHint::Never),
                "static" => Ok(TypeHint::Static),
                "self" => Ok(TypeHint::SelfType),
                "parent" => Ok(TypeHint::ParentType),
                "null" => Ok(TypeHint::Simple("null".to_string())),
                "false" => Ok(TypeHint::Simple("false".to_string())),
                "true" => Ok(TypeHint::Simple("true".to_string())),
                _ => Ok(TypeHint::Class(original_name)),
            }
        } else {
            Err(format!("Expected type name, got {:?}", self.current().kind))
        }
    }
}
