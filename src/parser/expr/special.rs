//! Special expression parsing
//!
//! Handles complex expressions like match (PHP 8.0), clone with modifications (PHP 8.1),
//! and parenthesized expressions.

use super::ExprParser;
use crate::ast::{Expr, ListElement, MatchArm, PropertyModification};
use crate::token::TokenKind;

/// Parse match expression: match ($expr) { cond1, cond2 => result, default => result }
pub fn parse_match(parser: &mut ExprParser) -> Result<Expr, String> {
    parser.advance(); // consume 'match'
    parser.consume(TokenKind::LeftParen, "Expected '(' after 'match'")?;
    let expr = parser.parse_expression(super::super::precedence::Precedence::None)?;
    parser.consume(TokenKind::RightParen, "Expected ')' after match expression")?;
    parser.consume(TokenKind::LeftBrace, "Expected '{' to start match body")?;

    let mut arms = Vec::new();
    let mut default: Option<Box<Expr>> = None;

    while !parser.check(&TokenKind::RightBrace) && !parser.check(&TokenKind::Eof) {
        // Check for default arm
        if parser.check(&TokenKind::Default) {
            parser.advance(); // consume 'default'
            parser.consume(TokenKind::DoubleArrow, "Expected '=>' after 'default'")?;
            let result = parser.parse_expression(super::super::precedence::Precedence::None)?;
            default = Some(Box::new(result));

            // Optional comma after arm
            if parser.check(&TokenKind::Comma) {
                parser.advance();
            }
            continue;
        }

        // Parse conditions (can be multiple, comma-separated before =>)
        let mut conditions = Vec::new();
        conditions.push(parser.parse_expression(super::super::precedence::Precedence::None)?);

        // Handle multiple conditions: 1, 2 => ...
        while parser.check(&TokenKind::Comma) {
            // Peek ahead to see if this is a continuation of conditions or the next arm
            let next_pos = *parser.pos + 1;
            if next_pos < parser.tokens.len() {
                // Check if after comma we have '=>' (end of conditions) or '}' (malformed)
                // or another expression (more conditions)
                let after_comma = &parser.tokens[next_pos].kind;
                if matches!(after_comma, TokenKind::RightBrace | TokenKind::Default) {
                    break;
                }
            }
            parser.advance(); // consume comma

            // If next is '=>', we're done with conditions for this arm
            if parser.check(&TokenKind::DoubleArrow) {
                break;
            }

            conditions.push(parser.parse_expression(super::super::precedence::Precedence::None)?);
        }

        parser.consume(
            TokenKind::DoubleArrow,
            "Expected '=>' after match condition(s)",
        )?;
        let result = parser.parse_expression(super::super::precedence::Precedence::None)?;

        arms.push(MatchArm {
            conditions,
            result: Box::new(result),
        });

        // Optional comma after arm
        if parser.check(&TokenKind::Comma) {
            parser.advance();
        }
    }

    parser.consume(TokenKind::RightBrace, "Expected '}' to end match body")?;

    Ok(Expr::Match {
        expr: Box::new(expr),
        arms,
        default,
    })
}

/// Parse clone or clone with expression
/// clone $obj
/// clone $obj with { prop: value, ... }
pub fn parse_clone(parser: &mut ExprParser) -> Result<Expr, String> {
    // Parse the object expression
    let object = Box::new(parser.parse_unary()?);

    // Check if followed by 'with'
    if parser.check(&TokenKind::With) {
        parser.advance(); // consume 'with'

        // Expect opening brace
        if !parser.check(&TokenKind::LeftBrace) {
            return Err(format!(
                "Expected '{{' after 'with' at line {}",
                parser.current().line
            ));
        }
        parser.advance(); // consume '{'

        let mut modifications = Vec::new();

        // Parse property modifications
        loop {
            // Check for closing brace
            if parser.check(&TokenKind::RightBrace) {
                parser.advance();
                break;
            }

            // Parse property name (identifier)
            let property = match &parser.current().kind {
                TokenKind::Identifier(name) => name.clone(),
                _ => {
                    return Err(format!(
                        "Expected property name at line {}",
                        parser.current().line
                    ))
                }
            };
            parser.advance();

            // Expect colon
            if !parser.check(&TokenKind::Colon) {
                return Err(format!(
                    "Expected ':' after property name at line {}",
                    parser.current().line
                ));
            }
            parser.advance(); // consume ':'

            // Parse value expression
            let value =
                Box::new(parser.parse_expression(super::super::precedence::Precedence::None)?);

            modifications.push(PropertyModification { property, value });

            // Check for comma or closing brace
            if parser.check(&TokenKind::Comma) {
                parser.advance();
                // Allow trailing comma before closing brace
                if parser.check(&TokenKind::RightBrace) {
                    parser.advance();
                    break;
                }
            } else if parser.check(&TokenKind::RightBrace) {
                parser.advance();
                break;
            } else {
                return Err(format!(
                    "Expected ',' or '}}' after property value at line {}",
                    parser.current().line
                ));
            }
        }

        if modifications.is_empty() {
            return Err(format!(
                "Clone with syntax requires at least one property modification at line {}",
                parser.current().line
            ));
        }

        Ok(Expr::CloneWith {
            object,
            modifications,
        })
    } else {
        // Simple clone without modifications
        Ok(Expr::Clone { object })
    }
}

/// Parse list() destructuring: list($a, $b) = $array
/// Supports: list($a, $b), list("key" => $a, "b" => $b), list($a, list($b, $c))
pub fn parse_list(parser: &mut ExprParser) -> Result<Expr, String> {
    parser.advance(); // consume 'list'

    parser.consume(TokenKind::LeftParen, "Expected '(' after 'list'")?;

    let mut elements = Vec::new();

    // Handle empty list: list()
    if !parser.check(&TokenKind::RightParen) {
        loop {
            if parser.check(&TokenKind::RightParen) {
                break;
            }

            // Check for key => value syntax
            if parser.check(&TokenKind::DoubleArrow) {
                return Err(format!(
                    "Unexpected '=>' in list at line {}, column {}",
                    parser.current().line,
                    parser.current().column
                ));
            }

            // Check if this is a nested list or a variable
            if parser.check(&TokenKind::Identifier(String::new())) {
                let ident = match &parser.current().kind {
                    TokenKind::Identifier(name) => name.clone(),
                    _ => unreachable!(),
                };

                // Check if it's actually 'list' for nested destructuring
                if ident.to_lowercase() == "list" {
                    // Parse nested list
                    let nested = parse_list(parser)?;
                    elements.push(ListElement {
                        key: None,
                        value: Box::new(nested),
                    });
                } else {
                    return Err(format!(
                        "Expected variable or 'list' in list() at line {}, column {}",
                        parser.current().line,
                        parser.current().column
                    ));
                }
            } else if parser.check(&TokenKind::Variable(String::new())) {
                // Simple variable: $a
                if let TokenKind::Variable(name) = &parser.current().kind {
                    let name = name.clone();
                    parser.advance();
                    elements.push(ListElement {
                        key: None,
                        value: Box::new(Expr::Variable(name)),
                    });
                }
            } else if parser.check(&TokenKind::String(String::new())) {
                // Key => variable syntax: "key" => $var
                // Parse the key
                let key_token = parser.current().clone();
                let key = match &key_token.kind {
                    TokenKind::String(s) => {
                        let k = s.clone();
                        parser.advance();
                        k
                    }
                    _ => unreachable!(),
                };

                // Expect =>
                parser.consume(
                    TokenKind::DoubleArrow,
                    "Expected '=>' after string key in list()",
                )?;

                // Parse the value (must be variable or nested list)
                if parser.check(&TokenKind::Identifier(String::new())) {
                    let ident = match &parser.current().kind {
                        TokenKind::Identifier(name) => name.clone(),
                        _ => unreachable!(),
                    };

                    if ident.to_lowercase() == "list" {
                        let nested = parse_list(parser)?;
                        elements.push(ListElement {
                            key: Some(Box::new(Expr::String(key))),
                            value: Box::new(nested),
                        });
                    } else {
                        return Err(format!(
                            "Expected 'list' after '=>' in list() at line {}, column {}",
                            parser.current().line,
                            parser.current().column
                        ));
                    }
                } else if parser.check(&TokenKind::Variable(String::new())) {
                    if let TokenKind::Variable(name) = &parser.current().kind {
                        let name = name.clone();
                        parser.advance();
                        elements.push(ListElement {
                            key: Some(Box::new(Expr::String(key))),
                            value: Box::new(Expr::Variable(name)),
                        });
                    }
                } else {
                    return Err(format!(
                        "Expected variable or 'list' after '=>' in list() at line {}, column {}",
                        parser.current().line,
                        parser.current().column
                    ));
                }
            } else {
                return Err(format!(
                    "Expected variable or 'list' in list() at line {}, column {}",
                    parser.current().line,
                    parser.current().column
                ));
            }

            // Check for comma or closing paren
            if parser.check(&TokenKind::Comma) {
                parser.advance();
                if parser.check(&TokenKind::RightParen) {
                    break;
                }
            } else if parser.check(&TokenKind::RightParen) {
                break;
            } else {
                return Err(format!(
                    "Expected ',' or ')' in list() at line {}, column {}",
                    parser.current().line,
                    parser.current().column
                ));
            }
        }
    }

    parser.consume(TokenKind::RightParen, "Expected ')' to close list()")?;

    // Create a placeholder that will be used in assignment context
    // The actual array expression comes after the = operator
    Ok(Expr::ListDestructure {
        elements,
        array: Box::new(Expr::Null), // Placeholder, will be replaced during assignment parsing
    })
}
