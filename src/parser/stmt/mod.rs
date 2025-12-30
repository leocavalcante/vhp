//! Statement parsing
//!
//! This module provides the main statement parser dispatcher and utility functions.
//! Specialized parsing is delegated to focused submodules:
//! - control_flow: if/else, loops, switch, break/continue
//! - declarations: functions and return statements
//! - class: class definitions and members
//! - interface: interface definitions
//! - trait_: trait definitions and usage
//! - enum_: enum definitions

pub mod class;
pub mod control_flow;
pub mod declarations;
pub mod enum_;
pub mod interface;
pub mod trait_;

use super::expr::ExprParser;
use super::precedence::Precedence;
use crate::ast::{
    Attribute, AttributeArgument, DeclareDirective, Expr, Method, Property, PropertyHook,
    PropertyHookBody, PropertyHookType, Stmt, Visibility,
};
use crate::token::{Token, TokenKind};

pub struct StmtParser<'a> {
    pub tokens: &'a [Token],
    pub pos: &'a mut usize,
}

impl<'a> StmtParser<'a> {
    pub fn new(tokens: &'a [Token], pos: &'a mut usize) -> Self {
        Self { tokens, pos }
    }

    pub fn current(&self) -> &Token {
        self.tokens.get(*self.pos).unwrap_or(&Token {
            kind: TokenKind::Eof,
            line: 0,
            column: 0,
        })
    }

    pub fn advance(&mut self) -> Token {
        let token = self.current().clone();
        if *self.pos < self.tokens.len() {
            *self.pos += 1;
        }
        token
    }

    pub fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.current().kind) == std::mem::discriminant(kind)
    }

    pub fn consume(&mut self, kind: TokenKind, msg: &str) -> Result<Token, String> {
        if self.check(&kind) {
            Ok(self.advance())
        } else {
            Err(format!(
                "{} at line {}, column {} (found {:?})",
                msg,
                self.current().line,
                self.current().column,
                self.current().kind
            ))
        }
    }

    pub fn parse_expression(&mut self, min_prec: Precedence) -> Result<Expr, String> {
        let mut expr_parser = ExprParser::new(self.tokens, self.pos);
        expr_parser.parse_expression(min_prec)
    }

    /// Parse attributes: #[AttributeName(args)] or #[AttributeName]
    /// Can have multiple attributes: #[Attr1] #[Attr2(arg)] or #[Attr1, Attr2]
    pub fn parse_attributes(&mut self) -> Result<Vec<Attribute>, String> {
        let mut attributes = Vec::new();

        // Keep parsing while we see #[
        while self.check(&TokenKind::Hash) {
            // Check if next token is [
            let current_pos = *self.pos;
            self.advance(); // consume '#'

            if !self.check(&TokenKind::LeftBracket) {
                // Not an attribute, restore position
                *self.pos = current_pos;
                break;
            }

            self.advance(); // consume '['

            // Parse comma-separated list of attributes within the same #[...]
            loop {
                // Parse attribute name (identifier)
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

                // Parse optional arguments
                let mut arguments = Vec::new();
                if self.check(&TokenKind::LeftParen) {
                    self.advance(); // consume '('

                    if !self.check(&TokenKind::RightParen) {
                        loop {
                            // Check for named argument (name: value)
                            let mut arg_name = None;
                            if let TokenKind::Identifier(id) = &self.current().kind {
                                // Look ahead for colon
                                let lookahead_pos = *self.pos + 1;
                                if lookahead_pos < self.tokens.len() {
                                    if let TokenKind::Colon = self.tokens[lookahead_pos].kind {
                                        // This is a named argument
                                        arg_name = Some(id.clone());
                                        self.advance(); // consume identifier
                                        self.advance(); // consume ':'
                                    }
                                }
                            }

                            // Parse argument value
                            let value = self.parse_expression(Precedence::None)?;

                            arguments.push(AttributeArgument {
                                name: arg_name,
                                value,
                            });

                            if !self.check(&TokenKind::Comma) {
                                break;
                            }
                            self.advance(); // consume ','
                        }
                    }

                    self.consume(
                        TokenKind::RightParen,
                        "Expected ')' after attribute arguments",
                    )?;
                }

                attributes.push(Attribute { name, arguments });

                // Check for comma (multiple attributes in same #[...])
                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance(); // consume ','
            }

            self.consume(TokenKind::RightBracket, "Expected ']' after attribute")?;
        }

        Ok(attributes)
    }

    /// Parse a type hint
    /// Supports: int, string, ?int, int|string, array, callable, ClassName, Iterator&Countable
    pub fn parse_type_hint(&mut self) -> Result<crate::ast::TypeHint, String> {
        use crate::ast::TypeHint;

        // Check for nullable prefix ?
        let nullable = if self.check(&TokenKind::QuestionMark) {
            self.advance();
            true
        } else {
            false
        };

        // Parse the base type
        let base_type = self.parse_single_type()?;

        // Check for union | or intersection &
        if self.check(&TokenKind::BitwiseOr) {
            let mut types = vec![base_type.clone()];
            while self.check(&TokenKind::BitwiseOr) {
                self.advance();
                types.push(self.parse_single_type()?);
            }
            if nullable {
                // ?int|string is not valid syntax, but int|string|null is
                return Err(
                    "Cannot use nullable syntax with union types, use |null instead".to_string(),
                );
            }
            return Ok(TypeHint::Union(types));
        }

        // Check for intersection & (we need to distinguish from reference &)
        // In type hints, & is used for intersection types
        // We check if the next token looks like a type name
        if let TokenKind::Identifier(next_id) = &self
            .tokens
            .get(*self.pos)
            .map(|t| &t.kind)
            .unwrap_or(&TokenKind::Eof)
        {
            if next_id == "&" {
                // This looks like it could be an intersection type
                // But we need to be more careful - check if what follows is a type name
                if let Some(after_amp) = self.tokens.get(*self.pos + 1) {
                    if matches!(after_amp.kind, TokenKind::Identifier(_)) {
                        let mut types = vec![base_type.clone()];
                        while let TokenKind::Identifier(amp) = &self.current().kind {
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

        // Apply nullable wrapper if needed
        if nullable {
            Ok(TypeHint::Nullable(Box::new(base_type)))
        } else {
            Ok(base_type)
        }
    }

    /// Parse a single type (without union/intersection)
    fn parse_single_type(&mut self) -> Result<crate::ast::TypeHint, String> {
        use crate::ast::TypeHint;

        if let TokenKind::Identifier(name) = &self.current().kind {
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
                _ => Ok(TypeHint::Class(original_name)), // Class/interface name
            }
        } else {
            Err(format!("Expected type name, got {:?}", self.current().kind))
        }
    }

    /// Parse echo statement
    pub fn parse_echo(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'echo'
        let mut expressions = Vec::new();

        expressions.push(self.parse_expression(Precedence::None)?);

        while self.check(&TokenKind::Comma) {
            self.advance();
            expressions.push(self.parse_expression(Precedence::None)?);
        }

        // Semicolon is optional before close tag
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

    /// Parse a block of statements enclosed in braces or a single statement
    pub fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        if self.check(&TokenKind::LeftBrace) {
            self.advance(); // consume '{'
            let mut statements = Vec::new();

            while !self.check(&TokenKind::RightBrace) && !self.check(&TokenKind::Eof) {
                if let Some(stmt) = self.parse_statement()? {
                    statements.push(stmt);
                }
            }

            self.consume(TokenKind::RightBrace, "Expected '}' after block")?;
            Ok(statements)
        } else if self.check(&TokenKind::Colon) {
            // Alternative syntax: if (...): ... endif;
            self.advance(); // consume ':'
            let mut statements = Vec::new();

            while !self.check(&TokenKind::Eof) {
                match &self.current().kind {
                    TokenKind::Endif
                    | TokenKind::Endwhile
                    | TokenKind::Endfor
                    | TokenKind::Endforeach
                    | TokenKind::Endswitch
                    | TokenKind::Else
                    | TokenKind::Elseif
                    | TokenKind::Case
                    | TokenKind::Default => break,
                    _ => {}
                }

                if let Some(stmt) = self.parse_statement()? {
                    statements.push(stmt);
                }
            }

            Ok(statements)
        } else {
            // Single statement
            let mut statements = Vec::new();
            if let Some(stmt) = self.parse_statement()? {
                statements.push(stmt);
            }
            Ok(statements)
        }
    }

    /// Parse visibility modifier (public, private, protected)
    /// Used by class, trait, and enum parsing
    pub fn parse_visibility(&mut self) -> Visibility {
        match &self.current().kind {
            TokenKind::Public => {
                self.advance();
                Visibility::Public
            }
            TokenKind::Protected => {
                self.advance();
                Visibility::Protected
            }
            TokenKind::Private => {
                self.advance();
                Visibility::Private
            }
            _ => Visibility::Public, // Default visibility is public
        }
    }

    /// Parse class property (shared between class and trait)
    pub fn parse_property(&mut self, visibility: Visibility) -> Result<Property, String> {
        let name = if let TokenKind::Variable(name) = &self.current().kind {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(format!(
                "Expected property name at line {}, column {}",
                self.current().line,
                self.current().column
            ));
        };

        // Check for property hooks (PHP 8.4)
        if self.check(&TokenKind::LeftBrace) {
            let hooks = self.parse_property_hooks()?;
            return Ok(Property {
                name,
                visibility,
                write_visibility: None, // Hooks and asymmetric visibility are incompatible
                default: None, // Properties with hooks cannot have default values
                readonly: false,
                is_static: false,
                attributes: Vec::new(),
                hooks,
            });
        }

        // Parse optional default value: = expr
        let default = if self.check(&TokenKind::Assign) {
            self.advance();
            Some(self.parse_expression(Precedence::None)?)
        } else {
            None
        };

        if self.check(&TokenKind::Semicolon) {
            self.advance();
        }

        Ok(Property {
            name,
            visibility,
            write_visibility: None, // Will be set by caller if needed
            default,
            readonly: false,        // Will be set by caller if needed
            is_static: false,       // Will be set by caller if needed
            attributes: Vec::new(), // Will be set by caller
            hooks: Vec::new(),      // Will be set by caller if needed
        })
    }

    /// Parse property hooks (PHP 8.4)
    fn parse_property_hooks(&mut self) -> Result<Vec<PropertyHook>, String> {
        // Expect opening brace
        if !self.check(&TokenKind::LeftBrace) {
            return Err(format!(
                "Expected '{{' at line {}, column {}",
                self.current().line,
                self.current().column
            ));
        }
        self.advance();

        let mut hooks = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.check(&TokenKind::Eof) {
            // Parse hook: get => expr; or set => expr; or get { ... } or set { ... }
            let hook_type = if self.check(&TokenKind::Get) {
                self.advance();
                PropertyHookType::Get
            } else if self.check(&TokenKind::Set) {
                self.advance();
                PropertyHookType::Set
            } else {
                return Err(format!(
                    "Expected 'get' or 'set' in property hook at line {}",
                    self.current().line
                ));
            };

            let body = if self.check(&TokenKind::DoubleArrow) {
                // Short syntax: get => expr;
                self.advance();
                let expr = self.parse_expression(Precedence::None)?;
                if !self.check(&TokenKind::Semicolon) {
                    return Err(format!(
                        "Expected ';' after property hook expression at line {}",
                        self.current().line
                    ));
                }
                self.advance();
                PropertyHookBody::Expression(Box::new(expr))
            } else if self.check(&TokenKind::LeftBrace) {
                // Block syntax: get { statements }
                self.advance();
                let mut statements = Vec::new();

                while !self.check(&TokenKind::RightBrace) && !self.check(&TokenKind::Eof) {
                    if let Some(stmt) = self.parse_statement()? {
                        statements.push(stmt);
                    }
                }

                if !self.check(&TokenKind::RightBrace) {
                    return Err(format!(
                        "Expected '}}' after property hook block at line {}",
                        self.current().line
                    ));
                }
                self.advance();
                PropertyHookBody::Block(statements)
            } else {
                return Err(format!(
                    "Expected '=>' or '{{' after hook type at line {}",
                    self.current().line
                ));
            };

            hooks.push(PropertyHook { hook_type, body });
        }

        if !self.check(&TokenKind::RightBrace) {
            return Err(format!(
                "Expected '}}' at line {}, column {}",
                self.current().line,
                self.current().column
            ));
        }
        self.advance();

        // Validate hooks
        if hooks.is_empty() {
            return Err("Property hooks cannot be empty".to_string());
        }

        // Check for duplicate hooks
        let get_count = hooks
            .iter()
            .filter(|h| matches!(h.hook_type, PropertyHookType::Get))
            .count();
        let set_count = hooks
            .iter()
            .filter(|h| matches!(h.hook_type, PropertyHookType::Set))
            .count();

        if get_count > 1 {
            return Err("Duplicate 'get' hook in property".to_string());
        }
        if set_count > 1 {
            return Err("Duplicate 'set' hook in property".to_string());
        }

        Ok(hooks)
    }

    /// Parse class method (shared between class and trait)
    pub fn parse_method(
        &mut self,
        visibility: Visibility,
        is_abstract_method: bool,
        is_final_method: bool,
    ) -> Result<Method, String> {
        self.advance(); // consume 'function'

        let name = if let TokenKind::Identifier(name) = &self.current().kind {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(format!(
                "Expected method name at line {}, column {}",
                self.current().line,
                self.current().column
            ));
        };

        // Detect if this is a constructor
        let is_constructor = name.to_lowercase() == "__construct";

        self.consume(TokenKind::LeftParen, "Expected '(' after method name")?;

        let mut params = Vec::new();
        let mut found_variadic = false;
        if !self.check(&TokenKind::RightParen) {
            loop {
                // Parse attributes for this parameter
                let param_attributes = self.parse_attributes()?;

                // Check for visibility modifiers only on constructors
                let param_visibility = if is_constructor {
                    match &self.current().kind {
                        TokenKind::Public => {
                            self.advance();
                            Some(Visibility::Public)
                        }
                        TokenKind::Protected => {
                            self.advance();
                            Some(Visibility::Protected)
                        }
                        TokenKind::Private => {
                            self.advance();
                            Some(Visibility::Private)
                        }
                        _ => None,
                    }
                } else {
                    // Error if visibility is used on non-constructor method
                    match &self.current().kind {
                        TokenKind::Public | TokenKind::Protected | TokenKind::Private => {
                            return Err(format!(
                                "Constructor property promotion can only be used in __construct at line {}, column {}",
                                self.current().line,
                                self.current().column
                            ));
                        }
                        _ => None,
                    }
                };

                // Check for readonly modifier (only in constructors with visibility)
                let param_readonly = if is_constructor && param_visibility.is_some() {
                    if self.check(&TokenKind::Readonly) {
                        self.advance();
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                // Parse type hint if present
                let type_hint = if let TokenKind::Identifier(_) = &self.current().kind {
                    // Check if this looks like a type (not preceded by $)
                    Some(self.parse_type_hint()?)
                } else if self.check(&TokenKind::QuestionMark) {
                    // Nullable type
                    Some(self.parse_type_hint()?)
                } else {
                    None
                };

                let by_ref = if let TokenKind::Identifier(s) = &self.current().kind {
                    if s == "&" {
                        self.advance();
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                // Check for variadic: ...
                let is_variadic = if self.check(&TokenKind::Ellipsis) {
                    self.advance();
                    true
                } else {
                    false
                };

                let param_name = if let TokenKind::Variable(name) = &self.current().kind {
                    let name = name.clone();
                    self.advance();
                    name
                } else {
                    return Err(format!(
                        "Expected parameter name at line {}, column {}",
                        self.current().line,
                        self.current().column
                    ));
                };

                let default = if self.check(&TokenKind::Assign) {
                    if is_variadic {
                        return Err(format!(
                            "Variadic parameter cannot have a default value at line {}, column {}",
                            self.current().line,
                            self.current().column
                        ));
                    }
                    self.advance();
                    Some(self.parse_expression(Precedence::None)?)
                } else {
                    None
                };

                // Variadic must be the last parameter (enforced by comma check below)
                if found_variadic {
                    return Err(format!(
                        "Only the last parameter can be variadic at line {}, column {}",
                        self.current().line,
                        self.current().column
                    ));
                }
                if is_variadic {
                    found_variadic = true;
                }

                params.push(crate::ast::FunctionParam {
                    name: param_name,
                    type_hint,
                    default,
                    by_ref,
                    is_variadic,
                    visibility: param_visibility,
                    readonly: param_readonly,
                    attributes: param_attributes,
                });

                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }

        self.consume(TokenKind::RightParen, "Expected ')' after parameters")?;

        // Parse return type hint if present (: type)
        let return_type = if self.check(&TokenKind::Colon) {
            self.advance();
            Some(self.parse_type_hint()?)
        } else {
            None
        };

        // Abstract methods end with semicolon, concrete methods have body
        let body = if is_abstract_method {
            self.consume(
                TokenKind::Semicolon,
                "Expected ';' after abstract method declaration",
            )?;
            Vec::new()
        } else {
            self.consume(TokenKind::LeftBrace, "Expected '{' before method body")?;

            let mut body = Vec::new();
            while !self.check(&TokenKind::RightBrace) && !self.check(&TokenKind::Eof) {
                if let Some(stmt) = self.parse_statement()? {
                    body.push(stmt);
                }
            }

            self.consume(TokenKind::RightBrace, "Expected '}' after method body")?;
            body
        };

        Ok(Method {
            name,
            visibility,
            is_static: false, // Will be set by caller if needed
            is_abstract: is_abstract_method,
            is_final: is_final_method,
            params,
            return_type,
            body,
            attributes: Vec::new(), // Will be set by caller
        })
    }

    /// Parse expression statement
    pub fn parse_expression_statement(&mut self) -> Result<Stmt, String> {
        let expr = self.parse_expression(Precedence::None)?;

        if self.check(&TokenKind::Semicolon) {
            self.advance();
        } else if !self.check(&TokenKind::CloseTag) && !self.check(&TokenKind::Eof) {
            return Err(format!(
                "Expected ';' after expression at line {}, column {}",
                self.current().line,
                self.current().column
            ));
        }

        Ok(Stmt::Expression(expr))
    }

    /// Parse try/catch/finally statement
    /// try { ... } catch (ExceptionType $e) { ... } finally { ... }
    pub fn parse_try(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'try'

        // Parse try block
        let try_body = self.parse_block()?;

        let mut catch_clauses = Vec::new();
        let mut finally_body = None;

        // Parse catch clauses
        while self.check(&TokenKind::Catch) {
            self.advance(); // consume 'catch'
            self.consume(TokenKind::LeftParen, "Expected '(' after 'catch'")?;

            // Parse exception types (supports Type1 | Type2)
            let mut exception_types = Vec::new();
            loop {
                if let TokenKind::Identifier(name) = &self.current().kind {
                    exception_types.push(name.clone());
                    self.advance();
                } else {
                    return Err(format!(
                        "Expected exception type at line {}, column {}",
                        self.current().line,
                        self.current().column
                    ));
                }

                // Check for multi-catch separator |
                if self.check(&TokenKind::BitwiseOr) {
                    self.advance();
                } else {
                    break;
                }
            }

            // Parse variable name
            let variable = if let TokenKind::Variable(name) = &self.current().kind {
                let name = name.clone();
                self.advance();
                name
            } else {
                return Err(format!(
                    "Expected exception variable at line {}, column {}",
                    self.current().line,
                    self.current().column
                ));
            };

            self.consume(TokenKind::RightParen, "Expected ')' after catch clause")?;
            let catch_body = self.parse_block()?;

            catch_clauses.push(crate::ast::CatchClause {
                exception_types,
                variable,
                body: catch_body,
            });
        }

        // Parse optional finally
        if self.check(&TokenKind::Finally) {
            self.advance(); // consume 'finally'
            finally_body = Some(self.parse_block()?);
        }

        // Must have at least one catch or finally
        if catch_clauses.is_empty() && finally_body.is_none() {
            return Err(format!(
                "Try must have at least one catch or finally block at line {}, column {}",
                self.current().line,
                self.current().column
            ));
        }

        Ok(Stmt::TryCatch {
            try_body,
            catch_clauses,
            finally_body,
        })
    }

    /// Parse throw statement
    pub fn parse_throw_statement(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'throw'
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

        Ok(Stmt::Throw(expr))
    }

    /// Main statement dispatcher
    pub fn parse_statement(&mut self) -> Result<Option<Stmt>, String> {
        // Parse any attributes that may precede declarations
        let attributes = self.parse_attributes()?;

        let token = self.current().clone();
        match token.kind {
            TokenKind::OpenTag => {
                self.advance();
                Ok(None)
            }
            TokenKind::CloseTag => {
                self.advance();
                Ok(None)
            }
            TokenKind::Echo => Ok(Some(self.parse_echo()?)),
            TokenKind::If => Ok(Some(self.parse_if()?)),
            TokenKind::While => Ok(Some(self.parse_while()?)),
            TokenKind::Do => Ok(Some(self.parse_do_while()?)),
            TokenKind::For => Ok(Some(self.parse_for()?)),
            TokenKind::Foreach => Ok(Some(self.parse_foreach()?)),
            TokenKind::Switch => Ok(Some(self.parse_switch()?)),
            TokenKind::Break => Ok(Some(self.parse_break()?)),
            TokenKind::Continue => Ok(Some(self.parse_continue()?)),
            TokenKind::Function => {
                let mut func = self.parse_function()?;
                if let Stmt::Function {
                    attributes: ref mut attrs,
                    ..
                } = func
                {
                    *attrs = attributes;
                }
                Ok(Some(func))
            }
            TokenKind::Class => {
                let mut class = self.parse_class()?;
                if let Stmt::Class {
                    attributes: ref mut attrs,
                    ..
                } = class
                {
                    *attrs = attributes;
                }
                Ok(Some(class))
            }
            TokenKind::Readonly => {
                // readonly can be used before class keyword (PHP 8.2)
                let mut class = self.parse_class()?;
                if let Stmt::Class {
                    attributes: ref mut attrs,
                    ..
                } = class
                {
                    *attrs = attributes;
                }
                Ok(Some(class))
            }
            TokenKind::Abstract => {
                // abstract can be used before class keyword
                let mut class = self.parse_class()?;
                if let Stmt::Class {
                    attributes: ref mut attrs,
                    ..
                } = class
                {
                    *attrs = attributes;
                }
                Ok(Some(class))
            }
            TokenKind::Final => {
                // final can be used before class keyword
                let mut class = self.parse_class()?;
                if let Stmt::Class {
                    attributes: ref mut attrs,
                    ..
                } = class
                {
                    *attrs = attributes;
                }
                Ok(Some(class))
            }
            TokenKind::Interface => {
                let mut iface = self.parse_interface()?;
                if let Stmt::Interface {
                    attributes: ref mut attrs,
                    ..
                } = iface
                {
                    *attrs = attributes;
                }
                Ok(Some(iface))
            }
            TokenKind::Trait => {
                let mut trait_stmt = self.parse_trait()?;
                if let Stmt::Trait {
                    attributes: ref mut attrs,
                    ..
                } = trait_stmt
                {
                    *attrs = attributes;
                }
                Ok(Some(trait_stmt))
            }
            TokenKind::Enum => {
                let mut enum_stmt = self.parse_enum()?;
                if let Stmt::Enum {
                    attributes: ref mut attrs,
                    ..
                } = enum_stmt
                {
                    *attrs = attributes;
                }
                Ok(Some(enum_stmt))
            }
            TokenKind::Return => Ok(Some(self.parse_return()?)),
            TokenKind::Try => Ok(Some(self.parse_try()?)),
            TokenKind::Throw => Ok(Some(self.parse_throw_statement()?)),
            TokenKind::Namespace => Ok(Some(self.parse_namespace()?)),
            TokenKind::Use => {
                // Distinguish between use statements (at top level) and trait use (in class)
                // This is a top-level use statement (namespace import)
                Ok(Some(self.parse_use_statement()?))
            }
            TokenKind::Declare => Ok(Some(self.parse_declare()?)),
            TokenKind::Html(html) => {
                self.advance();
                Ok(Some(Stmt::Html(html)))
            }
            TokenKind::Eof => Ok(None),
            TokenKind::Variable(_)
            | TokenKind::Integer(_)
            | TokenKind::Float(_)
            | TokenKind::String(_)
            | TokenKind::True
            | TokenKind::False
            | TokenKind::Null
            | TokenKind::LeftParen
            | TokenKind::Minus
            | TokenKind::Not
            | TokenKind::Increment
            | TokenKind::Decrement
            | TokenKind::Identifier(_)
            | TokenKind::New => Ok(Some(self.parse_expression_statement()?)),
            _ => Err(format!(
                "Unexpected token {:?} at line {}, column {}",
                token.kind, token.line, token.column
            )),
        }
    }

    /// Parse namespace declaration
    /// namespace Foo\Bar { ... } or namespace Foo\Bar; or namespace { ... }
    pub fn parse_namespace(&mut self) -> Result<Stmt, String> {
        use crate::ast::NamespaceBody;

        self.advance(); // consume 'namespace'

        // Check if this is a global namespace block: namespace { ... }
        // or a named namespace
        let name = if self.check(&TokenKind::LeftBrace) || self.check(&TokenKind::Semicolon) {
            None // Global namespace
        } else {
            Some(self.parse_qualified_name()?)
        };

        // Determine body style
        let body = if self.check(&TokenKind::LeftBrace) {
            // Braced namespace: namespace Foo { ... }
            self.advance(); // consume {
            let mut stmts = Vec::new();
            while !self.check(&TokenKind::RightBrace) && !self.check(&TokenKind::Eof) {
                if let Some(stmt) = self.parse_statement()? {
                    stmts.push(stmt);
                }
            }
            self.consume(TokenKind::RightBrace, "Expected '}' after namespace body")?;
            NamespaceBody::Braced(stmts)
        } else {
            // Unbraced namespace: namespace Foo; (rest of file)
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

        // First part
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

        // Additional parts after \
        while self.check(&TokenKind::Backslash) {
            // Peek ahead to see if there's an identifier after the backslash
            // Don't consume the backslash if it's followed by { (for group use)
            let next_idx = *self.pos + 1;
            if next_idx < self.tokens.len() {
                if let TokenKind::LeftBrace = &self.tokens[next_idx].kind {
                    // Stop here - this backslash is part of group use syntax
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
        self.advance(); // consume 'declare'
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
                    let value = self.parse_expression(Precedence::None)?;
                    match value {
                        Expr::Integer(0) => DeclareDirective::StrictTypes(false),
                        Expr::Integer(1) => DeclareDirective::StrictTypes(true),
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
                    let value = self.parse_expression(Precedence::None)?;
                    match value {
                        Expr::String(s) => DeclareDirective::Encoding(s),
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
                    let value = self.parse_expression(Precedence::None)?;
                    match value {
                        Expr::Integer(n) => DeclareDirective::Ticks(n),
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

        self.consume(TokenKind::RightParen, "Expected ')' after declare directives")?;

        // Check for block syntax: declare(...) { ... }
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
            // Alternative syntax: declare(...): ... enddeclare;
            self.advance();
            let mut stmts = Vec::new();
            loop {
                // Check for enddeclare
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
            // File-scope: declare(...);
            self.consume(TokenKind::Semicolon, "Expected ';' after declare")?;
            None
        };

        Ok(Stmt::Declare { directives, body })
    }

    /// Parse use statement
    /// use Foo\Bar; use Foo\Bar as Baz; use function Foo\bar; use const Foo\BAR;
    /// use Foo\{Bar, Baz}; (group use)
    pub fn parse_use_statement(&mut self) -> Result<Stmt, String> {
        use crate::ast::{UseItem, UseType};

        self.advance(); // consume 'use'

        // Check for `use function` or `use const`
        let default_type = if self.check(&TokenKind::Function) {
            self.advance();
            UseType::Function
        } else if self.check(&TokenKind::Const) {
            self.advance();
            UseType::Constant
        } else {
            UseType::Class
        };

        // Parse the name
        let name = self.parse_qualified_name()?;

        // Check for group use: use Foo\{Bar, Baz}
        // First consume the backslash before the brace if present
        if self.check(&TokenKind::Backslash) {
            self.advance(); // consume the \ before {
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

        // Parse single or multiple uses
        let mut items = vec![];

        // First item
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

        // Additional items after comma
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

        self.advance(); // consume '{'

        let mut items = vec![];

        loop {
            // Check for type modifier
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
