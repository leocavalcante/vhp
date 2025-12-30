//! Postfix operation parsing
//!
//! Handles postfix operations like array access, property access, method calls,
//! and increment/decrement operators. Postfix operations are applied after primary
//! expressions and can be chained together.

use super::ExprParser;
use crate::ast::{Expr, UnaryOp};
use crate::token::TokenKind;

/// Parse postfix operations (array access, property access, method calls, increment/decrement)
pub fn parse_postfix(parser: &mut ExprParser, mut expr: Expr) -> Result<Expr, String> {
    loop {
        match &parser.current().kind {
            TokenKind::LeftBracket => {
                parser.advance(); // consume '['

                // Check for empty brackets (append syntax: $arr[] = ...)
                if parser.check(&TokenKind::RightBracket) {
                    parser.advance(); // consume ']'
                                      // This creates an ArrayAccess with a special marker
                                      // The assignment handling will recognize this
                    expr = Expr::ArrayAccess {
                        array: Box::new(expr),
                        index: Box::new(Expr::Null), // Placeholder for append
                    };
                } else {
                    let index =
                        parser.parse_expression(super::super::precedence::Precedence::None)?;
                    parser.consume(TokenKind::RightBracket, "Expected ']' after array index")?;
                    expr = Expr::ArrayAccess {
                        array: Box::new(expr),
                        index: Box::new(index),
                    };
                }
            }
            TokenKind::Arrow => {
                parser.advance(); // consume '->'
                let member = if let TokenKind::Identifier(name) = &parser.current().kind {
                    let name = name.clone();
                    parser.advance();
                    name
                } else {
                    return Err(format!(
                        "Expected property or method name after '->' at line {}, column {}",
                        parser.current().line,
                        parser.current().column
                    ));
                };

                // Check if it's a method call or property access
                if parser.check(&TokenKind::LeftParen) {
                    parser.advance(); // consume '('

                    // Check for first-class callable: $obj->method(...)
                    // Must be ONLY ... with no other arguments
                    if parser.check(&TokenKind::Ellipsis) {
                        let start_pos = *parser.pos;
                        parser.advance(); // consume '...'

                        if parser.check(&TokenKind::RightParen) {
                            parser.advance(); // consume ')'
                            expr = Expr::CallableFromMethod {
                                object: Box::new(expr),
                                method: member,
                            };
                        } else {
                            // Not a first-class callable, rewind
                            *parser.pos = start_pos;
                            // Regular method call
                            let args = parser.parse_arguments()?;
                            parser.consume(
                                TokenKind::RightParen,
                                "Expected ')' after method arguments",
                            )?;
                            expr = Expr::MethodCall {
                                object: Box::new(expr),
                                method: member,
                                args,
                            };
                        }
                    } else {
                        // Regular method call
                        let args = parser.parse_arguments()?;
                        parser.consume(
                            TokenKind::RightParen,
                            "Expected ')' after method arguments",
                        )?;
                        expr = Expr::MethodCall {
                            object: Box::new(expr),
                            method: member,
                            args,
                        };
                    }
                } else {
                    expr = Expr::PropertyAccess {
                        object: Box::new(expr),
                        property: member,
                    };
                }
            }
            TokenKind::Increment => {
                if let Expr::Variable(_) = &expr {
                    parser.advance();
                    expr = Expr::Unary {
                        op: UnaryOp::PostInc,
                        expr: Box::new(expr),
                    };
                } else {
                    break;
                }
            }
            TokenKind::Decrement => {
                if let Expr::Variable(_) = &expr {
                    parser.advance();
                    expr = Expr::Unary {
                        op: UnaryOp::PostDec,
                        expr: Box::new(expr),
                    };
                } else {
                    break;
                }
            }
            TokenKind::LeftParen => {
                // Variable function call: $func(), closure call, etc.
                // Only allow for variables for now
                if matches!(&expr, Expr::Variable(_)) {
                    parser.advance(); // consume '('
                    let args = parser.parse_arguments()?;
                    parser.consume(TokenKind::RightParen, "Expected ')' after arguments")?;
                    expr = Expr::CallableCall {
                        callable: Box::new(expr),
                        args,
                    };
                } else {
                    break;
                }
            }
            _ => break,
        }
    }
    Ok(expr)
}
