//! Primary expression parsing
//!
//! Handles literals, variables, arrays, function calls, and basic expressions.
//! Primary expressions are the foundation for all higher-level expressions.

use super::{parse_clone, parse_match, parse_postfix, ExprParser};
use crate::ast::{Argument, ArrayElement, Expr};
use crate::token::TokenKind;

impl<'a> ExprParser<'a> {
    /// Parse function/method arguments with support for named arguments (PHP 8.0)
    /// Syntax: expr, expr, name: expr, name: expr
    /// Rule: positional arguments must come before named arguments
    pub fn parse_arguments(&mut self) -> Result<Vec<Argument>, String> {
        let mut args = Vec::new();
        let mut seen_named = false;

        if !self.check(&TokenKind::RightParen) {
            loop {
                // Try to detect named argument pattern: identifier ':'
                let is_named = if let TokenKind::Identifier(_) = &self.current().kind {
                    // Peek ahead to see if there's a colon
                    *self.pos + 1 < self.tokens.len()
                        && std::mem::discriminant(&self.tokens[*self.pos + 1].kind)
                            == std::mem::discriminant(&TokenKind::Colon)
                } else {
                    false
                };

                if is_named {
                    let name = if let TokenKind::Identifier(n) = &self.current().kind {
                        n.clone()
                    } else {
                        unreachable!()
                    };
                    self.advance(); // consume identifier
                    self.advance(); // consume ':'
                    let value =
                        self.parse_expression(super::super::precedence::Precedence::None)?;
                    args.push(Argument {
                        name: Some(name),
                        value: Box::new(value),
                    });
                    seen_named = true;
                } else {
                    if seen_named {
                        return Err(format!(
                            "Positional arguments cannot follow named arguments at line {}, column {}",
                            self.current().line,
                            self.current().column
                        ));
                    }

                    // Check for ellipsis ... (spread or placeholder)
                    if self.check(&TokenKind::Ellipsis) {
                        self.advance(); // consume ...

                        // Check if there's an expression after (spread) or not (placeholder)
                        if self.check(&TokenKind::RightParen) || self.check(&TokenKind::Comma) {
                            // This is a placeholder (...)
                            args.push(Argument {
                                name: None,
                                value: Box::new(Expr::Placeholder),
                            });
                        } else {
                            // This is a spread expression ...$array
                            let expr =
                                self.parse_expression(super::super::precedence::Precedence::None)?;
                            args.push(Argument {
                                name: None,
                                value: Box::new(Expr::Spread(Box::new(expr))),
                            });
                        }
                    } else {
                        let value =
                            self.parse_expression(super::super::precedence::Precedence::None)?;
                        args.push(Argument {
                            name: None,
                            value: Box::new(value),
                        });
                    }
                }

                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance(); // consume ','
            }
        }

        Ok(args)
    }

    /// Parse array literal: [elem1, elem2] or [key => value, ...]
    pub fn parse_array_literal(&mut self) -> Result<Expr, String> {
        self.advance(); // consume '['
        let mut elements = Vec::new();

        if !self.check(&TokenKind::RightBracket) {
            loop {
                // Parse first expression (could be key or value)
                let first = self.parse_expression(super::super::precedence::Precedence::None)?;

                // Check for => (key => value syntax)
                if self.check(&TokenKind::DoubleArrow) {
                    self.advance(); // consume '=>'
                    let value =
                        self.parse_expression(super::super::precedence::Precedence::None)?;
                    elements.push(ArrayElement {
                        key: Some(Box::new(first)),
                        value: Box::new(value),
                    });
                } else {
                    // Just a value, no explicit key
                    elements.push(ArrayElement {
                        key: None,
                        value: Box::new(first),
                    });
                }

                // Check for comma or end
                if self.check(&TokenKind::Comma) {
                    self.advance();
                    // Allow trailing comma
                    if self.check(&TokenKind::RightBracket) {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        self.consume(TokenKind::RightBracket, "Expected ']' after array elements")?;
        Ok(Expr::Array(elements))
    }

    /// Parse primary expression (literals, variables, grouped expressions)
    pub fn parse_primary(&mut self) -> Result<Expr, String> {
        let token = self.current().clone();

        match &token.kind {
            TokenKind::Integer(n) => {
                let n = *n;
                self.advance();
                parse_postfix(self, Expr::Integer(n))
            }
            TokenKind::Float(n) => {
                let n = *n;
                self.advance();
                parse_postfix(self, Expr::Float(n))
            }
            TokenKind::String(s) => {
                let s = s.clone();
                self.advance();
                parse_postfix(self, Expr::String(s))
            }
            TokenKind::True => {
                self.advance();
                parse_postfix(self, Expr::Bool(true))
            }
            TokenKind::False => {
                self.advance();
                parse_postfix(self, Expr::Bool(false))
            }
            TokenKind::Null => {
                self.advance();
                parse_postfix(self, Expr::Null)
            }
            TokenKind::LeftBracket => {
                let arr = self.parse_array_literal()?;
                parse_postfix(self, arr)
            }
            TokenKind::Variable(name) => {
                let name = name.clone();
                self.advance();
                // $this is a special reference
                let expr = if name == "this" {
                    Expr::This
                } else {
                    Expr::Variable(name)
                };
                parse_postfix(self, expr)
            }
            TokenKind::LeftParen => {
                self.advance();
                let expr = self.parse_expression(super::super::precedence::Precedence::None)?;
                self.consume(TokenKind::RightParen, "Expected ')' after expression")?;
                let grouped = Expr::Grouped(Box::new(expr));
                parse_postfix(self, grouped)
            }
            // Unary operators in primary context
            TokenKind::Minus => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: crate::ast::UnaryOp::Neg,
                    expr: Box::new(expr),
                })
            }
            TokenKind::Not => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: crate::ast::UnaryOp::Not,
                    expr: Box::new(expr),
                })
            }
            TokenKind::Increment => {
                self.advance();
                if let TokenKind::Variable(name) = &self.current().kind {
                    let name = name.clone();
                    self.advance();
                    Ok(Expr::Unary {
                        op: crate::ast::UnaryOp::PreInc,
                        expr: Box::new(Expr::Variable(name)),
                    })
                } else {
                    Err(format!(
                        "Expected variable after '++' at line {}, column {}",
                        self.current().line,
                        self.current().column
                    ))
                }
            }
            TokenKind::Decrement => {
                self.advance();
                if let TokenKind::Variable(name) = &self.current().kind {
                    let name = name.clone();
                    self.advance();
                    Ok(Expr::Unary {
                        op: crate::ast::UnaryOp::PreDec,
                        expr: Box::new(Expr::Variable(name)),
                    })
                } else {
                    Err(format!(
                        "Expected variable after '--' at line {}, column {}",
                        self.current().line,
                        self.current().column
                    ))
                }
            }
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();

                // Check for static method call (ClassName::method()), enum case access (EnumName::CASE), or static property (ClassName::$property)
                if self.check(&TokenKind::DoubleColon) {
                    self.advance(); // consume '::'

                    // Check for static property access (ClassName::$property)
                    if let TokenKind::Variable(prop_name) = &self.current().kind {
                        let prop_name = prop_name.clone();
                        self.advance();
                        let expr = Expr::StaticPropertyAccess {
                            class: name,
                            property: prop_name,
                        };
                        return parse_postfix(self, expr);
                    }

                    let method_or_case = if let TokenKind::Identifier(id) = &self.current().kind {
                        let id = id.clone();
                        self.advance();
                        id
                    } else {
                        return Err(format!(
                            "Expected method, case name, or property after '::' at line {}, column {}",
                            self.current().line,
                            self.current().column
                        ));
                    };

                    // Check if this is a method call (with parentheses) or enum case access
                    if self.check(&TokenKind::LeftParen) {
                        self.advance(); // consume '('

                        // Special case for Fiber static methods
                        if name.to_lowercase() == "fiber" {
                            match method_or_case.to_lowercase().as_str() {
                                "suspend" => {
                                    let value = if self.check(&TokenKind::RightParen) {
                                        None
                                    } else {
                                        Some(Box::new(self.parse_expression(
                                            super::super::precedence::Precedence::None,
                                        )?))
                                    };
                                    self.consume(
                                        TokenKind::RightParen,
                                        "Expected ')' after Fiber::suspend arguments",
                                    )?;
                                    let expr = Expr::FiberSuspend { value };
                                    return parse_postfix(self, expr);
                                }
                                "getcurrent" => {
                                    self.consume(
                                        TokenKind::RightParen,
                                        "Expected ')' after Fiber::getCurrent",
                                    )?;
                                    let expr = Expr::FiberGetCurrent;
                                    return parse_postfix(self, expr);
                                }
                                _ => {} // Fall through to regular static call
                            }
                        }

                        // Check for first-class callable: Class::method(...)
                        // Must be ONLY ... with no other arguments
                        if self.check(&TokenKind::Ellipsis) {
                            let start_pos = *self.pos;
                            self.advance(); // consume '...'

                            if self.check(&TokenKind::RightParen) {
                                self.advance(); // consume ')'
                                let callable = Expr::CallableFromStaticMethod {
                                    class: name,
                                    method: method_or_case,
                                };
                                return parse_postfix(self, callable);
                            } else {
                                // Not a first-class callable, rewind
                                *self.pos = start_pos;
                            }
                        }

                        // Regular static method call
                        let args = self.parse_arguments()?;
                        self.consume(
                            TokenKind::RightParen,
                            "Expected ')' after static method arguments",
                        )?;
                        let call = Expr::StaticMethodCall {
                            class_name: name,
                            method: method_or_case,
                            args,
                        };
                        parse_postfix(self, call)
                    } else {
                        // This is an enum case access (no parentheses)
                        let expr = Expr::EnumCase {
                            enum_name: name,
                            case_name: method_or_case,
                        };
                        parse_postfix(self, expr)
                    }
                } else if self.check(&TokenKind::LeftParen) {
                    // Function call or first-class callable
                    self.advance(); // consume '('

                    // Check for first-class callable: func(...)
                    // Must be ONLY ... with no other arguments
                    if self.check(&TokenKind::Ellipsis) {
                        let start_pos = *self.pos;
                        self.advance(); // consume '...'

                        // Check if this is the only thing in the parentheses
                        if self.check(&TokenKind::RightParen) {
                            self.advance(); // consume ')'
                            let callable = Expr::CallableFromFunction(name);
                            return parse_postfix(self, callable);
                        } else {
                            // Not a first-class callable, rewind and parse as regular call with spread
                            *self.pos = start_pos;
                        }
                    }

                    // Regular function call with arguments
                    let args = self.parse_arguments()?;
                    self.consume(
                        TokenKind::RightParen,
                        "Expected ')' after function arguments",
                    )?;
                    let call = Expr::FunctionCall { name, args };
                    parse_postfix(self, call)
                } else {
                    Err(format!(
                        "Unexpected identifier '{}' at line {}, column {}",
                        name, token.line, token.column
                    ))
                }
            }
            TokenKind::Fiber => {
                let name = "Fiber".to_string();
                self.advance();

                // Only allow Fiber:: static calls
                if self.check(&TokenKind::DoubleColon) {
                    self.advance(); // consume '::'
                    let method_name = if let TokenKind::Identifier(id) = &self.current().kind {
                        let id = id.clone();
                        self.advance();
                        id
                    } else {
                        return Err(format!(
                            "Expected method name after 'Fiber::' at line {}, column {}",
                            self.current().line,
                            self.current().column
                        ));
                    };

                    if self.check(&TokenKind::LeftParen) {
                        self.advance(); // consume '('

                        match method_name.to_lowercase().as_str() {
                            "suspend" => {
                                let value = if self.check(&TokenKind::RightParen) {
                                    None
                                } else {
                                    Some(Box::new(self.parse_expression(
                                        super::super::precedence::Precedence::None,
                                    )?))
                                };
                                self.consume(
                                    TokenKind::RightParen,
                                    "Expected ')' after Fiber::suspend arguments",
                                )?;
                                let expr = Expr::FiberSuspend { value };
                                parse_postfix(self, expr)
                            }
                            "getcurrent" => {
                                self.consume(
                                    TokenKind::RightParen,
                                    "Expected ')' after Fiber::getCurrent",
                                )?;
                                let expr = Expr::FiberGetCurrent;
                                parse_postfix(self, expr)
                            }
                            _ => {
                                let args = self.parse_arguments()?;
                                self.consume(
                                    TokenKind::RightParen,
                                    "Expected ')' after static method arguments",
                                )?;
                                let call = Expr::StaticMethodCall {
                                    class_name: name,
                                    method: method_name,
                                    args,
                                };
                                parse_postfix(self, call)
                            }
                        }
                    } else {
                        Err(format!(
                            "Expected '(' after 'Fiber::{}' at line {}, column {}",
                            method_name,
                            self.current().line,
                            self.current().column
                        ))
                    }
                } else {
                    Err(format!(
                        "Unexpected 'Fiber' token at line {}, column {}",
                        token.line, token.column
                    ))
                }
            }
            TokenKind::Parent => {
                self.advance(); // consume 'parent'

                // parent::method() call or parent::$property access
                if self.check(&TokenKind::DoubleColon) {
                    self.advance(); // consume '::'

                    // Check for static property access (parent::$property)
                    if let TokenKind::Variable(prop_name) = &self.current().kind {
                        let prop_name = prop_name.clone();
                        self.advance();
                        let expr = Expr::StaticPropertyAccess {
                            class: "parent".to_string(),
                            property: prop_name,
                        };
                        return parse_postfix(self, expr);
                    }

                    let method = if let TokenKind::Identifier(method_name) = &self.current().kind {
                        let method_name = method_name.clone();
                        self.advance();
                        method_name
                    } else {
                        return Err(format!(
                            "Expected method name or property after 'parent::' at line {}, column {}",
                            self.current().line,
                            self.current().column
                        ));
                    };

                    self.consume(
                        TokenKind::LeftParen,
                        "Expected '(' after parent method name",
                    )?;
                    let args = self.parse_arguments()?;
                    self.consume(
                        TokenKind::RightParen,
                        "Expected ')' after parent method arguments",
                    )?;
                    let call = Expr::StaticMethodCall {
                        class_name: "parent".to_string(),
                        method,
                        args,
                    };
                    parse_postfix(self, call)
                } else {
                    Err(format!(
                        "Expected '::' after 'parent' at line {}, column {}",
                        token.line, token.column
                    ))
                }
            }
            TokenKind::Static => {
                self.advance(); // consume 'static'

                // static::method() call or static::$property access (late static binding)
                if self.check(&TokenKind::DoubleColon) {
                    self.advance(); // consume '::'

                    // Check for static property access (static::$property)
                    if let TokenKind::Variable(prop_name) = &self.current().kind {
                        let prop_name = prop_name.clone();
                        self.advance();
                        let expr = Expr::StaticPropertyAccess {
                            class: "static".to_string(),
                            property: prop_name,
                        };
                        return parse_postfix(self, expr);
                    }

                    let method = if let TokenKind::Identifier(method_name) = &self.current().kind {
                        let method_name = method_name.clone();
                        self.advance();
                        method_name
                    } else {
                        return Err(format!(
                            "Expected method name or property after 'static::' at line {}, column {}",
                            self.current().line,
                            self.current().column
                        ));
                    };

                    self.consume(
                        TokenKind::LeftParen,
                        "Expected '(' after static method name",
                    )?;
                    let args = self.parse_arguments()?;
                    self.consume(
                        TokenKind::RightParen,
                        "Expected ')' after static method arguments",
                    )?;
                    let call = Expr::StaticMethodCall {
                        class_name: "static".to_string(),
                        method,
                        args,
                    };
                    parse_postfix(self, call)
                } else {
                    Err(format!(
                        "Expected '::' after 'static' at line {}, column {}",
                        token.line, token.column
                    ))
                }
            }
            TokenKind::New => {
                self.advance(); // consume 'new'

                // Check for anonymous class: new class ...
                if self.check(&TokenKind::Class) {
                    let anon_class = self.parse_anonymous_class()?;
                    return parse_postfix(self, anon_class);
                }

                // Parse qualified class name (e.g., \Foo\Bar or Foo\Bar\Baz)
                let mut class_name_parts = Vec::new();
                let is_fully_qualified = if self.check(&TokenKind::Backslash) {
                    self.advance();
                    true
                } else {
                    false
                };

                let class_name = match &self.current().kind {
                    TokenKind::Identifier(name) => {
                        class_name_parts.push(name.clone());
                        self.advance();

                        // Continue parsing backslash-separated parts
                        while self.check(&TokenKind::Backslash) {
                            self.advance();
                            if let TokenKind::Identifier(part) = &self.current().kind {
                                class_name_parts.push(part.clone());
                                self.advance();
                            } else {
                                return Err(format!(
                                    "Expected identifier after '\\' at line {}, column {}",
                                    self.current().line,
                                    self.current().column
                                ));
                            }
                        }

                        // Construct the full class name
                        if is_fully_qualified {
                            format!("\\{}", class_name_parts.join("\\"))
                        } else {
                            class_name_parts.join("\\")
                        }
                    }
                    TokenKind::Fiber => {
                        self.advance();
                        "Fiber".to_string()
                    }
                    _ => {
                        return Err(format!(
                            "Expected class name after 'new' at line {}, column {}",
                            self.current().line,
                            self.current().column
                        ));
                    }
                };

                // Special case for Fiber constructor
                if class_name.to_lowercase() == "fiber" {
                    self.consume(TokenKind::LeftParen, "Expected '(' after 'new Fiber'")?;
                    let callback =
                        self.parse_expression(super::super::precedence::Precedence::None)?;
                    self.consume(TokenKind::RightParen, "Expected ')' after fiber callback")?;
                    let fiber_expr = Expr::NewFiber {
                        callback: Box::new(callback),
                    };
                    return parse_postfix(self, fiber_expr);
                }

                // Regular class instantiation
                let mut args = Vec::new();
                if self.check(&TokenKind::LeftParen) {
                    self.advance(); // consume '('
                    args = self.parse_arguments()?;
                    self.consume(
                        TokenKind::RightParen,
                        "Expected ')' after constructor arguments",
                    )?;
                }

                let new_expr = Expr::New { class_name, args };
                parse_postfix(self, new_expr)
            }
            TokenKind::Clone => {
                self.advance(); // consume 'clone'
                let clone_expr = parse_clone(self)?;
                parse_postfix(self, clone_expr)
            }
            TokenKind::Match => {
                let match_expr = parse_match(self)?;
                parse_postfix(self, match_expr)
            }
            TokenKind::Fn => {
                self.advance(); // consume 'fn'
                let arrow_func = self.parse_arrow_function()?;
                parse_postfix(self, arrow_func)
            }
            TokenKind::Throw => {
                self.advance(); // consume 'throw'
                let expr = self.parse_unary()?;
                Ok(Expr::Throw(Box::new(expr)))
            }
            TokenKind::Yield => {
                self.advance(); // consume 'yield'
                let mut key: Option<Box<Expr>> = None;
                let mut value: Option<Box<Expr>> = None;

                // Check for yield from: yield from $expr
                if self.check(&TokenKind::From) {
                    self.advance(); // consume 'from'
                    let expr = self.parse_unary()?;
                    return Ok(Expr::YieldFrom(Box::new(expr)));
                }

                // Check for yield $key => $value syntax
                // We need to parse an expression, check if it's followed by '=>'
                let first_expr = self.parse_unary()?;
                if self.check(&TokenKind::DoubleArrow) {
                    self.advance(); // consume '=>'
                    key = Some(Box::new(first_expr));
                    value = Some(Box::new(self.parse_unary()?));
                } else {
                    value = Some(Box::new(first_expr));
                }

                Ok(Expr::Yield { key, value })
            }
            _ => Err(format!(
                "Expected expression but found {:?} at line {}, column {}",
                token.kind, token.line, token.column
            )),
        }
    }

    /// Parse arrow function: fn(params) => expression
    /// PHP 7.4+ feature for short closures
    pub fn parse_arrow_function(&mut self) -> Result<Expr, String> {
        // 'fn' already consumed

        self.consume(TokenKind::LeftParen, "Expected '(' after 'fn'")?;

        // Parse parameters (simplified version without type hints)
        let mut params = Vec::new();

        if !self.check(&TokenKind::RightParen) {
            loop {
                // Parse by-reference &
                let by_ref = if self.check(&TokenKind::And) {
                    self.advance();
                    true
                } else {
                    false
                };

                // Parse ellipsis for variadic
                let is_variadic = if self.check(&TokenKind::Ellipsis) {
                    self.advance();
                    true
                } else {
                    false
                };

                // Parse parameter name
                let param_name = if let TokenKind::Variable(name) = &self.current().kind {
                    let n = name.clone();
                    self.advance();
                    n
                } else {
                    return Err(format!(
                        "Expected parameter name at line {}, column {}",
                        self.current().line,
                        self.current().column
                    ));
                };

                // Parse default value
                let default = if self.check(&TokenKind::Assign) {
                    self.advance();
                    Some(self.parse_expression(super::super::precedence::Precedence::None)?)
                } else {
                    None
                };

                params.push(crate::ast::FunctionParam {
                    name: param_name,
                    type_hint: None,
                    default,
                    by_ref,
                    is_variadic,
                    visibility: None,
                    readonly: false,
                    attributes: Vec::new(),
                });

                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }

        self.consume(TokenKind::RightParen, "Expected ')' after parameters")?;

        // Expect => (fat arrow / double arrow)
        self.consume(
            TokenKind::DoubleArrow,
            "Expected '=>' after arrow function parameters",
        )?;

        // Parse the expression body (NOT a statement block)
        let body = self.parse_expression(super::super::precedence::Precedence::None)?;

        Ok(Expr::ArrowFunction {
            params,
            body: Box::new(body),
        })
    }

    /// Parse anonymous class: new class(...) extends X implements Y { ... }
    pub fn parse_anonymous_class(&mut self) -> Result<Expr, String> {
        self.consume(TokenKind::Class, "Expected 'class'")?;

        // Parse optional constructor arguments: new class(arg1, arg2)
        let constructor_args = if self.check(&TokenKind::LeftParen) {
            self.advance();
            let args = self.parse_arguments()?;
            self.consume(
                TokenKind::RightParen,
                "Expected ')' after constructor arguments",
            )?;
            args
        } else {
            vec![]
        };

        // Parse optional extends
        let parent = if self.check(&TokenKind::Extends) {
            self.advance();
            if let TokenKind::Identifier(name) = &self.current().kind {
                let name = name.clone();
                self.advance();
                Some(name)
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

        // Parse optional implements
        let mut interfaces = vec![];
        if self.check(&TokenKind::Implements) {
            self.advance();
            loop {
                if let TokenKind::Identifier(name) = &self.current().kind {
                    interfaces.push(name.clone());
                    self.advance();
                } else {
                    return Err(format!(
                        "Expected interface name at line {}, column {}",
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

        // Parse class body using StmtParser
        self.consume(
            TokenKind::LeftBrace,
            "Expected '{' for anonymous class body",
        )?;

        // Create a StmtParser to parse class members
        let mut stmt_parser = crate::parser::stmt::StmtParser::new(self.tokens, self.pos);

        let mut traits = vec![];
        let mut properties = vec![];
        let mut methods = vec![];

        while !stmt_parser.check(&TokenKind::RightBrace) && !stmt_parser.check(&TokenKind::Eof) {
            // Parse class member
            if stmt_parser.check(&TokenKind::Use) {
                // Parse trait use
                stmt_parser.advance();
                let mut trait_names = vec![];
                loop {
                    if let TokenKind::Identifier(name) = &stmt_parser.current().kind {
                        trait_names.push(name.clone());
                        stmt_parser.advance();
                    } else {
                        return Err(format!(
                            "Expected trait name at line {}, column {}",
                            stmt_parser.current().line,
                            stmt_parser.current().column
                        ));
                    }

                    if !stmt_parser.check(&TokenKind::Comma) {
                        break;
                    }
                    stmt_parser.advance();
                }
                stmt_parser.consume(TokenKind::Semicolon, "Expected ';' after trait use")?;
                traits.push(crate::ast::TraitUse {
                    traits: trait_names,
                    resolutions: vec![],
                });
            } else {
                // Parse visibility and other modifiers
                let mut visibility = crate::ast::Visibility::Public;
                let mut is_abstract = false;
                let mut is_final = false;

                loop {
                    if stmt_parser.check(&TokenKind::Public) {
                        visibility = crate::ast::Visibility::Public;
                        stmt_parser.advance();
                    } else if stmt_parser.check(&TokenKind::Protected) {
                        visibility = crate::ast::Visibility::Protected;
                        stmt_parser.advance();
                    } else if stmt_parser.check(&TokenKind::Private) {
                        visibility = crate::ast::Visibility::Private;
                        stmt_parser.advance();
                    } else if stmt_parser.check(&TokenKind::Abstract) {
                        is_abstract = true;
                        stmt_parser.advance();
                    } else if stmt_parser.check(&TokenKind::Final) {
                        is_final = true;
                        stmt_parser.advance();
                    } else {
                        break;
                    }
                }

                if stmt_parser.check(&TokenKind::Function) {
                    // It's a method
                    let method = stmt_parser.parse_method(visibility, is_abstract, is_final)?;
                    methods.push(method);
                } else {
                    // It's a property
                    let property = stmt_parser.parse_property(visibility)?;
                    properties.push(property);
                }
            }
        }

        stmt_parser.consume(
            TokenKind::RightBrace,
            "Expected '}' after anonymous class body",
        )?;

        Ok(Expr::NewAnonymousClass {
            constructor_args,
            parent,
            interfaces,
            traits,
            properties,
            methods,
        })
    }
}
